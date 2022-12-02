use std::cell::{OnceCell, RefCell};
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::iter;
use std::ops::Deref;
use std::path::{PathBuf};
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use bimap::BiMap;
use by_address::ByAddress;
use itertools::Itertools;
use libloading::{Error, Library, Symbol};
use libloading::os::unix::{RTLD_GLOBAL, RTLD_LAZY};

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use inheritance_tree::bit_vec_path::BitVecPaths;
use inheritance_tree::class_ids::ClassIDs;
use inheritance_tree::InheritanceTree;
use interface_vtable::ITables;
use interface_vtable::lookup_cache::InvokeInterfaceLookupCache;
use jvmti_jni_bindings::{jint, jlong, JNI_VERSION_1_1, jobject};
use jvmti_jni_bindings::invoke_interface::JNIInvokeInterfaceNamedReservedPointers;
use method_table::interface_table::InterfaceTable;
use method_table::MethodTable;
use perf_metrics::PerfMetrics;
use runtime_class_stuff::ClassStatus;
use runtime_class_stuff::method_numbers::{MethodNumber, MethodNumberMappings};
use runtime_class_stuff::RuntimeClass;
use runtime_class_stuff::static_fields::AllTheStaticFields;
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::code::LiveObjectIndex;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use rust_jvm_common::cpdtype_table::CPDTypeTable;
use rust_jvm_common::loading::{ClassLoadingError, LivePoolGetter, LoaderIndex, LoaderName};
use rust_jvm_common::method_shape::{MethodShape, MethodShapeIDs, ShapeOrderWrapperOwned};
use rust_jvm_common::opaque_id_table::OpaqueIDs;
use rust_jvm_common::vtype::VType;
use sketch_jvm_version_of_utf8::wtf8_pool::Wtf8Pool;
use stage0::compiler::RecompileConditions;
use stage0::compiler_common::frame_data::{FunctionFrameData, SunkVerifierFrames};
use stage0::compiler_common::JavaCompilerMethodAndFrameData;
use verification::{ClassFileGetter, OperandStack, VerifierContext, verify};
use verification::verifier::Frame;
use vtable::lookup_cache::InvokeVirtualLookupCache;
use vtable::VTables;

use crate::{AllocatedHandle, NewAsObjectOrJavaValue, UnAllocatedObject};
use crate::better_java_stack::frames::PushableFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::class_loading::{ClassIntrinsicsData, DefaultClassfileGetter, DefaultLivePoolGetter};
use crate::field_table::FieldTable;
use crate::function_instruction_count::FunctionInstructionExecutionCount;
use crate::ir_to_java_layer::java_vm_state::JavaVMStateWrapper;
use crate::java_values::{ByAddressAllocatedObject, GC, JavaValue};
use crate::leaked_interface_arrays::InterfaceArrays;
use crate::loading::Classpath;
use crate::native_allocation::NativeAllocator;
use crate::new_java_values::allocated_objects::{AllocatedNormalObjectHandle, AllocatedObjectHandleByAddress};
use crate::new_java_values::unallocated_objects::{ObjectFields, UnAllocatedObjectObject};
use crate::options::{ExitTracingOptions, InstructionTraceOptions, ThreadTracingOptions};
use crate::rust_jni::invoke_interface::get_invoke_interface_new;
use crate::rust_jni::jvmti::SharedLibJVMTI;
use crate::rust_jni::mangling::ManglingRegex;
use crate::rust_jni::PerStackInterfaces;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::lang::stack_trace_element::StackTraceElement;
use crate::string_exit_cache::StringExitCache;
use crate::threading::safepoints::Monitor2;
use crate::threading::thread_state::ThreadState;
use crate::tracing::TracingSettings;

pub static mut JVM: Option<&'static JVMState> = None;

pub struct JVMConfig {
    pub compiled_mode_active: bool,
    pub store_generated_classes: bool,
    pub debug_print_exceptions: bool,
    pub assertions_enabled: bool,
    pub tracing: TracingSettings,
    pub main_class_name: CClassName,
    pub compile_threshold: u64,
}

