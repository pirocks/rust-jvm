use std::cell::{RefCell, UnsafeCell};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ffi::{c_void, OsString};
use std::iter::FromIterator;
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::LocalKey;
use std::time::Instant;

use bimap::BiMap;
use by_address::ByAddress;
use crossbeam::thread::Scope;
use itertools::Itertools;
use libloading::{Error, Library, Symbol};
use libloading::os::unix::{RTLD_GLOBAL, RTLD_LAZY};

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use jvmti_jni_bindings::{JavaVM, jint, jlong, JNIInvokeInterface_, jobject};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CompressedClassfileStringPool, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::LiveObjectIndex;
use rust_jvm_common::compressed_classfile::descriptors::CompressedMethodDescriptorsPool;
use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName, FieldName};
use rust_jvm_common::cpdtype_table::CPDTypeTable;
use rust_jvm_common::loading::{ClassLoadingError, LivePoolGetter, LoaderIndex, LoaderName};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::opaque_id_table::OpaqueIDs;
use sketch_jvm_version_of_utf8::Utf8OrWtf8::Wtf;
use sketch_jvm_version_of_utf8::wtf8_pool::Wtf8Pool;
use verification::{ClassFileGetter, VerifierContext, verify};
use verification::verifier::{Frame, TypeSafetyError};

use crate::class_loading::{DefaultClassfileGetter, DefaultLivePoolGetter};
use crate::field_table::FieldTable;
use crate::inheritance_method_ids::InheritanceMethodIDs;
use crate::inheritance_vtable::VTables;
use crate::interpreter_state::InterpreterStateGuard;
use crate::invoke_interface::get_invoke_interface;
use crate::ir_to_java_layer::compiler::JavaCompilerMethodAndFrameData;
use crate::ir_to_java_layer::JavaVMStateWrapper;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::stack_trace_element::StackTraceElement;
use crate::java_values::{ByAddressGcManagedObject, GC, GcManagedObject, JavaValue, NativeJavaValue, NormalObject, Object, ObjectFieldsAndClass};
use crate::jit::state::{JITedCodeState, JITSTATE};
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::loading::Classpath;
use crate::method_table::MethodTable;
use crate::native_allocation::NativeAllocator;
use crate::options::{JVMOptions, SharedLibraryPaths};
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::stack_entry::RuntimeClassClassId;
use crate::threading::safepoints::Monitor2;
use crate::threading::ThreadState;
use crate::tracing::TracingSettings;

pub static mut JVM: Option<&'static JVMState> = None;

pub struct JVMConfig {
    pub compiled_mode_active: bool,
    pub store_generated_classes: bool,
    pub debug_print_exceptions: bool,
    pub assertions_enabled: bool,
    pub tracing: TracingSettings,
    pub main_class_name: CClassName,
}

pub struct Native {
    pub jvmti_state: Option<JVMTIState>,
    pub invoke_interface: RwLock<Option<*const JNIInvokeInterface_>>,
    pub native_interface_allocations: NativeAllocator,
}

