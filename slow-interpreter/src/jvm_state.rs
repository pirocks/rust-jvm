use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ffi::{c_void, OsString};
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use bimap::BiMap;
use by_address::ByAddress;
use libloading::{Error, Library, Symbol};
use libloading::os::unix::{RTLD_GLOBAL, RTLD_LAZY};

use classfile_view::loading::{LivePoolGetter, LoaderIndex, LoaderName};
use classfile_view::view::ClassBackedView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{JavaVM, jint, jlong, JNIInvokeInterface_, jobject};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classfile::ConstantKind::LiveObject;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;
use verification::ClassFileGetter;

use crate::field_table::FieldTable;
use crate::interpreter_state::InterpreterStateGuard;
use crate::invoke_interface::get_invoke_interface;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::stack_trace_element::StackTraceElement;
use crate::java_values::{JavaValue, NormalObject, Object};
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::loading::Classpath;
use crate::method_table::{MethodId, MethodTable};
use crate::native_allocation::NativeAllocator;
use crate::options::{JVMOptions, SharedLibraryPaths};
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::threading::safepoints::Monitor2;
use crate::threading::ThreadState;
use crate::tracing::TracingSettings;

pub static mut JVM: Option<JVMState> = None;


pub struct JVMState {
    pub libjava_path: OsString,
    pub(crate) properties: Vec<String>,
    pub system_domain_loader: bool,
    pub string_pool: StringPool,
    pub string_internment: RwLock<StringInternment>,
    pub start_instant: Instant,
    pub libjava: LibJavaLoading,

    pub classes: RwLock<Classes>,
    pub class_loaders: RwLock<BiMap<LoaderIndex, ByAddress<Arc<Object>>>>,
    pub protection_domains: RwLock<BiMap<ByAddress<Arc<RuntimeClass>>, ByAddress<Arc<Object>>>>,
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
    pub unittest_mode: bool,
    pub resolved_method_handles: RwLock<HashMap<ByAddress<Arc<Object>>, MethodId>>,

    pub include_name_field: AtomicBool,
    pub store_generated_classes: bool,
    pub debug_print_exceptions: bool,
    pub assertions_enabled: bool,

    pub stacktraces_by_throwable: RwLock<HashMap<ByAddress<Arc<Object>>, Vec<StackTraceElement>>>,

    pub monitors2: RwLock<Vec<Monitor2>>,
}

pub struct Classes {
    //todo needs to be used for all instances of getClass
    pub loaded_classes_by_type: HashMap<LoaderName, HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub initiating_loaders: HashMap<PTypeView, (LoaderName, Arc<RuntimeClass>)>,
    pub class_object_pool: BiMap<ByAddress<Arc<Object>>, ByAddress<Arc<RuntimeClass>>>,
    pub anon_classes: RwLock<Vec<Arc<RuntimeClass>>>,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,
    pub class_class: Arc<RuntimeClass>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
        self.initiating_loaders.get(&ptype)?.1.clone().into()
    }

    pub fn get_initiating_loader(&self, class_: &Arc<RuntimeClass>) -> LoaderName {
        let (res, actual_class) = self.initiating_loaders.get(&class_.ptypeview()).unwrap();
        assert!(Arc::ptr_eq(class_, actual_class));
        *res
    }

    pub fn get_class_obj(&self, ptypeview: PTypeView) -> Option<Arc<Object>> {
        let runtime_class = self.initiating_loaders.get(&ptypeview)?.1.clone();
        let obj = self.class_object_pool.get_by_right(&ByAddress(runtime_class.clone())).unwrap().clone().0;
        Some(obj)
    }
}


impl JVMState {
    pub fn new(jvm_options: JVMOptions) -> (Vec<String>, Self) {
        let JVMOptions { main_class_name, classpath, args, shared_libs, enable_tracing, enable_jvmti, properties, unittest_mode, store_generated_classes, debug_print_exceptions, assertions_enabled } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let classpath_arc = Arc::new(classpath);


        let tracing = if enable_tracing { TracingSettings::new() } else { TracingSettings::disabled() };

        let jvmti_state = if enable_jvmti {
            JVMTIState {
                built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(&libjdwp)),
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
            libjava_path: libjava,
            properties,
            system_domain_loader: true,
            libjava: LibJavaLoading::new(),
            string_pool,
            string_internment: RwLock::new(StringInternment { strings: HashMap::new() }),
            start_instant: Instant::now(),
            classes,
            class_loaders: RwLock::new(BiMap::new()),
            protection_domains: RwLock::new(BiMap::new()),
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
            include_name_field: AtomicBool::new(false),
            store_generated_classes,
            debug_print_exceptions,
            assertions_enabled,
            stacktraces_by_throwable: RwLock::new(HashMap::new()),
            monitors2: RwLock::new(vec![]),
        };
        jvm.add_class_class_class_object();
        (args, jvm)
    }