thread_local!(pub static CURRENT_THREAD_INVOKE_INTERFACE: RefCell<Option<*const JNIInvokeInterfaceNamedReservedPointers>> = RefCell::new(None));

pub struct Native {
    pub jvmti_state: Option<JVMTIState>,
    pub invoke_interface: &'static std::thread::LocalKey<RefCell<Option<*const JNIInvokeInterfaceNamedReservedPointers>>>,
    pub native_interface_allocations: NativeAllocator,
}

pub struct JVMState<'gc> {
    pub config: JVMConfig,
    pub java_vm_state: JavaVMStateWrapper<'gc>,
    pub gc: &'gc GC<'gc>,
    pub native_libaries: NativeLibraries<'gc>,
    pub properties: Vec<(String, String)>,
    pub string_pool: &'gc CompressedClassfileStringPool,
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
    pub instruction_tracing_options: InstructionTraceOptions,
    pub exit_tracing_options: ExitTracingOptions,
    pub thread_tracing_options: ThreadTracingOptions,
    pub checkcast_debug_assertions: bool,
    pub perf_metrics: PerfMetrics,
    pub recompilation_conditions: RwLock<RecompileConditions>,
    pub vtables: Mutex<VTables<'gc>>,
    pub itables: Mutex<ITables<'gc>>,
    pub interface_table: InterfaceTable<'gc>,
    pub invoke_virtual_lookup_cache: RwLock<InvokeVirtualLookupCache<'gc>>,
    pub invoke_interface_lookup_cache: RwLock<InvokeInterfaceLookupCache<'gc>>,
    pub string_exit_cache: RwLock<StringExitCache<'gc>>,
    pub function_execution_count: FunctionInstructionExecutionCount,
    pub class_ids: ClassIDs,
    pub inheritance_tree: InheritanceTree,
    pub bit_vec_paths: RwLock<BitVecPaths>,
    pub interface_arrays: RwLock<InterfaceArrays>,
    pub program_args_array: OnceCell<AllocatedHandle<'gc>>,
    pub mangling_regex: ManglingRegex,
    pub default_per_stack_initial_interfaces: PerStackInterfaces,
    pub all_the_static_fields: AllTheStaticFields<'gc>,
    pub java_home: PathBuf,
    pub boot_classpath: Vec<PathBuf>,
}


pub struct Classes<'gc> {
    //todo needs to be used for all instances of getClass
    pub loaded_classes_by_type: HashMap<LoaderName, HashMap<CPDType, Arc<RuntimeClass<'gc>>>>,
    pub initiating_loaders: HashMap<CPDType, (LoaderName, Arc<RuntimeClass<'gc>>)>,
    pub class_object_pool: BiMap<ByAddressAllocatedObject<'gc>, ByAddress<Arc<RuntimeClass<'gc>>>>,
    pub anon_classes: Vec<Arc<RuntimeClass<'gc>>>,
    pub anon_class_live_object_ldc_pool: Vec<AllocatedHandle<'gc>>,
    pub class_class: Arc<RuntimeClass<'gc>>,
    pub class_loaders: BiMap<LoaderIndex, ByAddressAllocatedObject<'gc>>,
    pub protection_domains: BiMap<ByAddress<Arc<RuntimeClass<'gc>>>, ByAddressAllocatedObject<'gc>>,
    pub class_class_view: Arc<ClassBackedView>,
    pub object_view: Arc<ClassBackedView>,
}

impl<'gc> Classes<'gc> {
    pub fn debug_assert(&self, jvm: &'gc JVMState<'gc>) {
        for allocated_obj in self.class_object_pool.left_values() {
            let handle = allocated_obj.owned_inner_ref().duplicate_discouraged();
            handle.cast_class().debug_assert(jvm);
        }
    }

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

