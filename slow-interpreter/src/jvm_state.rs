use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

use by_address::ByAddress;
use libloading::Library;

use classfile_view::loading::{LivePoolGetter, LoaderArc, LoaderName};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{JavaVM, jint, jlong, JNIInvokeInterface_, jobject};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;

use crate::field_table::FieldTable;
use crate::interpreter_state::InterpreterStateGuard;
use crate::interpreter_util::check_inited_class;
use crate::invoke_interface::get_invoke_interface;
use crate::java_values::{JavaValue, NormalObject, Object};
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::loading::{BootstrapLoader, Classpath};
use crate::method_table::{MethodId, MethodTable};
use crate::native_allocation::NativeAllocator;
use crate::options::{JVMOptions, SharedLibraryPaths};
use crate::runtime_class::RuntimeClass;
use crate::threading::ThreadState;
use crate::tracing::TracingSettings;

pub struct JVMState {
    pub(crate) properties: Vec<String>,
    loaders: RwLock<HashMap<LoaderName, Arc<Object>>>,
    pub bootstrap_loader: LoaderArc,
    pub system_domain_loader: bool,
    pub string_pool: StringPool,
    pub start_instant: Instant,
    //todo needs to be used for all instances of getClass
    pub libjava: LibJavaLoading,

    pub classes: Classes,

    pub main_class_name: ClassName,

    pub classpath: Arc<Classpath>,
    pub(crate) invoke_interface: RwLock<Option<*const JNIInvokeInterface_>>,

    pub jvmti_state: Option<JVMTIState>,
    pub thread_state: ThreadState,
    pub tracing: TracingSettings,
    pub method_table: RwLock<MethodTable>,
    pub field_table: RwLock<FieldTable>,
    pub native_interface_allocations: NativeAllocator,
    pub(crate) live: AtomicBool,
    // pub int_state_guard: &'static LocalKey<RefCell<Option<*mut InterpreterStateGuard<'static>>>>,//so technically isn't 'static, but we need to be able to store this in a localkey

    pub unittest_mode: bool,
    pub resolved_method_handles: RwLock<HashMap<ByAddress<Arc<Object>>, MethodId>>
}

pub struct Classes {
    //todo maybe switch to coarser locking due to probabilty of races here
    pub prepared_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub initializing_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub initialized_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub class_object_pool: RwLock<HashMap<PTypeView, Arc<Object>>>,
    //anon classes
    pub anon_class_counter: AtomicUsize,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,
}