    fn add_class_class_class_object(&mut self) {
        let mut classes = self.classes.write().unwrap();
        //todo desketchify this
        let mut fields: HashMap<String, JavaValue, RandomState> = Default::default();
        fields.insert("name".to_string(), JavaValue::Object(None));
        fields.insert("classLoader".to_string(), JavaValue::Object(None));
        let class_object = Arc::new(Object::Object(NormalObject {
            monitor: self.thread_state.new_monitor("class class object monitor".to_string()),
            fields: UnsafeCell::new(fields),
            class_pointer: classes.class_class.clone(),
        }));
        let runtime_class = ByAddress(classes.class_class.clone());
        classes.class_object_pool.insert(ByAddress(class_object), runtime_class);
    }

    fn init_classes(classpath_arc: &Arc<Classpath>) -> RwLock<Classes> {
        //todo turn this into a ::new
        let class_class = Arc::new(RuntimeClass::Object(RuntimeClassClass {
            class_view: Arc::new(ClassBackedView::from(classpath_arc.lookup(&ClassName::class()).unwrap())),
            static_vars: Default::default(),
            status: ClassStatus::UNPREPARED.into(),
        }));
        let mut initiating_loaders: HashMap<PTypeView, (LoaderName, Arc<RuntimeClass>), RandomState> = Default::default();
        initiating_loaders.insert(ClassName::class().into(), (LoaderName::BootstrapLoader, class_class.clone()));
        let class_object_pool: BiMap<ByAddress<Arc<Object>>, ByAddress<Arc<RuntimeClass>>> = Default::default();
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


    pub unsafe fn get_int_state<'l>(&self) -> &'l mut InterpreterStateGuard<'l> {
        assert!(self.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        let ptr = self.thread_state.int_state_guard.with(|refcell| *refcell.borrow().as_ref().unwrap());
        let res = transmute::<&mut InterpreterStateGuard<'static>, &mut InterpreterStateGuard<'l>>(ptr.as_mut().unwrap());//todo make this less sketch maybe
        assert!(res.registered);
        res
    }

    pub fn get_loader_obj(&self, loader: LoaderName) -> Option<ClassLoader> {
        match loader {
            LoaderName::UserDefinedLoader(loader_idx) => {
                let guard = self.class_loaders.read().unwrap();
                let jvalue = JavaValue::Object(guard.get_by_left(&loader_idx).unwrap().clone().0.into());
                Some(jvalue.cast_class_loader())
            }
            LoaderName::BootstrapLoader => None
        }
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
pub struct NativeLib {
    pub library: Library
}


#[derive(Debug)]
pub struct LibJavaLoading {
    pub native_libs: RwLock<HashMap<String, NativeLib>>,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass>>, RwLock<HashMap<u16, unsafe extern fn()>>>>,
}

impl LibJavaLoading {
    pub unsafe fn load(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, path: &OsString, name: String) {
        let onload_fn_ptr = self.get_onload_ptr_and_add(path, name);
        let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm, int_state);
        onload_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterface_, null_mut());//todo check return res
    }

    pub unsafe fn get_onload_ptr_and_add(&self, path: &OsString, name: String) -> fn(*mut *const JNIInvokeInterface_, *mut c_void) -> i32 {
        let lib = Library::new(path, (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
        let on_load = lib.get::<fn(vm: *mut JavaVM, reserved: *mut c_void) -> jint>("JNI_OnLoad".as_bytes()).unwrap();
        let onload_fn_ptr = *on_load.deref();
        self.native_libs.write().unwrap().insert(name, NativeLib { library: lib });
        onload_fn_ptr
    }

    pub unsafe fn lookup_onload(&self, name: String) -> Result<unsafe extern "system" fn(), LookupError> {
        let guard = self.native_libs.read().unwrap();
        let native_lib = guard.get(&name);
        let result = native_lib.ok_or(LookupError::NoLib)?.library.get("JNI_OnLoad".as_bytes());
        let symbol: Symbol<unsafe extern "system" fn()> = result?;
        Ok(*symbol.deref())
    }
}

pub enum LookupError {
    LibLoading(libloading::Error),
    NoLib,
}

impl From<libloading::Error> for LookupError {
    fn from(err: Error) -> Self {
        LookupError::LibLoading(err)
    }
}

impl LivePoolGetter for LivePoolGetterImpl {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.read().unwrap()[idx];
        JavaValue::Object(object.clone().into()).to_type().unwrap_ref_type().clone()
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
        assert_eq!(loader, LoaderName::BootstrapLoader);
        self.jvm.classpath.lookup(&class).unwrap()
    }
}


pub struct StringInternment {
    pub strings: HashMap<Vec<u16>, Arc<Object>>
}