pub struct JVMState<'gc_life> {
    pub config: JVMConfig,
    pub java_vm_state: JavaVMStateWrapper<'gc_life>,
    pub gc: &'gc_life GC<'gc_life>,
    pub jit_state: &'static LocalKey<RefCell<JITedCodeState>>,
    pub native_libaries: NativeLibraries<'gc_life>,
    pub properties: Vec<String>,
    pub string_pool: CompressedClassfileStringPool,
    pub string_internment: RwLock<StringInternment<'gc_life>>,
    pub start_instant: Instant,
    pub classes: RwLock<Classes<'gc_life>>,
    pub classpath: Arc<Classpath>,
    pub thread_state: ThreadState<'gc_life>,
    pub method_table: RwLock<MethodTable<'gc_life>>,
    pub field_table: RwLock<FieldTable<'gc_life>>,
    pub wtf8_pool: Wtf8Pool,
    pub cpdtype_table: RwLock<CPDTypeTable>,
    pub opaque_ids: RwLock<OpaqueIDs>,
    pub native: Native,
    pub live: AtomicBool,
    pub resolved_method_handles: RwLock<HashMap<ByAddress<GcManagedObject<'gc_life>>, MethodId>>,
    pub include_name_field: AtomicBool,
    pub stacktraces_by_throwable: RwLock<HashMap<ByAddress<GcManagedObject<'gc_life>>, Vec<StackTraceElement<'gc_life>>>>,
    pub function_frame_type_data: RwLock<HashMap<MethodId, HashMap<ByteCodeOffset, Frame>>>,
    pub java_function_frame_data: RwLock<HashMap<MethodId, JavaCompilerMethodAndFrameData>>,
    pub vtables: RwLock<VTables>,
    pub inheritance_ids: RwLock<InheritanceMethodIDs>
}

pub struct Classes<'gc_life> {
    //todo needs to be used for all instances of getClass
    pub loaded_classes_by_type: HashMap<LoaderName, HashMap<CPDType, Arc<RuntimeClass<'gc_life>>>>,
    pub initiating_loaders: HashMap<CPDType, (LoaderName, Arc<RuntimeClass<'gc_life>>)>,
    pub(crate) class_object_pool: BiMap<ByAddressGcManagedObject<'gc_life>, ByAddress<Arc<RuntimeClass<'gc_life>>>>,
    pub anon_classes: Vec<Arc<RuntimeClass<'gc_life>>>,
    pub anon_class_live_object_ldc_pool: Vec<GcManagedObject<'gc_life>>,
    pub(crate) class_class: Arc<RuntimeClass<'gc_life>>,
    class_loaders: BiMap<LoaderIndex, ByAddress<GcManagedObject<'gc_life>>>,
    pub protection_domains: BiMap<ByAddress<Arc<RuntimeClass<'gc_life>>>, ByAddress<GcManagedObject<'gc_life>>>,
}

impl<'gc_life> Classes<'gc_life> {
    pub fn get_loaded_classes(&self) -> Vec<(LoaderName, CPDType)> {
        self.loaded_classes_by_type.iter().flat_map(|(l, rc)| rc.keys().map(move |ptype| (*l, ptype.clone()))).collect_vec()
    }

