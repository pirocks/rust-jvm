use std::collections::{HashMap, HashSet};
use std::collections::hash_map::RandomState;
use std::ffi::{c_void, OsString};
use std::iter;
use std::iter::{FromIterator};
use std::mem::transmute;
use std::ops::Deref;
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use bimap::BiMap;
use by_address::ByAddress;
use crossbeam::thread::Scope;
use itertools::Itertools;
use libloading::{Error, Library, Symbol};
use libloading::os::unix::{RTLD_GLOBAL, RTLD_LAZY};

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use interface_vtable::lookup_cache::InvokeInterfaceLookupCache;
use jvmti_jni_bindings::{JavaVM, jint, jlong, JNIInvokeInterface_, jobject};
use method_table::MethodTable;
use perf_metrics::PerfMetrics;
use runtime_class_stuff::{ClassStatus, RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use runtime_class_stuff::method_numbers::{MethodNumber, MethodNumberMappings};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::{CompressedClassfileStringPool, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::LiveObjectIndex;
use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName, FieldName};
use rust_jvm_common::cpdtype_table::CPDTypeTable;
use rust_jvm_common::loading::{ClassLoadingError, LivePoolGetter, LoaderIndex, LoaderName};
use rust_jvm_common::method_shape::{MethodShape, MethodShapeIDs, ShapeOrderWrapperOwned};
use rust_jvm_common::opaque_id_table::OpaqueIDs;
use rust_jvm_common::vtype::VType;
use sketch_jvm_version_of_utf8::wtf8_pool::Wtf8Pool;
use verification::{ClassFileGetter, OperandStack, VerifierContext, verify};
use verification::verifier::{Frame};

use crate::class_loading::{DefaultClassfileGetter, DefaultLivePoolGetter, get_static_var_types};
use crate::field_table::FieldTable;
use crate::interpreter_state::InterpreterStateGuard;
use crate::invoke_interface::get_invoke_interface;
use stage0::compiler::{RecompileConditions};
use crate::ir_to_java_layer::java_vm_state::JavaVMStateWrapper;
use crate::java::lang::class_loader::ClassLoader;
use crate::java::lang::stack_trace_element::StackTraceElement;
use crate::java_values::{ByAddressAllocatedObject, default_value, GC, JavaValue};
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::known_type_to_address_mappings::KnownAddresses;
use crate::loading::Classpath;
use crate::native_allocation::NativeAllocator;
use crate::options::{ExitTracingOptions, InstructionTraceOptions, JVMOptions, SharedLibraryPaths};
use runtime_class_stuff::RuntimeClass;
use stage0::compiler_common::frame_data::{FunctionFrameData, SunkVerifierFrames};
use stage0::compiler_common::JavaCompilerMethodAndFrameData;
use vtable::lookup_cache::InvokeVirtualLookupCache;
use vtable::VTables;
use crate::{AllocatedHandle, JavaValueCommon, UnAllocatedObject};
use crate::function_instruction_count::FunctionInstructionExecutionCount;
use crate::new_java_values::allocated_objects::{AllocatedNormalObjectHandle, AllocatedObjectHandleByAddress};
use crate::new_java_values::unallocated_objects::UnAllocatedObjectObject;
use crate::string_exit_cache::StringExitCache;
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

pub struct JVMState<'gc> {
    pub config: JVMConfig,
    pub java_vm_state: JavaVMStateWrapper<'gc>,
    pub gc: &'gc GC<'gc>,
    pub native_libaries: NativeLibraries<'gc>,
    pub properties: Vec<String>,
    pub string_pool: CompressedClassfileStringPool,
    pub string_internment: RwLock<StringInternment<'gc>>,
    pub start_instant: Instant,
    pub classes: RwLock<Classes<'gc>>,
    pub classpath: Arc<Classpath>,
    pub thread_state: ThreadState<'gc>,
    pub method_table: RwLock<MethodTable<'gc>>,
    pub field_table: RwLock<FieldTable<'gc>>,
    pub wtf8_pool: Wtf8Pool,
    pub cpdtype_table: RwLock<CPDTypeTable>,
    pub opaque_ids: RwLock<OpaqueIDs>,
    pub native: Native,
    pub live: AtomicBool,
    pub resolved_method_handles: RwLock<HashMap<ByAddressAllocatedObject<'gc>, MethodId>>,
    pub include_name_field: AtomicBool,
    pub stacktraces_by_throwable: RwLock<HashMap<AllocatedObjectHandleByAddress<'gc>, Vec<StackTraceElement<'gc>>>>,
    // pub function_frame_type_data_no_tops: RwLock<HashMap<MethodId, HashMap<ByteCodeOffset, SunkVerifierFrames>>>,
    // pub function_frame_type_data_with_tops: RwLock<HashMap<MethodId, HashMap<ByteCodeOffset, SunkVerifierFrames>>>,
    pub function_frame_type_data: RwLock<FunctionFrameData>,
    pub java_function_frame_data: RwLock<HashMap<MethodId, JavaCompilerMethodAndFrameData>>,
    pub object_monitors: RwLock<HashMap<*const c_void, Arc<Monitor2>>>,
    pub method_shapes: MethodShapeIDs,
    pub instruction_trace_options: InstructionTraceOptions,
    pub exit_trace_options: ExitTracingOptions,
    pub checkcast_debug_assertions: bool,
    pub perf_metrics: PerfMetrics,
    pub recompilation_conditions: RwLock<RecompileConditions>,
    pub known_addresses: KnownAddresses,
    pub vtable: Mutex<VTables<'gc>>,
    pub invoke_virtual_lookup_cache: RwLock<InvokeVirtualLookupCache<'gc>>,
    pub invoke_interface_lookup_cache: RwLock<InvokeInterfaceLookupCache<'gc>>,
    pub string_exit_cache: RwLock<StringExitCache<'gc>>,
    pub function_execution_count: FunctionInstructionExecutionCount
}


