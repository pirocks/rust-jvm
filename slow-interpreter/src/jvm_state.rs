use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ffi::{c_void, OsString};
use std::iter::FromIterator;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use bimap::BiMap;
use by_address::ByAddress;
use crossbeam::thread::Scope;
use itertools::Itertools;
use libloading::{Error, Library, Symbol};
use libloading::os::unix::{RTLD_GLOBAL, RTLD_LAZY};

use classfile_view::loading::{LivePoolGetter, LoaderIndex, LoaderName};
use classfile_view::view::ClassBackedView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use gc_memory_layout_common::FrameBackedStackframeMemoryLayout;
use jvmti_jni_bindings::{JavaVM, jint, jlong, JNIInvokeInterface_, jobject};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;
use verification::{ClassFileGetter, VerifierContext, verify};
use verification::verifier::Frame;

use crate::class_loading::{DefaultClassfileGetter, DefaultLivePoolGetter};
use crate::field_table::FieldTable;
use crate::interpreter_state::InterpreterStateGuard;
use crate::invoke_interface::get_invoke_interface;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::stack_trace_element::StackTraceElement;
use crate::java_values::{GC, GcManagedObject, JavaValue, NativeJavaValue, NormalObject, Object, ObjectFieldsAndClass};
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::loading::Classpath;
use crate::method_table::{MethodId, MethodTable};
use crate::native_allocation::NativeAllocator;
use crate::options::{JVMOptions, SharedLibraryPaths};
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::stack_entry::RuntimeClassClassId;
use crate::threading::safepoints::Monitor2;
use crate::threading::ThreadState;
use crate::tracing::TracingSettings;

pub static mut JVM: Option<&'static JVMState> = None;


pub struct JVMState<'gc_life> {
    pub libjava_path: OsString,
    pub(crate) properties: Vec<String>,
    pub system_domain_loader: bool,
    pub string_pool: StringPool,
    pub string_internment: RwLock<StringInternment<'gc_life>>,
    pub start_instant: Instant,
    pub libjava: LibJavaLoading<'gc_life>,

    pub classes: RwLock<Classes<'gc_life>>,
    pub class_loaders: RwLock<BiMap<LoaderIndex, ByAddress<GcManagedObject<'gc_life>>>>,
    pub gc: &'gc_life GC<'gc_life>,
    pub protection_domains: RwLock<BiMap<ByAddress<Arc<RuntimeClass<'gc_life>>>, ByAddress<GcManagedObject<'gc_life>>>>,
    pub main_class_name: ClassName,

    pub classpath: Arc<Classpath>,
    pub(crate) invoke_interface: RwLock<Option<*const JNIInvokeInterface_>>,

    pub jvmti_state: Option<JVMTIState>,
    pub thread_state: ThreadState<'gc_life>,
    pub tracing: TracingSettings,
    pub method_table: RwLock<MethodTable<'gc_life>>,
    pub stack_frame_layouts: RwLock<HashMap<MethodId, FrameBackedStackframeMemoryLayout>>,
    pub field_table: RwLock<FieldTable<'gc_life>>,
    pub native_interface_allocations: NativeAllocator,
    pub(crate) live: AtomicBool,
    pub unittest_mode: bool,
    pub resolved_method_handles: RwLock<HashMap<ByAddress<GcManagedObject<'gc_life>>, MethodId>>,

    pub include_name_field: AtomicBool,
    pub store_generated_classes: bool,
    pub debug_print_exceptions: bool,
    pub assertions_enabled: bool,

    pub stacktraces_by_throwable: RwLock<HashMap<ByAddress<GcManagedObject<'gc_life>>, Vec<StackTraceElement<'gc_life>>>>,

    pub monitors2: RwLock<Vec<Monitor2>>,

    pub function_frame_type_data: RwLock<HashMap<MethodId, HashMap<u16, Frame>>>,
}