    pub fn is_loaded(&self, ptype: &CPDType) -> Option<Arc<RuntimeClass<'gc_life>>> {
        self.initiating_loaders.get(&ptype)?.1.clone().into()
    }

    pub fn get_initiating_loader(&self, class_: &Arc<RuntimeClass<'gc_life>>) -> LoaderName {
        let (res, actual_class) = self.initiating_loaders.get(&class_.cpdtype()).unwrap();
        if !Arc::ptr_eq(class_, actual_class) {
            dbg!(class_.cpdtype().unwrap_class_type());
            dbg!(actual_class.cpdtype().unwrap_class_type());
            dbg!(res);
            // panic!()//todo
        }
        *res
    }

    pub fn get_class_obj(&self, ptypeview: CPDType, loader: Option<LoaderName>) -> Option<GcManagedObject<'gc_life>> {
        if loader.is_some() {
            todo!()
        }
        let runtime_class = self.initiating_loaders.get(&ptypeview)?.1.clone();
        let obj = self.class_object_pool.get_by_right(&ByAddress(runtime_class.clone())).unwrap().0.clone();
        Some(obj)
    }

    pub fn get_class_obj_from_runtime_class(&self, runtime_class: Arc<RuntimeClass<'gc_life>>) -> GcManagedObject<'gc_life> {
        self.class_object_pool.get_by_right(&ByAddress(runtime_class.clone())).unwrap().0.clone()
    }

    pub fn convert_runtime_class_class_id(&self, id: RuntimeClassClassId) -> &RuntimeClassClass {
        todo!()
    }

    pub fn classes_gc_roots<'specific_gc_life>(&'specific_gc_life self) -> impl Iterator<Item=GcManagedObject<'gc_life>> + 'specific_gc_life {
        self.class_object_pool
            .left_values()
            .map(|by_address| by_address.0.clone())
            .chain(self.anon_class_live_object_ldc_pool.iter().cloned())
            .chain(self.class_loaders.right_values().map(|by_address| by_address.0.clone()))
            .chain(self.protection_domains.right_values().map(|by_address| by_address.0.clone()))
            .chain(self.initiating_loaders.values().flat_map(|(_loader, class)| class.try_unwrap_class_class()).flat_map(|class| class.static_vars.read().unwrap().values().flat_map(|jv| jv.try_unwrap_object()).flatten().collect_vec()))
    }

    pub fn loaded_classes_by_type(&self, loader: &LoaderName, type_: &CPDType) -> &Arc<RuntimeClass<'gc_life>> {
        self.loaded_classes_by_type.get(loader).unwrap().get(type_).unwrap()
    }

    pub fn object_to_runtime_class(&self, object: GcManagedObject<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
        self.class_object_pool.get_by_left(&ByAddressGcManagedObject(object)).unwrap().0.clone()
    }

    pub fn lookup_class_loader(&self, loader_name: LoaderIndex) -> &GcManagedObject<'gc_life> {
        &self.class_loaders.get_by_left(&loader_name).unwrap().0
    }

    pub fn lookup_or_add_classloader(&mut self, obj: GcManagedObject<'gc_life>) -> LoaderName {
        let mut loaders_guard = &mut self.class_loaders;
        let loader_index_lookup = loaders_guard.get_by_right(&ByAddress(obj.clone()));
        LoaderName::UserDefinedLoader(match loader_index_lookup {
            Some(x) => *x,
            None => {
                let new_loader_id = LoaderIndex(loaders_guard.len());
                assert!(!loaders_guard.contains_left(&new_loader_id));
                loaders_guard.insert(new_loader_id, ByAddress(obj));
                //todo this whole mess needs a register class loader function which addes to approprate classes data structure
                new_loader_id
            }
        })
    }

    pub fn lookup_live_object_pool(&self, idx: &LiveObjectIndex) -> &GcManagedObject<'gc_life> {
        &self.anon_class_live_object_ldc_pool[idx.0]
    }

    pub fn get_loader_and_runtime_class(&self, cpdtype: &CPDType) -> Option<(LoaderName, Arc<RuntimeClass<'gc_life>>)> {
        Some(self.initiating_loaders.get(cpdtype)?.clone())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ClassStatus {
    UNPREPARED,
    PREPARED,
    INITIALIZING,
    INITIALIZED,
}

impl<'gc_life> JVMState<'gc_life> {
    pub fn new(jvm_options: JVMOptions, scope: Scope<'gc_life>, gc: &'gc_life GC<'gc_life>, string_pool:CompressedClassfileStringPool) -> (Vec<String>, Self) {
        let JVMOptions {
            main_class_name,
            classpath,
            args,
            shared_libs,
            enable_tracing,
            enable_jvmti,
            properties,
            unittest_mode,
            store_generated_classes,
            debug_print_exceptions,
            assertions_enabled,
        } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let classpath_arc = Arc::new(classpath);

        let tracing = if enable_tracing { TracingSettings::new() } else { TracingSettings::disabled() };

        let jvmti_state = if enable_jvmti {
            JVMTIState {
                built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(&libjdwp)),
                break_points: RwLock::new(HashMap::new()),
                tags: RwLock::new(HashMap::new()),
            }
                .into()
        } else {
            None
        };
        let thread_state = ThreadState::new(scope);
        let classes = JVMState::init_classes(&string_pool, &classpath_arc);
        let main_class_name = CompressedClassName(string_pool.add_name(main_class_name.get_referred_name().clone(), true));

        let jvm = Self {
            jit_state: &JITSTATE,
            config: JVMConfig {
                store_generated_classes,
                debug_print_exceptions,
                assertions_enabled,
                compiled_mode_active: true,
                tracing,
                main_class_name,
            },
            properties,
            native_libaries: NativeLibraries::new(libjava),
            string_pool,
            string_internment: RwLock::new(StringInternment { strings: HashMap::new() }),
            start_instant: Instant::now(),
            classes,
            gc,
            classpath: classpath_arc,
            thread_state,
            method_table: RwLock::new(MethodTable::new()),
            field_table: RwLock::new(FieldTable::new()),
            wtf8_pool: Wtf8Pool::new(),
            cpdtype_table: RwLock::new(CPDTypeTable::new()),
            opaque_ids: RwLock::new(OpaqueIDs::new()),
            native: Native {
                jvmti_state,
                invoke_interface: RwLock::new(None),
                native_interface_allocations: NativeAllocator { allocations: RwLock::new(HashMap::new()) },
            },
            live: AtomicBool::new(false),
            resolved_method_handles: RwLock::new(HashMap::new()),
            include_name_field: AtomicBool::new(false),
            stacktraces_by_throwable: RwLock::new(HashMap::new()),
            function_frame_type_data: Default::default(),
            java_vm_state: JavaVMStateWrapper::new(),
            java_function_frame_data: Default::default()
        };
        (args, jvm)
    }

    pub fn sink_function_verification_date(&self, verification_types: &HashMap<u16, HashMap<ByteCodeOffset, Frame>>, rc: Arc<RuntimeClass<'gc_life>>) {
        let mut method_table = self.method_table.write().unwrap();
        for (method_i, verification_types) in verification_types {
            let method_id = method_table.get_method_id(rc.clone(), *method_i);
            self.function_frame_type_data.write().unwrap().insert(method_id, verification_types.clone());
        }
    }

    pub fn verify_class_and_object(&self, object_runtime_class: Arc<RuntimeClass<'gc_life>>, class_runtime_class: Arc<RuntimeClass<'gc_life>>) {
        let mut context = VerifierContext {
            live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
            classfile_getter: Arc::new(DefaultClassfileGetter { jvm: self }) as Arc<dyn ClassFileGetter>,
            string_pool: &self.string_pool,
            class_view_cache: Mutex::new(Default::default()),
            current_loader: LoaderName::BootstrapLoader,
            verification_types: HashMap::new(),
            debug: false,
        };
        let lookup = self.classpath.lookup(&CClassName::object(), &self.string_pool).expect("Can not find Object class");
        verify(&mut context, CClassName::object(), LoaderName::BootstrapLoader).expect("Object doesn't verify");
        self.sink_function_verification_date(&context.verification_types, object_runtime_class);
        context.verification_types.clear();
        let lookup = self.classpath.lookup(&CClassName::class(), &self.string_pool).expect("Can not find Class class");
        verify(&mut context, CClassName::class(), LoaderName::BootstrapLoader).expect("Class doesn't verify");
        self.sink_function_verification_date(&context.verification_types, class_runtime_class);
    }

    pub fn add_class_class_class_object(&'gc_life self) {
        let mut classes = self.classes.write().unwrap();
        //todo desketchify this
        let mut fields: HashMap<String, JavaValue<'gc_life>, RandomState> = Default::default();
        fields.insert("name".to_string(), JavaValue::null());
        fields.insert("classLoader".to_string(), JavaValue::null());
        const MAX_LOCAL_VARS: i32 = 100;
        let mut fields_vec = (0..MAX_LOCAL_VARS).map(|_| NativeJavaValue { object: null_mut() }).collect_vec();
        let class_object = self.allocate_object(Object::Object(NormalObject {
            objinfo: ObjectFieldsAndClass { fields: RwLock::new(fields_vec.as_mut_slice()), class_pointer: classes.class_class.clone() },
            obj_ptr: None,
        }));
        let runtime_class = ByAddress(classes.class_class.clone());
        classes.class_object_pool.insert(ByAddressGcManagedObject(class_object), runtime_class);
        let runtime_class = classes.class_class.clone();
        classes.loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::class().into(), runtime_class);
    }

    fn init_classes(pool: &CompressedClassfileStringPool, classpath_arc: &Arc<Classpath>) -> RwLock<Classes<'gc_life>> {
        //todo turn this into a ::new
        let field_numbers = JVMState::get_class_field_numbers();
        let class_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::class(), pool).unwrap(), pool));
        let static_vars = Default::default();
        let parent = None;
        let interfaces = vec![];
        let status = ClassStatus::UNPREPARED.into();
        let recursive_num_fields = field_numbers.len();
        let class_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new(class_view, field_numbers, recursive_num_fields, static_vars, parent, interfaces, status)));
        let mut initiating_loaders: HashMap<CPDType, (LoaderName, Arc<RuntimeClass<'gc_life>>), RandomState> = Default::default();
        initiating_loaders.insert(CClassName::class().into(), (LoaderName::BootstrapLoader, class_class.clone()));
        let class_object_pool: BiMap<ByAddressGcManagedObject<'gc_life>, ByAddress<Arc<RuntimeClass<'gc_life>>>> = Default::default();
        let classes = RwLock::new(Classes {
            loaded_classes_by_type: Default::default(),
            initiating_loaders,
            class_object_pool,
            anon_classes: Default::default(),
            anon_class_live_object_ldc_pool: Vec::new(),
            class_class,
            class_loaders: Default::default(),
            protection_domains: Default::default(),
        });
        classes
    }

    pub fn get_class_field_numbers() -> HashMap<FieldName, (usize, CPDType)> {
        let class_class_fields = vec![
            (FieldName::field_cachedConstructor(), CClassName::constructor().into()),
            (FieldName::field_newInstanceCallerCache(), CClassName::class().into()),
            (FieldName::field_name(), CClassName::string().into()),
            (FieldName::field_classLoader(), CClassName::classloader().into()),
            (FieldName::field_reflectionData(), CPDType::object()),
            (FieldName::field_classRedefinedCount(), CPDType::IntType),
            (FieldName::field_genericInfo(), CPDType::object()),
            (FieldName::field_enumConstants(), CPDType::array(CPDType::object())),
            (FieldName::field_enumConstantDirectory(), CPDType::object()),
            (FieldName::field_annotationData(), CPDType::object()),
            (FieldName::field_annotationType(), CPDType::object()),
            (FieldName::field_classValueMap(), CPDType::object()),
        ];
        let field_numbers = HashMap::from_iter(class_class_fields.iter().cloned().sorted_by_key(|(name, _)| name.clone()).enumerate().map(|(_1, (_2_name, _2_type))| ((_2_name.clone()), (_1, _2_type.clone()))).collect_vec().into_iter());
        field_numbers
    }

    pub unsafe fn get_int_state<'l, 'interpreter_guard>(&self) -> &'interpreter_guard mut InterpreterStateGuard<'l,'interpreter_guard> {
        assert!(self.thread_state.int_state_guard_valid.with(|refcell| { *refcell.borrow() }));
        let ptr = self.thread_state.int_state_guard.with(|refcell| *refcell.borrow().as_ref().unwrap());
        let res = transmute::<&mut InterpreterStateGuard<'static,'static>, &mut InterpreterStateGuard<'l,'interpreter_guard>>(ptr.as_mut().unwrap()); //todo make this less sketch maybe
        assert!(res.registered());
        res
    }

    pub fn get_loader_obj(&self, loader: LoaderName) -> Option<ClassLoader<'gc_life>> {
        match loader {
            LoaderName::UserDefinedLoader(loader_idx) => {
                let classes_guard = self.classes.read().unwrap();
                let jvalue = JavaValue::Object(classes_guard.class_loaders.get_by_left(&loader_idx).unwrap().clone().0.into());
                Some(jvalue.cast_class_loader())
            }
            LoaderName::BootstrapLoader => None,
        }
    }

    pub fn allocate_object(&'gc_life self, object: Object<'gc_life, 'l>) -> GcManagedObject<'gc_life> {
        self.gc.allocate_object(self, object)
    }

    pub fn jvmti_state(&self) -> Option<&JVMTIState> {
        self.native.jvmti_state.as_ref()
    }

    pub fn max_locals_by_method_id(&self, method_id: MethodId) -> u16 {
        let (rc, method_i) = self.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.code_attribute().unwrap().max_locals
    }

    pub fn is_native_by_method_id(&self, method_id: MethodId) -> bool {
        let (rc, method_i) = self.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.is_native()
    }

    pub fn num_args_by_method_id(&self, method_id: MethodId) -> u16 {
        let (rc, method_i) = self.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.desc().arg_types.len() as u16
    }
}