pub struct Classes<'gc> {
    //todo needs to be used for all instances of getClass
    pub loaded_classes_by_type: HashMap<LoaderName, HashMap<CPDType, Arc<RuntimeClass<'gc>>>>,
    pub initiating_loaders: HashMap<CPDType, (LoaderName, Arc<RuntimeClass<'gc>>)>,
    pub(crate) class_object_pool: BiMap<ByAddressAllocatedObject<'gc>, ByAddress<Arc<RuntimeClass<'gc>>>>,
    pub anon_classes: Vec<Arc<RuntimeClass<'gc>>>,
    pub anon_class_live_object_ldc_pool: Vec<&'gc AllocatedNormalObjectHandle<'gc>>,
    pub(crate) class_class: Arc<RuntimeClass<'gc>>,
    class_loaders: BiMap<LoaderIndex, ByAddressAllocatedObject<'gc>>,
    pub protection_domains: BiMap<ByAddress<Arc<RuntimeClass<'gc>>>, ByAddressAllocatedObject<'gc>>,
}

impl<'gc> Classes<'gc> {
    pub fn get_loaded_classes(&self) -> Vec<(LoaderName, CPDType)> {
        self.loaded_classes_by_type.iter().flat_map(|(l, rc)| rc.keys().map(move |ptype| (*l, ptype.clone()))).collect_vec()
    }