    pub fn lookup_live_object_pool(&self, idx: &LiveObjectIndex) -> AllocatedHandle<'gc> {
        self.anon_class_live_object_ldc_pool[idx.0].duplicate_discouraged()
    }

    pub fn get_loader_and_runtime_class(&self, cpdtype: &CPDType) -> Option<(LoaderName, Arc<RuntimeClass<'gc>>)> {
        Some(self.initiating_loaders.get(cpdtype)?.clone())
    }
}


impl<'gc> JVMState<'gc> {
    pub fn boot_classpath_string(&self) -> String {
        self.boot_classpath.iter().map(|path|path.to_str().unwrap()).join(":")
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

    pub fn verify_class_and_object(&self,
                                   object_runtime_class: Arc<RuntimeClass<'gc>>,
                                   class_runtime_class: Arc<RuntimeClass<'gc>>,
    ) {
        let mut context = VerifierContext {
            live_pool_getter: Arc::new(DefaultLivePoolGetter {}) as Arc<dyn LivePoolGetter>,
            classfile_getter: Arc::new(DefaultClassfileGetter { jvm: self }) as Arc<dyn ClassFileGetter>,
            string_pool: &self.string_pool,
            current_class: CClassName::object(),
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
        context.current_class = CClassName::class();
        let lookup = self.classpath.lookup(&CClassName::class(), &self.string_pool).expect("Can not find Class class");
        verify(&mut context, CClassName::class(), LoaderName::BootstrapLoader).expect("Class doesn't verify");
        self.sink_function_verification_date(&context.verification_types, class_runtime_class.clone());

        for interface in class_runtime_class.unwrap_class_class().interfaces.iter() {
            context.verification_types.clear();
            context.current_class = interface.cpdtype().unwrap_class_type();
            let name = interface.cpdtype().unwrap_class_type();
            let lookup = self.classpath.lookup(&name, &self.string_pool).expect("Can not find Class class jni_interface");
            verify(&mut context, name, LoaderName::BootstrapLoader).expect("Class doesn't verify");
            self.sink_function_verification_date(&context.verification_types, interface.clone());
        }
    }


    pub fn early_add_class(&'gc self, cpd_type_table: &RwLock<CPDTypeTable>, class: Arc<RuntimeClass<'gc>>) {
        let class_class = self.classes.read().unwrap().class_class.clone();
        let class_class_class_unwrapped = class_class.unwrap_class_class();
        assert!(!class.cpdtype().is_array());
        assert!(!class.cpdtype().is_primitive());
        let class_intrinsic_data = ClassIntrinsicsData {
            is_array: false,
            is_primitive: false,
            component_type: None,
            this_cpdtype: class.cpdtype(),
        };
        let class_object_handle = self.allocate_object(UnAllocatedObject::Object(UnAllocatedObjectObject {
            object_rc: self.classes.read().unwrap().class_class.clone(),
            object_fields: ObjectFields::new_default_with_hidden_fields(&class_class_class_unwrapped.object_layout),
        }));
        let class_object_handle = class_object_handle.cast_class().apply_intrinsic_data(&class_class, cpd_type_table, class_intrinsic_data).object();
        let class_object = self.gc.handle_lives_for_gc_life(class_object_handle);
        let mut classes = self.classes.write().unwrap();
        classes.class_object_pool.insert(ByAddressAllocatedObject::Owned(class_object.duplicate_discouraged()), ByAddress(class.clone()));
        classes.loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(class.clone().cpdtype(), class.clone());
    }

    pub fn add_class_class_class_object(&'gc self, cpd_type_table: &RwLock<CPDTypeTable>) {
        //todo desketchify this
        let class_class = self.classes.read().unwrap().class_class.clone();
        self.early_add_class(cpd_type_table, class_class.clone());
        for interface in class_class.unwrap_class_class().interfaces.iter() {
            self.early_add_class(cpd_type_table, interface.clone());
        }
    }

    pub fn get_class_class_or_object_class_method_numbers(pool: &CompressedClassfileStringPool, class_class_view: &dyn ClassView, parent: Option<&dyn ClassView>) -> (u32, HashMap<MethodShape, MethodNumber>) {
        let mut method_number_mappings = MethodNumberMappings::new();

        if let Some(parent) = parent {
            for method_shape in parent.methods()
                .filter(|method| !method.is_static())
                .map(|method| ShapeOrderWrapperOwned(method.method_shape())).sorted() {
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
        self.thread_state.debug_assert(self);
        let res = self.gc.allocate_object(self, object);
        self.thread_state.debug_assert(self);
        res
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

    pub fn num_local_var_slots(&self, method_id: MethodId) -> u16 {
        let (rc, method_i) = self.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        method_view.local_var_slots()
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
    pub libjava_path: PathBuf,
    pub native_libs: RwLock<HashMap<String, NativeLib>>,
    pub registered_natives: RwLock<HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, RwLock<HashMap<u16, unsafe extern "C" fn()>>>>,
}

fn default_on_load(_: *mut *const JNIInvokeInterfaceNamedReservedPointers, _: *mut c_void) -> i32 {
    JNI_VERSION_1_1 as i32
}

impl<'gc> NativeLibraries<'gc> {
    pub unsafe fn load<'l>(&self, jvm: &'gc JVMState<'gc>, opaque_frame: &mut OpaqueFrame<'gc, '_>, path: &PathBuf, name: String) {
        let onload_fn_ptr = self.get_onload_ptr_and_add(path, name);
        let interface: *const JNIInvokeInterfaceNamedReservedPointers = get_invoke_interface_new(jvm, opaque_frame);
        onload_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterfaceNamedReservedPointers, null_mut());
        //todo check return res
    }

    pub unsafe fn load_old<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, path: &PathBuf, name: String) {
        let onload_fn_ptr = self.get_onload_ptr_and_add(path, name);
        let interface: *const JNIInvokeInterfaceNamedReservedPointers = todo!()/*get_invoke_interface(jvm, todo!()/*int_state*/)*/;
        onload_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterfaceNamedReservedPointers, null_mut());
        //todo check return res
    }

    pub unsafe fn get_onload_ptr_and_add(&self, path: &PathBuf, name: String) -> fn(*mut *const JNIInvokeInterfaceNamedReservedPointers, *mut c_void) -> i32 {
        let lib = Library::new(path, (RTLD_LAZY | RTLD_GLOBAL) as i32).unwrap();
        let on_load = match lib.get::<fn(vm: *mut *const JNIInvokeInterfaceNamedReservedPointers, reserved: *mut c_void) -> jint>("JNI_OnLoad".as_bytes()) {
            Ok(x) => Some(x),
            Err(err) => {
                if err.to_string().contains(" undefined symbol: JNI_OnLoad") {
                    None
                } else {
                    todo!()
                }
            }
        };
        let onload_fn_ptr = on_load.map(|on_load| *on_load.deref()).unwrap_or(default_on_load);
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
        match monitors_guard.get(&obj_ptr) {
            None => {
                let new_monitor = self.thread_state.new_monitor("".to_string());
                monitors_guard.insert(obj_ptr, new_monitor.clone());
                new_monitor
            }
            Some(monitor) => {
                monitor.clone()
            }
        }
    }
}

pub struct BootstrapLoaderClassGetter<'vm, 'l> {
    jvm: &'l JVMState<'vm>,
}

impl ClassFileGetter for BootstrapLoaderClassGetter<'_, '_> {
    fn get_classfile(&self, vf_context: &VerifierContext, loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
        assert_eq!(loader, LoaderName::BootstrapLoader);
        Ok(Arc::new(ClassBackedView::from(self.jvm.classpath.lookup(&class, &self.jvm.string_pool)?, &self.jvm.string_pool)))
    }
}

pub struct StringInternment<'gc> {
    pub strings: HashMap<Vec<u16>, AllocatedHandle<'gc>>,
}