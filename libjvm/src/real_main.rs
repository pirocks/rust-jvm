use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ffi::OsString;
use std::mem::transmute;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::AtomicBool;
use std::thread::Scope;
use std::time::{Duration, Instant};

use argparse::{ArgumentParser, List, Store, StoreTrue};
use clap::Parser;
use itertools::all;
use raw_cpuid::CpuId;

use classfile_view::view::ClassBackedView;
use gc_memory_layout_common::early_startup::get_regions;
use inheritance_tree::bit_vec_path::BitVecPaths;
use inheritance_tree::class_ids::ClassIDs;
use inheritance_tree::InheritanceTree;
use interface_vtable::ITables;
use interface_vtable::lookup_cache::InvokeInterfaceLookupCache;
use interfaces::initial_per_stack_interfaces;
use jvm_args::JVMArgs;
use method_table::interface_table::InterfaceTable;
use method_table::MethodTable;
use perf_metrics::PerfMetrics;
use runtime_class_stuff::{ClassStatus, RuntimeClass, RuntimeClassClass};
use runtime_class_stuff::static_fields::AllTheStaticFields;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::class_names::{CClassName, CompressedClassName};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;
use rust_jvm_common::cpdtype_table::CPDTypeTable;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::method_shape::MethodShapeIDs;
use rust_jvm_common::opaque_id_table::OpaqueIDs;
use sketch_jvm_version_of_utf8::wtf8_pool::Wtf8Pool;
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::better_java_stack::remote_frame::RemoteFrame;
use slow_interpreter::field_table::FieldTable;
use slow_interpreter::function_instruction_count::FunctionInstructionExecutionCount;
use slow_interpreter::ir_to_java_layer::java_vm_state::JavaVMStateWrapper;
use slow_interpreter::java_values::GC;
use slow_interpreter::jvm_state::{Classes, CURRENT_THREAD_INVOKE_INTERFACE, JVM, JVMConfig, JVMState, JVMTIState, Native, NativeLibraries, StringInternment};
use slow_interpreter::leaked_interface_arrays::InterfaceArrays;
use slow_interpreter::loading::Classpath;
use slow_interpreter::native_allocation::NativeAllocator;
use slow_interpreter::options::{JVMOptions, JVMOptionsStart, SharedLibraryPaths};
use slow_interpreter::rust_jni::jvmti::SharedLibJVMTI;
use slow_interpreter::rust_jni::mangling::ManglingRegex;
use slow_interpreter::string_exit_cache::StringExitCache;
use slow_interpreter::threading::java_thread::JavaThread;
use slow_interpreter::threading::jvm_startup::{bootstrap_main_thread, MainThreadStartInfo};
use slow_interpreter::threading::thread_state::ThreadState;
use slow_interpreter::tracing::TracingSettings;
use stage0::compiler::RecompileConditions;
use stage0::compiler_common::frame_data::FunctionFrameData;
use vtable::lookup_cache::InvokeVirtualLookupCache;
use vtable::VTables;

#[no_mangle]
unsafe extern "system" fn rust_jvm_real_main() {
    avx_check();
    main_()
}

fn avx_check() {
    let cpuid = CpuId::new();
    //todo figure out why these libs don't allow for checking avx2
    //or maybe use safe_arch = "0.6.0"
    if !cpuid.get_feature_info().expect("Cpuid doesn't work?").has_avx() {
        eprintln!("This JVM requires AVX");
    }
}

pub fn main_<'l, 'env>() {
    let jvm_args: JVMArgs = JVMArgs::parse();
    let jvm_options_start = JVMOptionsStart::from_java_home(jvm_args.java_home.clone(), jvm_args);
    let jvm_options = JVMOptions::from_options_start(jvm_options_start);
    let gc: GC<'l> = GC::new(get_regions());
    std::thread::scope::<'env>(|scope: &Scope<'_, 'env>| {
        let gc_ref: &'l GC = unsafe { transmute(&gc) };//todo why do I need this?
        let scope_ref: &'l Scope<'l, 'l> = unsafe { transmute(scope) };
        let string_pool = CompressedClassfileStringPool::new();
        let string_pool_ref: &'l CompressedClassfileStringPool = unsafe { transmute(&string_pool) };
        within_thread_scope(scope_ref, jvm_options, gc_ref, string_pool_ref);
    });
    panic!();
}

fn within_thread_scope<'l>(scope: &'l Scope<'l, 'l>, jvm_options: JVMOptions, gc: &'l GC<'l>, string_pool: &'l CompressedClassfileStringPool) {
    let (args, jvm): (Vec<String>, JVMState<'l>) = initial_jvm_state(jvm_options, scope, gc, string_pool);

    let jvm_ref: &'l JVMState<'l> = Box::leak(box jvm);
    main_run(args, &jvm_ref);
    //todo clean jvm shutdown
    std::process::exit(0);
}