    pub fn is_loaded(&self, ptype: &CPDType) -> Option<Arc<RuntimeClass<'gc>>> {
        self.initiating_loaders.get(&ptype)?.1.clone().into()
    }

    pub fn is_inited_or_initing(&self, ptype: &CPDType) -> Option<Arc<RuntimeClass<'gc>>> {
        let rc = self.initiating_loaders.get(&ptype)?.1.clone();
        Some(match rc.status() {
            ClassStatus::UNPREPARED |
            ClassStatus::PREPARED => {
                return None;
            }
            ClassStatus::INITIALIZING |
            ClassStatus::INITIALIZED => {
                rc
            }
        })
    }

    pub fn get_initiating_loader(&self, class_: &Arc<RuntimeClass<'gc>>) -> LoaderName {
        let (res, actual_class) = self.initiating_loaders.get(&class_.cpdtype()).unwrap();
        if !Arc::ptr_eq(class_, actual_class) {
            // dbg!(class_.cpdtype().unwrap_class_type());
            // dbg!(actual_class.cpdtype().unwrap_class_type());
            // dbg!(res);
            // panic!()//todo
        }
        *res
    }

    pub fn get_class_obj(&self, ptypeview: CPDType, loader: Option<LoaderName>) -> Option<AllocatedNormalObjectHandle<'gc>> {
        if loader.is_some() {
            todo!()
        }
        let runtime_class = self.initiating_loaders.get(&ptypeview)?.1.clone();
        let obj = self.class_object_pool.get_by_right(&ByAddress(runtime_class.clone())).unwrap().clone().owned_inner();
        Some(obj)
    }

    pub fn get_class_obj_from_runtime_class(&self, runtime_class: Arc<RuntimeClass<'gc>>) -> AllocatedNormalObjectHandle<'gc> {
        self.class_object_pool.get_by_right(&ByAddress(runtime_class.clone())).unwrap().clone().owned_inner()
    }

    pub fn classes_gc_roots<'specific_gc_life>(&'specific_gc_life self) -> impl Iterator<Item=&'specific_gc_life AllocatedNormalObjectHandle<'gc>> + 'specific_gc_life {
        /* self.class_object_pool
             .left_values()
             .map(|by_address| by_address.clone().owned_inner())
             .chain(todo!()/*self.anon_class_live_object_ldc_pool.iter().cloned().cloned()*/)
             .chain(self.class_loaders.right_values().map(|by_address| by_address.clone().owned_inner()))
             .chain(self.protection_domains.right_values().map(|by_address| by_address.clone().owned_inner()))
             .chain(self.initiating_loaders.values()
                 .flat_map(|(_loader, class): &(_, Arc<RuntimeClass<'gc>>)| class.try_unwrap_class_class())
                 .flat_map(|class: &RuntimeClassClass<'gc>| {
                     let guard = class.static_vars.read().unwrap();
                     /*guard.values().map(|jv| {
                         todo!()
                     })*/
                     once(todo!())
                 })
             )*/
        iter::once(todo!())
    }

    pub fn loaded_classes_by_type(&self, loader: &LoaderName, type_: &CPDType) -> &Arc<RuntimeClass<'gc>> {
        self.loaded_classes_by_type.get(loader).unwrap().get(type_).unwrap()
    }

    pub fn object_to_runtime_class<'any>(&self, object: &'any AllocatedNormalObjectHandle<'gc>) -> Arc<RuntimeClass<'gc>> {
        self.class_object_pool.get_by_left(&ByAddressAllocatedObject::LookupOnly(object.raw_ptr_usize())).unwrap().0.clone()
    }

    pub fn lookup_class_loader(&self, loader_name: LoaderIndex) -> AllocatedNormalObjectHandle<'gc> {
        self.class_loaders.get_by_left(&loader_name).unwrap().clone().owned_inner()
    }

    pub fn lookup_or_add_classloader(&mut self, obj: AllocatedNormalObjectHandle<'gc>) -> LoaderName {
        let loaders_guard = &mut self.class_loaders;
        let loader_index_lookup = loaders_guard.get_by_right(&ByAddressAllocatedObject::LookupOnly(obj.raw_ptr_usize()));
        LoaderName::UserDefinedLoader(match loader_index_lookup {
            Some(x) => *x,
            None => {
                let new_loader_id = LoaderIndex(loaders_guard.len() as u32);
                assert!(!loaders_guard.contains_left(&new_loader_id));
                loaders_guard.insert(new_loader_id, ByAddressAllocatedObject::Owned(obj));
                //todo this whole mess needs a register class loader function which addes to approprate classes data structure
                new_loader_id
            }
        })
    }

    pub fn lookup_live_object_pool(&self, idx: &LiveObjectIndex) -> &'gc AllocatedNormalObjectHandle<'gc> {
        &self.anon_class_live_object_ldc_pool[idx.0]
    }

    pub fn get_loader_and_runtime_class(&self, cpdtype: &CPDType) -> Option<(LoaderName, Arc<RuntimeClass<'gc>>)> {
        Some(self.initiating_loaders.get(cpdtype)?.clone())
    }
}