pub struct Classes<'gc_life> {
    //todo needs to be used for all instances of getClass
    pub loaded_classes_by_type: HashMap<LoaderName, HashMap<PTypeView, Arc<RuntimeClass<'gc_life>>>>,
    pub initiating_loaders: HashMap<PTypeView, (LoaderName, Arc<RuntimeClass<'gc_life>>)>,
    pub class_object_pool: BiMap<ByAddress<GcManagedObject<'gc_life>>, ByAddress<Arc<RuntimeClass<'gc_life>>>>,
    pub anon_classes: RwLock<Vec<Arc<RuntimeClass<'gc_life>>>>,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<GcManagedObject<'gc_life>>>>,
    pub class_class: Arc<RuntimeClass<'gc_life>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClassStatus {
    UNPREPARED,
    PREPARED,
    INITIALIZING,
    INITIALIZED,
}

impl<'gc_life> Classes<'gc_life> {
    pub fn get_loaded_classes(&self) -> Vec<(LoaderName, PTypeView)> {
        self.loaded_classes_by_type.iter().flat_map(|(l, rc)| rc.keys().map(move |ptype| (*l, ptype.clone()))).collect_vec()
    }


    pub fn is_loaded(&self, ptype: &PTypeView) -> Option<Arc<RuntimeClass<'gc_life>>> {
        self.initiating_loaders.get(&ptype)?.1.clone().into()
    }

    pub fn get_initiating_loader(&self, class_: &Arc<RuntimeClass<'gc_life>>) -> LoaderName {
        let (res, actual_class) = self.initiating_loaders.get(&class_.ptypeview()).unwrap();
        assert!(Arc::ptr_eq(class_, actual_class));
        *res
    }

    pub fn get_class_obj(&self, ptypeview: PTypeView) -> Option<GcManagedObject<'gc_life>> {
        let runtime_class = self.initiating_loaders.get(&ptypeview)?.1.clone();
        let obj = self.class_object_pool.get_by_right(&ByAddress(runtime_class.clone())).unwrap().clone().0;
        Some(obj)
    }

    pub fn convert_runtime_class_class_id(&self, id: RuntimeClassClassId) -> &RuntimeClassClass {
        todo!()
    }
}