pub fn main_run<'gc>(args: Vec<String>, jvm_ref: &'gc JVMState<'gc>) {
    jvm_ref.java_vm_state.init(jvm_ref);
    unsafe { JVM = Some(transmute(jvm_ref)) }
    jvm_ref.add_class_class_class_object(&jvm_ref.cpdtype_table);
    let thread_state = &jvm_ref.thread_state;
    let main_thread: Arc<JavaThread> = bootstrap_main_thread(jvm_ref, MainThreadStartInfo { args });
    let main_thread_clone = main_thread.clone();
    // jvm_ref.thread_state.threads.create_thread(Some("stacktracer".to_string())).start_thread(box move |_| unsafe {
    //     loop {
    //         for (jtid, java_thread) in jvm_ref.thread_state.get_all_threads().iter() {
    //             if let Some(name) = java_thread.thread_object().try_name(jvm_ref) {
    //                 dbg!(name.to_rust_string(jvm_ref));
    //             } else {
    //                 dbg!("unnamed");
    //             }
    //             java_thread.clone().pause_and_remote_view(jvm_ref, |remote_frame| {
    //                 remote_frame.debug_print_stack_trace(jvm_ref);
    //                 let method_id = remote_frame.frame_ref().method_id().unwrap();
    //                 dbg!(jvm_ref.method_table.read().unwrap().lookup_method_string(method_id, &jvm_ref.string_pool));
    //                 dbg!(method_id);
    //                 ()
    //             });
    //         }
    //         std::thread::sleep(Duration::from_millis(1000));
    //     }
    // }, box ());
    main_thread.get_underlying().join();
    jvm_ref.thread_state.wait_all_non_daemon_threads(jvm_ref);
}

pub fn initial_jvm_state<'gc>(jvm_options: JVMOptions, scope: &'gc Scope<'gc, 'gc>, gc: &'gc GC<'gc>, string_pool: &'gc CompressedClassfileStringPool) -> (Vec<String>, JVMState<'gc>) {
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
        thread_tracing_options,
        java_home,
        boot_classpath,
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
    let all_the_static_fields = AllTheStaticFields::new(string_pool);
    let thread_state = ThreadState::new(scope);
    let class_ids = ClassIDs::new();
    let object_class_id = class_ids.get_id_or_add(CPDType::object());
    let inheritance_tree = InheritanceTree::new(object_class_id);
    let bt_vec_paths = RwLock::new(BitVecPaths::new());
    let classes = init_classes(&string_pool, &all_the_static_fields, &class_ids, &inheritance_tree, &mut bt_vec_paths.write().unwrap(), &classpath_arc);
    let main_class_name = CompressedClassName(string_pool.add_name(main_class_name.get_referred_name().clone(), true));

    let jvm = JVMState {
        config: JVMConfig {
            store_generated_classes,
            debug_print_exceptions,
            assertions_enabled,
            compiled_mode_active: true,
            tracing,
            main_class_name,
            compile_threshold: 1000,
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
            invoke_interface: &CURRENT_THREAD_INVOKE_INTERFACE,
            native_interface_allocations: NativeAllocator::new(),
        },
        live: AtomicBool::new(false),
        resolved_method_handles: RwLock::new(HashMap::new()),
        include_name_field: AtomicBool::new(false),
        stacktraces_by_throwable: RwLock::new(HashMap::new()),
        java_vm_state: JavaVMStateWrapper::new(),
        java_function_frame_data: Default::default(),
        object_monitors: Default::default(),
        method_shapes: MethodShapeIDs::new(),
        instruction_tracing_options: instruction_trace_options,
        exit_tracing_options: exit_trace_options,
        thread_tracing_options,
        checkcast_debug_assertions: false,
        perf_metrics: PerfMetrics::new(),
        recompilation_conditions: RwLock::new(RecompileConditions::new()),
        vtables: Mutex::new(VTables::new()),
        itables: Mutex::new(ITables::new()),
        interface_table: InterfaceTable::new(),
        invoke_virtual_lookup_cache: RwLock::new(InvokeVirtualLookupCache::new()),
        invoke_interface_lookup_cache: RwLock::new(InvokeInterfaceLookupCache::new()),
        string_exit_cache: RwLock::new(StringExitCache::new()),
        function_frame_type_data: RwLock::new(FunctionFrameData {
            no_tops: Default::default(),
            tops: Default::default(),
        }),
        function_execution_count: FunctionInstructionExecutionCount::new(),
        class_ids,
        inheritance_tree,
        bit_vec_paths: bt_vec_paths,
        interface_arrays: RwLock::new(InterfaceArrays::new()),
        program_args_array: Default::default(),
        mangling_regex: ManglingRegex::new(),
        default_per_stack_initial_interfaces: initial_per_stack_interfaces(),
        all_the_static_fields,
        java_home,
        boot_classpath,
    };
    (args, jvm)
}