impl JVMState {
    pub fn new(jvm_options: JVMOptions) -> (Vec<String>, Self) {
        let JVMOptions { main_class_name, classpath, args, shared_libs, enable_tracing, enable_jvmti, properties, unittest_mode } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let classpath_arc = Arc::new(classpath);
        let bootstrap_loader = Arc::new(BootstrapLoader {
            loaded: RwLock::new(HashMap::new()),
            parsed: RwLock::new(HashMap::new()),
            name: RwLock::new(LoaderName::BootstrapLoader),
            classpath: classpath_arc.clone(),
        });


        let tracing = if enable_tracing { TracingSettings::new() } else { TracingSettings::disabled() };

        let jvmti_state = if enable_jvmti {
            JVMTIState {
                built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(libjdwp.as_str())),
                break_points: RwLock::new(HashMap::new()),
                tags: RwLock::new(HashMap::new()),
            }.into()
        } else { None };
        let thread_state = ThreadState::new();
        let jvm = Self {
            properties,
            loaders: RwLock::new(HashMap::new()),
            bootstrap_loader,
            system_domain_loader: true,
            libjava: LibJavaLoading::new_java_loading(libjava),
            string_pool: StringPool {
                entries: HashSet::new()
            },
            start_instant: Instant::now(),
            classes: Classes {
                prepared_classes: RwLock::new(HashMap::new()),
                initializing_classes: RwLock::new(HashMap::new()),
                initialized_classes: RwLock::new(HashMap::new()),
                class_object_pool: RwLock::new(HashMap::new()),
                anon_class_live_object_ldc_pool: Arc::new(RwLock::new(vec![])),
                anon_class_counter: AtomicUsize::new(0),
            },
            main_class_name,
            classpath: classpath_arc,
            invoke_interface: RwLock::new(None),
            jvmti_state,
            thread_state,
            tracing,
            method_table: RwLock::new(MethodTable::new()),
            field_table: RwLock::new(FieldTable::new()),
            native_interface_allocations: NativeAllocator { allocations: RwLock::new(HashMap::new()) },
            live: AtomicBool::new(false),
            // int_state_guard: &INT_STATE_GUARD
            unittest_mode,
            resolved_method_handles: RwLock::new(HashMap::new())
        };
        (args, jvm)
    }

    pub fn get_or_create_bootstrap_object_loader(&self, int_state: &mut InterpreterStateGuard) -> JavaValue {//todo this should really take frame as a parameter
        if !self.vm_live() {
            return JavaValue::Object(None);
        }
        let mut loader_guard = self.loaders.write().unwrap();
        match loader_guard.get(&self.bootstrap_loader.name()) {
            None => {
                let java_lang_class_loader = ClassName::new("java/lang/ClassLoader");
                let current_loader = self.bootstrap_loader.clone();// todo is the bootstrap loader object loaded by the bootstrap loder?
                let class_loader_class = check_inited_class(self, int_state, &java_lang_class_loader.into(), current_loader.clone());
                let res = Arc::new(Object::Object(NormalObject {
                    monitor: self.thread_state.new_monitor("bootstrap loader object monitor".to_string()),
                    fields: RefCell::new(HashMap::new()),
                    class_pointer: class_loader_class,
                    class_object_type: None,
                }));
                loader_guard.insert(self.bootstrap_loader.name(), res.clone());
                JavaValue::Object(res.into())
            }
            Some(res) => { JavaValue::Object(res.clone().into()) }
        }
    }

    pub unsafe fn get_int_state<'l>(&self) -> &'l mut InterpreterStateGuard<'l> {
        assert!(self.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        let ptr = self.thread_state.int_state_guard.with(|refcell| *refcell.borrow().as_ref().unwrap());
        let res = transmute::<&mut InterpreterStateGuard<'static>, &mut InterpreterStateGuard<'l>>(ptr.as_mut().unwrap());
        assert!(res.registered);
        res
    }
}


type CodeIndex = isize;

pub struct JVMTIState {
    pub built_in_jdwp: Arc<SharedLibJVMTI>,
    pub break_points: RwLock<HashMap<MethodId, HashSet<CodeIndex>>>,
    pub tags: RwLock<HashMap<jobject, jlong>>,
}

struct LivePoolGetterImpl {
    anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>
}

#[derive(Debug)]
pub struct LibJavaLoading {
    pub libjava: Library,
    pub libnio: Library,
    pub libawt: Library,
    pub libxawt: Library,
    pub registered_natives: RwLock<HashMap<Arc<RuntimeClass>, RwLock<HashMap<u16, unsafe extern fn()>>>>,
}

impl LibJavaLoading {
    pub unsafe fn load(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
        for library in vec![&self.libjava, &self.libnio, &self.libawt, &self.libxawt] {
            let on_load = library.get::<fn(vm: *mut JavaVM, reserved: *mut c_void) -> jint>("JNI_OnLoad".as_bytes()).unwrap();
            let onload_fn_ptr = on_load.deref();
            let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm, int_state);
            dbg!(interface);
            onload_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterface_, null_mut());//todo check return res
        }
        //todo I have no idea why this is needed, but is
        let jvm_symbol = self.libxawt.get::<*mut *mut JavaVM>("jvm".as_bytes()).unwrap();
        let jvm_ptr = jvm_symbol.deref();

        jvm_ptr.write(Box::into_raw(box get_invoke_interface(
            jvm, int_state,
        )) as *mut JavaVM);
    }
}

impl LivePoolGetter for LivePoolGetterImpl {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.read().unwrap()[idx];
        JavaValue::Object(object.clone().into()).to_type().unwrap_ref_type().clone()
        // ReferenceTypeView::Class(object.unwrap_normal_object().class_pointer.view().name())//todo handle arrays
    }
}

pub struct NoopLivePoolGetter {}

impl LivePoolGetter for NoopLivePoolGetter {
    fn elem_type(&self, _idx: usize) -> ReferenceTypeView {
        panic!()
    }
}


impl JVMState {
    pub fn vm_live(&self) -> bool {
        self.live.load(Ordering::SeqCst)
    }

    pub fn get_live_object_pool_getter(&self) -> Arc<dyn LivePoolGetter> {
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.classes.anon_class_live_object_ldc_pool.clone() })
    }
}