impl<'gc_life> JVMState<'gc_life> {
    pub fn new(jvm_options: JVMOptions, scope: Scope<'gc_life>, gc: &'gc_life GC<'gc_life>) -> (Vec<String>, Self) {
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
        let thread_state = ThreadState::new(scope);
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
            gc,
            protection_domains: RwLock::new(BiMap::new()),
            main_class_name,
            classpath: classpath_arc,
            invoke_interface: RwLock::new(None),
            jvmti_state,
            thread_state,
            tracing,
            method_table: RwLock::new(MethodTable::new()),
            stack_frame_layouts: RwLock::new(HashMap::new()),
            field_table: RwLock::new(FieldTable::new()),
            native_interface_allocations: NativeAllocator { allocations: RwLock::new(HashMap::new()) },
            live: AtomicBool::new(false),
            unittest_mode,
            resolved_method_handles: RwLock::new(HashMap::new()),
            include_name_field: AtomicBool::new(false),
            store_generated_classes,
            debug_print_exceptions,
            assertions_enabled,
            stacktraces_by_throwable: RwLock::new(HashMap::new()),
            monitors2: RwLock::new(vec![]),
            function_frame_type_data: Default::default(),
        };
        jvm.add_class_class_class_object();
        (args, jvm)
    }

    pub fn sink_function_verification_date(&self, verification_types: &HashMap<u16, HashMap<CodeIndex, Frame>>, rc: Arc<RuntimeClass<'gc_life>>) {
        let mut method_table = self.method_table.write().unwrap();
        for (method_i, verification_types) in verification_types {
            let method_id = method_table.get_method_id(rc.clone(), *method_i);
            self.function_frame_type_data.write().unwrap().insert(method_id, verification_types.clone());
        }
    }

    pub fn verify_class_and_object(&self, object_runtime_class: Arc<RuntimeClass<'gc_life>>, class_runtime_class: Arc<RuntimeClass<'gc_life>>) {
        let mut context = VerifierContext {
            live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
            classfile_getter: Arc::new(DefaultClassfileGetter {
                jvm: self
            }) as Arc<dyn ClassFileGetter>,
            current_loader: LoaderName::BootstrapLoader,
            verification_types: HashMap::new(),
            debug: false,
        };
        let lookup = self.classpath.lookup(&ClassName::object()).expect("Can not find Object class");
        verify(&mut context, &ClassBackedView::from(lookup), LoaderName::BootstrapLoader).expect("Object doesn't verify");
        self.sink_function_verification_date(&context.verification_types, object_runtime_class);
        context.verification_types.clear();
        let lookup = self.classpath.lookup(&ClassName::class()).expect("Can not find Class class");
        verify(&mut context, &ClassBackedView::from(lookup), LoaderName::BootstrapLoader).expect("Class doesn't verify");
        self.sink_function_verification_date(&context.verification_types, class_runtime_class);
    }

    fn add_class_class_class_object(&mut self) {
        let mut classes = self.classes.write().unwrap();
        //todo desketchify this
        let mut fields: HashMap<String, JavaValue<'gc_life>, RandomState> = Default::default();
        fields.insert("name".to_string(), JavaValue::null());
        fields.insert("classLoader".to_string(), JavaValue::null());
        const MAX_LOCAL_VARS: i32 = 100;
        let class_object = self.allocate_object(Object::Object(NormalObject {
            monitor: self.thread_state.new_monitor("class class object monitor".to_string()),
            objinfo: ObjectFieldsAndClass {
                fields: (0..MAX_LOCAL_VARS).map(|_| UnsafeCell::new(NativeJavaValue { object: null_mut() })).collect_vec(),
                class_pointer: classes.class_class.clone(),
            },
        }));
        let runtime_class = ByAddress(classes.class_class.clone());
        classes.class_object_pool.insert(ByAddress(class_object), runtime_class);
    }

    fn init_classes(classpath_arc: &Arc<Classpath>) -> RwLock<Classes<'gc_life>> {
        //todo turn this into a ::new
        let field_numbers = JVMState::get_class_field_numbers();
        let class_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&ClassName::class()).unwrap()));
        let static_vars = Default::default();
        let parent = None;
        let interfaces = vec![];
        let status = ClassStatus::UNPREPARED.into();
        let class_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new(class_view, field_numbers, static_vars, parent, interfaces, status)));
        let mut initiating_loaders: HashMap<PTypeView, (LoaderName, Arc<RuntimeClass<'gc_life>>), RandomState> = Default::default();
        initiating_loaders.insert(ClassName::class().into(), (LoaderName::BootstrapLoader, class_class.clone()));
        let class_object_pool: BiMap<ByAddress<GcManagedObject<'gc_life>>, ByAddress<Arc<RuntimeClass<'gc_life>>>> = Default::default();
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

    pub fn get_class_field_numbers() -> HashMap<String, (usize, PTypeView)> {
        let class_class_fields = vec![
            ("cachedConstructor", ClassName::constructor().into()),
            ("newInstanceCallerCache", ClassName::class().into()),
            ("name", ClassName::string().into()),
            ("classLoader", ClassName::classloader().into()),
            ("reflectionData", PTypeView::object()),
            ("classRedefinedCount", PTypeView::IntType),
            ("genericInfo", PTypeView::object()),
            ("enumConstants", PTypeView::array(PTypeView::object())),
            ("enumConstantDirectory", PTypeView::object()),
            ("annotationData", PTypeView::object()),
            ("annotationType", PTypeView::object()),
            ("classValueMap", PTypeView::object()),
        ];
        let field_numbers = HashMap::from_iter(class_class_fields.iter().cloned().sorted_by_key(|(name, _)| name.to_string()).enumerate().map(|(_1, (_2_name, _2_type))| ((_2_name.to_string()), (_1, _2_type))).collect_vec().into_iter());
        field_numbers
    }


    pub unsafe fn get_int_state<'l, 'interpreter_guard>(&self) -> &'l mut InterpreterStateGuard<'l, 'interpreter_guard> {
        assert!(self.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        let ptr = self.thread_state.int_state_guard.with(|refcell| *refcell.borrow().as_ref().unwrap());
        let res = transmute::<&mut InterpreterStateGuard<'static, 'static>, &mut InterpreterStateGuard<'l, 'interpreter_guard>>(ptr.as_mut().unwrap());//todo make this less sketch maybe
        assert!(res.registered);
        res
    }

    pub fn get_loader_obj(&self, loader: LoaderName) -> Option<ClassLoader> {
        match loader {
            LoaderName::UserDefinedLoader(loader_idx) => {
                let guard = self.class_loaders.read().unwrap();
                let jvalue = JavaValue::Object(todo!()/*guard.get_by_left(&loader_idx).unwrap().clone().0.into()*/);
                Some(jvalue.cast_class_loader())
            }
            LoaderName::BootstrapLoader => None
        }
    }

    pub fn allocate_object(&self, object: Object<'gc_life>) -> GcManagedObject<'gc_life> {
        self.gc.allocate_object(object)
    }
}


type CodeIndex = u16;

pub struct JVMTIState {
    pub built_in_jdwp: Arc<SharedLibJVMTI>,
    pub break_points: RwLock<HashMap<MethodId, HashSet<CodeIndex>>>,
    pub tags: RwLock<HashMap<jobject, jlong>>,
}

struct LivePoolGetterImpl<'gc_life> {
    anon_class_live_object_ldc_pool: Arc<RwLock<Vec<GcManagedObject<'gc_life>>>>,
}