impl<'gc> JVMState<'gc> {
    pub fn new(jvm_options: JVMOptions, scope: Scope<'gc>, gc: &'gc GC<'gc>, string_pool: CompressedClassfileStringPool) -> (Vec<String>, Self) {
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
            instruction_trace_options,
            exit_trace_options,
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
            java_vm_state: JavaVMStateWrapper::new(),
            java_function_frame_data: Default::default(),
            object_monitors: Default::default(),
            method_shapes: MethodShapeIDs::new(),
            instruction_trace_options,
            exit_trace_options,
            checkcast_debug_assertions: false,
            perf_metrics: PerfMetrics::new(),
            recompilation_conditions: RwLock::new(RecompileConditions::new()),
            known_addresses: KnownAddresses::new(),
            vtable: Mutex::new(VTables::new()),
            invoke_virtual_lookup_cache: RwLock::new(InvokeVirtualLookupCache::new()),
            invoke_interface_lookup_cache: RwLock::new(InvokeInterfaceLookupCache::new()),
            string_exit_cache: RwLock::new(StringExitCache::new()),
            function_frame_type_data: RwLock::new(FunctionFrameData{
                no_tops: Default::default(),
                tops: Default::default()
            }),
            function_execution_count: FunctionInstructionExecutionCount::new()
        };
        (args, jvm)
    }

    pub fn sink_function_verification_date(&self, verification_types: &HashMap<u16, HashMap<ByteCodeOffset, Frame>>, rc: Arc<RuntimeClass<'gc>>) {
        let mut method_table = self.method_table.write().unwrap();
        let view = rc.view();
        for (method_i, verification_types) in verification_types {
            let method_id = method_table.get_method_id(rc.clone(), *method_i);
            let method_view = view.method_view_i(*method_i);
            let code = method_view.code_attribute().unwrap();
            let verification_types_without_top: HashMap<ByteCodeOffset, SunkVerifierFrames> = verification_types.iter().map(|(offset, Frame { locals, stack_map, flag_this_uninit })| {
                let stack_without_top = stack_map.data.iter().filter(|type_| !matches!(type_,VType::TopType)).cloned().collect();
                let locals_without_top = locals.iter().filter(|type_| !matches!(type_,VType::TopType)).cloned().collect();
                (*offset, SunkVerifierFrames::FullFrame(Frame {
                    locals: Rc::new(locals_without_top),
                    stack_map: OperandStack { data: stack_without_top },
                    flag_this_uninit: *flag_this_uninit,
                }))
            }).collect();
            for (offset, _) in code.instructions.iter() {
                assert!(verification_types_without_top.contains_key(offset));
            }
            self.function_frame_type_data.write().unwrap().no_tops.insert(method_id, verification_types_without_top);
            self.function_frame_type_data.write().unwrap().tops.insert(method_id, verification_types.iter().map(|(offset, frame)| {
                (*offset, SunkVerifierFrames::FullFrame(frame.clone()))
            }).collect());
        }
    }

    pub fn verify_class_and_object(&self, object_runtime_class: Arc<RuntimeClass<'gc>>, class_runtime_class: Arc<RuntimeClass<'gc>>) {
        let mut context = VerifierContext {
            live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
            classfile_getter: Arc::new(DefaultClassfileGetter { jvm: self }) as Arc<dyn ClassFileGetter>,
            string_pool: &self.string_pool,
            class_view_cache: Mutex::new(Default::default()),
            current_loader: LoaderName::BootstrapLoader,
            verification_types: HashMap::new(),
            debug: false,
            perf_metrics: &self.perf_metrics,
            permissive_types_workaround: false,
        };
        let lookup = self.classpath.lookup(&CClassName::object(), &self.string_pool).expect("Can not find Object class");
        verify(&mut context, CClassName::object(), LoaderName::BootstrapLoader).expect("Object doesn't verify");
        self.sink_function_verification_date(&context.verification_types, object_runtime_class);
        context.verification_types.clear();
        let lookup = self.classpath.lookup(&CClassName::class(), &self.string_pool).expect("Can not find Class class");
        verify(&mut context, CClassName::class(), LoaderName::BootstrapLoader).expect("Class doesn't verify");
        self.sink_function_verification_date(&context.verification_types, class_runtime_class);
    }

    pub fn add_class_class_class_object(&'gc self) {
        let mut classes = self.classes.write().unwrap();
        //todo desketchify this
        let recursive_num_fields = classes.class_class.unwrap_class_class().recursive_num_fields;
        let field_numbers_reverse = &classes.class_class.unwrap_class_class().field_numbers_reverse;
        let fields_map_owned = (0..recursive_num_fields).map(|i| {
            let field_number = FieldNumber(i as u32);
            let (field_name, cpd_type) = field_numbers_reverse.get(&field_number).unwrap();
            let default_jv = default_value(&cpd_type);
            (field_number, default_jv)
        }).collect::<Vec<_>>();
        let fields = fields_map_owned.iter().map(|(field_number, handle)| (*field_number, handle.as_njv())).collect();

        let class_object_handle = self.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject { object_rc: classes.class_class.clone(), fields }));
        let class_object = self.gc.handle_lives_for_gc_life(class_object_handle.unwrap_normal_object());
        let runtime_class = ByAddress(classes.class_class.clone());
        classes.class_object_pool.insert(ByAddressAllocatedObject::Owned(class_object.duplicate_discouraged()), runtime_class);
        let runtime_class = classes.class_class.clone();
        classes.loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::class().into(), runtime_class);
    }

    fn init_classes(pool: &CompressedClassfileStringPool, classpath_arc: &Arc<Classpath>) -> RwLock<Classes<'gc>> {
        //todo turn this into a ::new
        let field_numbers = JVMState::get_class_class_field_numbers();
        let class_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::class(), pool).unwrap(), pool));
        let static_vars = Default::default();
        let interfaces = vec![];
        let status = ClassStatus::UNPREPARED.into();
        let recursive_num_fields = field_numbers.len() as u32;
        let object_class_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::object(), pool).unwrap(), pool));
        let (object_recursive_num_methods,object_method_numbers) = JVMState::get_class_class_or_object_class_method_numbers(pool,object_class_view.deref(), None);
        let (class_recursive_num_methods, class_class_method_numbers) = JVMState::get_class_class_or_object_class_method_numbers(pool,class_view.deref(), Some(object_class_view.deref()));
        for (object_method_shape, object_method_number) in object_method_numbers.iter(){
            let class_method_number = class_class_method_numbers.get(object_method_shape).unwrap();
            assert_eq!(object_method_number, class_method_number);
        }
        let object_class_static_var_types = get_static_var_types(object_class_view.deref());
        let temp_object_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new(
            object_class_view,
            HashMap::new(),
            object_method_numbers,
            0,
            object_recursive_num_methods as u32,
            RwLock::new(HashMap::new()),
            None,
            vec![],
            RwLock::new(ClassStatus::INITIALIZED),
            object_class_static_var_types,
        )));
        let class_class_static_var_types = get_static_var_types(class_view.deref());
        let class_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new(
            class_view,
            field_numbers,
            class_class_method_numbers,
            recursive_num_fields,
            class_recursive_num_methods,
            static_vars,
            Some(temp_object_class),
            interfaces,
            status,
            class_class_static_var_types,
        )));
        let mut initiating_loaders: HashMap<CPDType, (LoaderName, Arc<RuntimeClass<'gc>>), RandomState> = Default::default();
        initiating_loaders.insert(CClassName::class().into(), (LoaderName::BootstrapLoader, class_class.clone()));
        let class_object_pool: BiMap<ByAddressAllocatedObject<'gc>, ByAddress<Arc<RuntimeClass<'gc>>>> = Default::default();
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

    pub fn get_class_class_field_numbers() -> HashMap<FieldName, (FieldNumber, CPDType)> {
        //todo this use the class view instead
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
        let field_numbers = HashMap::from_iter(class_class_fields.iter().cloned().sorted_by_key(|(name, _)| name.clone()).enumerate().map(|(_1, (_2_name, _2_type))| ((_2_name.clone()), (FieldNumber(_1 as u32), _2_type.clone()))).collect_vec().into_iter());
        field_numbers
    }

    pub fn get_class_class_or_object_class_method_numbers(pool: &CompressedClassfileStringPool, class_class_view: &dyn ClassView, parent: Option<&dyn ClassView>) -> (u32,HashMap<MethodShape, MethodNumber>) {
        let mut method_number_mappings = MethodNumberMappings::new();

        if let Some(parent) = parent{
            for method_shape in parent.methods()
                .filter(|method| !method.is_static())
                .map(|method|ShapeOrderWrapperOwned(method.method_shape())).sorted(){
                method_number_mappings.sink_method(method_shape.0);
            }
        }
        for method_shape in class_class_view.methods()
            .filter(|method| !method.is_static())
            .map(|method| ShapeOrderWrapperOwned(method.method_shape())).sorted() {
            method_number_mappings.sink_method(method_shape.0);
        }

        let reverse_mapping = method_number_mappings.mapping.iter().map(|(_1, _2)| (_2.clone(), _1.clone())).collect::<HashMap<MethodNumber, MethodShape>>();

        (method_number_mappings.current_method_number, method_number_mappings.mapping)
    }

    pub unsafe fn get_int_state<'l, 'interpreter_guard>(&self) -> &'interpreter_guard mut InterpreterStateGuard<'l, 'interpreter_guard> {
        assert!(self.thread_state.int_state_guard_valid.with(|elem| elem.borrow().clone()));
        let ptr = self.thread_state.int_state_guard.with(|elem| elem.borrow().clone().unwrap());
        let res = transmute::<&mut InterpreterStateGuard<'static, 'static>, &mut InterpreterStateGuard<'l, 'interpreter_guard>>(ptr.as_mut().unwrap()); //todo make this less sketch maybe
        assert!(res.registered());
        assert!(res.thread().thread_status.read().unwrap().alive);
        res
    }

    pub fn get_loader_obj(&self, loader: LoaderName) -> Option<ClassLoader<'gc>> {
        match loader {
            LoaderName::UserDefinedLoader(loader_idx) => {
                let classes_guard = self.classes.read().unwrap();
                let jvalue = JavaValue::Object(classes_guard.class_loaders.get_by_left(&loader_idx).unwrap().clone().owned_inner().to_gc_managed().into());
                Some(jvalue.cast_class_loader())
            }
            LoaderName::BootstrapLoader => None,
        }
    }

    pub fn allocate_object(&'gc self, object: UnAllocatedObject<'gc, '_>) -> AllocatedHandle<'gc> {
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

    pub fn num_local_vars_native(&self, method_id: MethodId) -> u16 {
        let (rc, method_i) = self.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        assert!(method_view.is_native());
        method_view.desc().count_local_vars_needed() as u16 + if method_view.is_static() { 0 } else { 1 }
    }
}

pub struct JVMTIState {
    pub built_in_jdwp: Arc<SharedLibJVMTI>,
    pub break_points: RwLock<HashMap<MethodId, HashSet<ByteCodeOffset>>>,
    pub tags: RwLock<HashMap<jobject, jlong>>,
}

#[allow(unused)]
struct LivePoolGetterImpl<'gc> {
    jvm: &'gc JVMState<'gc>,
}