pub struct JVMTIState {
    pub built_in_jdwp: Arc<SharedLibJVMTI>,
    pub break_points: RwLock<HashMap<MethodId, HashSet<ByteCodeOffset>>>,
    pub tags: RwLock<HashMap<jobject, jlong>>,
}

struct LivePoolGetterImpl<'gc_life> {
    jvm: &'gc_life JVMState<'gc_life>,
    // anon_class_live_object_ldc_pool: Arc<RwLock<Vec<GcManagedObject<'gc_life>>>>,
}

#[derive(Debug)]
pub struct NativeLib {
    pub library: Library,
}

#[derive(Debug)]
pub struct NativeLibraries<'gc_life> {
    pub libjava_path: OsString,
    pub native_libs: RwLock<HashMap<String, NativeLib>>,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass<'gc_life>>>, RwLock<HashMap<u16, unsafe extern "C" fn()>>>>,
}

impl<'gc_life> NativeLibraries<'gc_life> {
    pub unsafe fn load(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, path: &OsString, name: String) {
        let onload_fn_ptr = self.get_onload_ptr_and_add(path, name);
        let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm, int_state);
        onload_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterface_, null_mut());
        //todo check return res
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
    fn elem_type(&self, idx: LiveObjectIndex) -> CPRefType {
        let classes_guard = self.jvm.classes.read().unwrap();
        let object = &classes_guard.anon_class_live_object_ldc_pool[idx.0];
        JavaValue::Object(object.clone().into()).to_type().unwrap_ref_type().clone();
        todo!()
    }
}

impl<'gc_life> JVMState<'gc_life> {
    pub fn vm_live(&self) -> bool {
        self.live.load(Ordering::SeqCst)
    }

    pub fn get_live_object_pool_getter(&'gc_life self) -> Arc<dyn LivePoolGetter + 'gc_life> {
        Arc::new(LivePoolGetterImpl { jvm: self })
    }

    pub fn get_class_getter(&'l self, loader: LoaderName) -> Arc<dyn ClassFileGetter + 'l> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Arc::new(BootstrapLoaderClassGetter { jvm: self })
    }
}

pub struct BootstrapLoaderClassGetter<'vm_life, 'l> {
    jvm: &'l JVMState<'vm_life>,
}

impl ClassFileGetter for BootstrapLoaderClassGetter<'_, '_> {
    fn get_classfile(&self, loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Ok(Arc::new(ClassBackedView::from(self.jvm.classpath.lookup(&class, &self.jvm.string_pool)?, &self.jvm.string_pool)))
    }
}

pub struct StringInternment<'gc_life> {
    pub strings: HashMap<Vec<u16>, GcManagedObject<'gc_life>>,
}