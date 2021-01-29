use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ffi::c_void;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use bimap::BiMap;
use by_address::ByAddress;
use libloading::Library;

use classfile_view::loading::{LivePoolGetter, LoaderIndex, LoaderName};
use classfile_view::view::ClassView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{JavaVM, jint, jlong, JNIInvokeInterface_, jobject};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;
use verification::ClassFileGetter;

use crate::class_loading::{assert_inited_or_initing_class, check_loaded_class};
use crate::field_table::FieldTable;
use crate::interpreter_state::InterpreterStateGuard;
use crate::invoke_interface::get_invoke_interface;
use crate::java_values::{JavaValue, NormalObject, Object};
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::loading::Classpath;
use crate::method_table::{MethodId, MethodTable};
use crate::native_allocation::NativeAllocator;
use crate::options::{JVMOptions, SharedLibraryPaths};
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::threading::ThreadState;
use crate::tracing::TracingSettings;

pub struct JVMState {
    pub(crate) properties: Vec<String>,
    loaders: RwLock<HashMap<LoaderName, Arc<Object>>>,
    // pub bootstrap_loader: LoaderArc,//todo what Should this be?
    pub system_domain_loader: bool,
    pub string_pool: StringPool,
    pub start_instant: Instant,
    //todo needs to be used for all instances of getClass
    pub libjava: LibJavaLoading,

    pub classes: RwLock<Classes>,
    pub class_loaders: RwLock<BiMap<LoaderIndex, ByAddress<Arc<Object>>>>,
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
    pub resolved_method_handles: RwLock<HashMap<ByAddress<Arc<Object>>, MethodId>>,
}

pub struct Classes {
    pub loaded_classes_by_type: HashMap<LoaderName, HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub initiating_loaders: HashMap<PTypeView, (LoaderName, Arc<RuntimeClass>)>,
    pub class_object_pool: BiMap<ByAddress<Arc<Object>>, ByAddress<Arc<RuntimeClass>>>,
    pub anon_classes: RwLock<Vec<Arc<RuntimeClass>>>,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,
    pub class_class: Arc<RuntimeClass>,
}

#[derive(Debug, Copy, Clone)]
pub enum ClassStatus {
    UNPREPARED,
    PREPARED,
    INITIALIZING,
    INITIALIZED,
}

impl Classes {
    pub fn get_loaded_classes(&self) -> impl Iterator<Item=(LoaderName, PTypeView)> + '_ {
        self.loaded_classes_by_type.iter().flat_map(|(l, rc)| rc.keys().map(move |ptype| (*l, ptype.clone())))
    }


    pub fn is_loaded(&self, ptype: &PTypeView) -> Option<Arc<RuntimeClass>> {
        todo!()
    }

    pub fn get_initiating_loader(&self, class_: &Arc<RuntimeClass>) -> LoaderName {
        let (res, actual_class) = self.initiating_loaders.get(&class_.ptypeview()).unwrap();
        assert!(Arc::ptr_eq(class_, actual_class));
        *res
    }
}