#[derive(Debug)]
pub struct NativeLib {
    pub library: Library,
}

#[derive(Debug)]
pub struct NativeLibraries<'gc> {
    pub libjava_path: OsString,
    pub native_libs: RwLock<HashMap<String, NativeLib>>,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, RwLock<HashMap<u16, unsafe extern "C" fn()>>>>,
}

impl<'gc> NativeLibraries<'gc> {
    pub unsafe fn load<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, path: &OsString, name: String) {
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

impl<'gc> LivePoolGetter for LivePoolGetterImpl<'gc> {
    fn elem_type(&self, idx: LiveObjectIndex) -> CPRefType {
        // let classes_guard = self.jvm.classes.read().unwrap();
        // let object = &classes_guard.anon_class_live_object_ldc_pool[idx.0];
        // JavaValue::Object(object.clone().to_gc_managed().into()).to_type().unwrap_ref_type().clone();
        todo!()
    }
}

impl<'gc> JVMState<'gc> {
    pub fn vm_live(&self) -> bool {
        self.live.load(Ordering::SeqCst)
    }

    pub fn get_live_object_pool_getter(&'gc self) -> Arc<dyn LivePoolGetter + 'gc> {
        Arc::new(LivePoolGetterImpl { jvm: self })
    }

    pub fn get_class_getter<'l>(&'l self, loader: LoaderName) -> Arc<dyn ClassFileGetter + 'l> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Arc::new(BootstrapLoaderClassGetter { jvm: self })
    }

    pub fn monitor_for(&self, obj_ptr: *const c_void) -> Arc<Monitor2> {
        assert!(obj_ptr != null_mut());
        let mut monitors_guard = self.object_monitors.write().unwrap();
        let next_id = monitors_guard.len();
        let monitor = monitors_guard.entry(obj_ptr).or_insert_with(|| Arc::new(Monitor2::new(next_id)));
        monitor.clone()
    }
}

pub struct BootstrapLoaderClassGetter<'vm, 'l> {
    jvm: &'l JVMState<'vm>,
}

impl ClassFileGetter for BootstrapLoaderClassGetter<'_, '_> {
    fn get_classfile(&self, loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Ok(Arc::new(ClassBackedView::from(self.jvm.classpath.lookup(&class, &self.jvm.string_pool)?, &self.jvm.string_pool)))
    }
}

pub struct StringInternment<'gc> {
    pub strings: HashMap<Vec<u16>, AllocatedHandle<'gc>>,
}