#[derive(Debug)]
pub struct NativeLib {
    pub library: Library,
}


#[derive(Debug)]
pub struct LibJavaLoading<'gc_life> {
    pub native_libs: RwLock<HashMap<String, NativeLib>>,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass<'gc_life>>>, RwLock<HashMap<u16, unsafe extern fn()>>>>,
}

impl<'gc_life> LibJavaLoading<'gc_life> {
    pub unsafe fn load(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, path: &OsString, name: String) {
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

impl<'gc_life> LivePoolGetter for LivePoolGetterImpl<'gc_life> {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.read().unwrap()[idx];
        JavaValue::Object(todo!()/*object.clone().into()*/).to_type().unwrap_ref_type().clone()
    }
}

pub struct NoopLivePoolGetter {}

impl LivePoolGetter for NoopLivePoolGetter {
    fn elem_type(&self, _idx: usize) -> ReferenceTypeView {
        panic!()
    }
}


impl<'gc_life> JVMState<'gc_life> {
    pub fn vm_live(&self) -> bool {
        self.live.load(Ordering::SeqCst)
    }

    pub fn get_live_object_pool_getter(&'l self) -> Arc<dyn LivePoolGetter + 'l> {
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.classes.read().unwrap().anon_class_live_object_ldc_pool.clone() })
    }

    pub fn get_class_getter(&'l self, loader: LoaderName) -> Arc<dyn ClassFileGetter + 'l> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Arc::new(BootstrapLoaderClassGetter {
            jvm: self
        })
    }
}

pub struct BootstrapLoaderClassGetter<'vm_life, 'l> {
    jvm: &'l JVMState<'vm_life>,
}

impl ClassFileGetter for BootstrapLoaderClassGetter<'_, '_> {
    fn get_classfile(&self, loader: LoaderName, class: ClassName) -> Arc<Classfile> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        self.jvm.classpath.lookup(&class).unwrap()
    }
}


pub struct StringInternment<'gc_life> {
    pub strings: HashMap<Vec<u16>, GcManagedObject<'gc_life>>,
}