fn init_classes<'gc>(pool: &CompressedClassfileStringPool, all_the_static_fields: &AllTheStaticFields, class_ids: &ClassIDs, inheritance_tree: &InheritanceTree, bit_vec_paths: &mut BitVecPaths, classpath_arc: &Arc<Classpath>) -> RwLock<Classes<'gc>> {
    //todo turn this into a ::new
    let class_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::class(), pool).unwrap(), pool));
    let serializable_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::serializable(), pool).unwrap(), pool));
    let generic_declaration_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::generic_declaration(), pool).unwrap(), pool));
    let type_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::type_(), pool).unwrap(), pool));
    let annotated_element_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::annotated_element(), pool).unwrap(), pool));
    let object_class_view = Arc::new(ClassBackedView::from(classpath_arc.lookup(&CClassName::object(), pool).unwrap(), pool));
    let temp_object_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new_new(
        inheritance_tree,
        all_the_static_fields,
        bit_vec_paths,
        object_class_view.clone(),
        None,
        vec![],
        RwLock::new(ClassStatus::UNPREPARED),
        pool,
        class_ids,
    )));

    let annotated_element_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new_new(
        inheritance_tree,
        all_the_static_fields,
        bit_vec_paths,
        annotated_element_view,
        None,
        vec![],
        RwLock::new(ClassStatus::UNPREPARED),
        pool,
        class_ids,
    )));

    let type_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new_new(
        inheritance_tree,
        all_the_static_fields,
        bit_vec_paths,
        type_view,
        None,
        vec![],
        RwLock::new(ClassStatus::UNPREPARED),
        pool,
        class_ids,
    )));

    let serializable_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new_new(
        inheritance_tree,
        all_the_static_fields,
        bit_vec_paths,
        serializable_view,
        None,
        vec![],
        RwLock::new(ClassStatus::UNPREPARED),
        pool,
        class_ids,
    )));


    let generic_declaration_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new_new(
        inheritance_tree,
        all_the_static_fields,
        bit_vec_paths,
        generic_declaration_view,
        None,
        vec![annotated_element_class.clone()],
        RwLock::new(ClassStatus::UNPREPARED),
        pool,
        class_ids,
    )));
    //todo Class does implement several interfaces, but non handled here
    let class_class = Arc::new(RuntimeClass::Object(RuntimeClassClass::new_new(
        inheritance_tree,
        all_the_static_fields,
        bit_vec_paths,
        class_view.clone(),
        Some(temp_object_class),
        vec![annotated_element_class.clone(), generic_declaration_class.clone(), type_class.clone(), serializable_class.clone()],
        RwLock::new(ClassStatus::UNPREPARED),
        pool,
        class_ids,
    )));
    let mut loaded_classes_by_type: HashMap<LoaderName, HashMap<CPDType, Arc<RuntimeClass>>> = Default::default();
    loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::class().into(), class_class.clone());
    loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::serializable().into(), serializable_class.clone());
    loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::type_().into(), type_class.clone());
    loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::generic_declaration().into(), generic_declaration_class.clone());
    loaded_classes_by_type.entry(LoaderName::BootstrapLoader).or_default().insert(CClassName::annotated_element().into(), annotated_element_class.clone());
    let mut initiating_loaders: HashMap<CPDType, (LoaderName, Arc<RuntimeClass<'gc>>)> = Default::default();
    initiating_loaders.insert(CClassName::class().into(), (LoaderName::BootstrapLoader, class_class.clone()));
    initiating_loaders.insert(CClassName::serializable().into(), (LoaderName::BootstrapLoader, serializable_class.clone()));
    initiating_loaders.insert(CClassName::type_().into(), (LoaderName::BootstrapLoader, type_class.clone()));
    initiating_loaders.insert(CClassName::generic_declaration().into(), (LoaderName::BootstrapLoader, generic_declaration_class.clone()));
    initiating_loaders.insert(CClassName::annotated_element().into(), (LoaderName::BootstrapLoader, annotated_element_class.clone()));
    let classes = RwLock::new(Classes {
        loaded_classes_by_type,
        initiating_loaders,
        class_object_pool: Default::default(),
        anon_classes: Default::default(),
        anon_class_live_object_ldc_pool: Vec::new(),
        class_class,
        class_loaders: Default::default(),
        protection_domains: Default::default(),
        class_class_view: class_view,
        object_view: object_class_view,
    });
    classes
}