impl JVMState {
    pub fn new(jvm_options: JVMOptions) -> (Vec<String>, Self) {
        let JVMOptions { main_class_name, classpath, args, shared_libs, enable_tracing, enable_jvmti, properties, unittest_mode } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let classpath_arc = Arc::new(classpath);


        let tracing = if enable_tracing { TracingSettings::new() } else { TracingSettings::disabled() };

        let jvmti_state = if enable_jvmti {
            JVMTIState {
                built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(libjdwp.as_str())),
                break_points: RwLock::new(HashMap::new()),
                tags: RwLock::new(HashMap::new()),
            }.into()
        } else { None };
        let thread_state = ThreadState::new();
        let classes = JVMState::init_classes(&classpath_arc);
        let string_pool = StringPool {
            entries: HashSet::new()
        };
        let mut jvm = Self {
            properties,
            loaders: RwLock::new(HashMap::new()),
            system_domain_loader: true,
            libjava: LibJavaLoading::new_java_loading(libjava),
            string_pool,
            start_instant: Instant::now(),
            classes,
            class_loaders: RwLock::new(BiMap::new()),
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
            resolved_method_handles: RwLock::new(HashMap::new()),
        };
        jvm.add_class_class_class_object();
        (args, jvm)
    }

    fn add_class_class_class_object(&mut self) {
        let mut classes = self.classes.write().unwrap();
        let class_object = Arc::new(Object::Object(NormalObject {
            monitor: self.thread_state.new_monitor("class class object monitor".to_string()),
            fields: UnsafeCell::new(Default::default()),
            class_pointer: classes.class_class.clone(),
        }));
        let runtime_class = ByAddress(classes.class_class.clone());
        classes.class_object_pool.insert(ByAddress(class_object), runtime_class);
    }

    fn init_classes(classpath_arc: &Arc<Classpath>) -> RwLock<Classes> {
        //todo turn this into a ::new
        let class_class = Arc::new(RuntimeClass::Object(RuntimeClassClass {
            class_view: Arc::new(ClassView::from(classpath_arc.lookup(&ClassName::class()).unwrap())),
            static_vars: Default::default(),
            status: ClassStatus::UNPREPARED.into(),
        }));
        let mut initiating_loaders: HashMap<PTypeView, (LoaderName, Arc<RuntimeClass>), RandomState> = Default::default();
        initiating_loaders.insert(ClassName::class().into(), (LoaderName::BootstrapLoader, class_class.clone()));
        let mut class_object_pool: BiMap<ByAddress<Arc<Object>>, ByAddress<Arc<RuntimeClass>>> = Default::default();
        let classes = RwLock::new(Classes {
            loaded_classes_by_type: Default::default(),
            initiating_loaders,
            class_object_pool,
            anon_classes: Default::default(),
            anon_class_live_object_ldc_pool: Arc::new(RwLock::new(Vec::new())),
            class_class,
        });
        classes
    }

    pub fn get_or_create_bootstrap_object_loader(&self, int_state: &mut InterpreterStateGuard) -> JavaValue {//todo this should really take frame as a parameter
        if !self.vm_live() {
            return JavaValue::Object(None);
        }
        let mut loader_guard = self.loaders.write().unwrap();
        match loader_guard.get(&LoaderName::BootstrapLoader) {
            None => {
                let java_lang_class_loader = ClassName::new("java/lang/ClassLoader");
                let class_loader_class = assert_inited_or_initing_class(self, int_state, java_lang_class_loader.into());
                let res = Arc::new(Object::Object(NormalObject {
                    monitor: self.thread_state.new_monitor("bootstrap loader object monitor".to_string()),
                    fields: UnsafeCell::new(HashMap::new()),
                    class_pointer: class_loader_class,
                }));
                loader_guard.insert(LoaderName::BootstrapLoader, res.clone());
                JavaValue::Object(res.into())
            }
            Some(res) => { JavaValue::Object(res.clone().into()) }
        }
    }

    pub unsafe fn get_int_state<'l>(&self) -> &'l mut InterpreterStateGuard<'l> {
        assert!(self.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        let ptr = self.thread_state.int_state_guard.with(|refcell| *refcell.borrow().as_ref().unwrap());
        let res = transmute::<&mut InterpreterStateGuard<'static>, &mut InterpreterStateGuard<'l>>(ptr.as_mut().unwrap());//todo make this less sketch maybe
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
    // pub libzip: Library,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass>>, RwLock<HashMap<u16, unsafe extern fn()>>>>,
}

impl LibJavaLoading {
    pub unsafe fn load(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
        for library in vec![&self.libjava, &self.libnio, &self.libawt, &self.libxawt/*, &self.libzip*/] {//todo reenable
            let on_load = library.get::<fn(vm: *mut JavaVM, reserved: *mut c_void) -> jint>("JNI_OnLoad".as_bytes()).unwrap();
            let onload_fn_ptr = on_load.deref();
            let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm, int_state);
            // dbg!(interface);
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
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.classes.read().unwrap().anon_class_live_object_ldc_pool.clone() })
    }
}

impl JVMState {
    pub fn get_class_getter<'l>(&'l self, loader: LoaderName) -> Arc<dyn ClassFileGetter + 'l> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Arc::new(BootstrapLoaderClassGetter {
            jvm: self
        })
    }
}

pub struct BootstrapLoaderClassGetter<'l> {
    jvm: &'l JVMState
}

impl ClassFileGetter for BootstrapLoaderClassGetter<'_> {
    fn get_classfile(&self, loader: LoaderName, class: ClassName) -> Arc<Classfile> {
        // assert_eq!(loader, LoaderName::BootstrapLoader);
        todo!()
    }
}
