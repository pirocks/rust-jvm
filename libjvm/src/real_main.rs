use std::ffi::OsString;
use std::mem::transmute;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::thread::Scope;
use std::time::Duration;

use argparse::{ArgumentParser, List, Store, StoreTrue};
use raw_cpuid::CpuId;

use gc_memory_layout_common::early_startup::get_regions;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;
use slow_interpreter::better_java_stack::frames::{HasFrame};
use slow_interpreter::better_java_stack::remote_frame::RemoteFrame;
use slow_interpreter::java_values::GC;
use slow_interpreter::jvm_state::{JVM, JVMState};
use slow_interpreter::loading::Classpath;
use slow_interpreter::options::JVMOptions;
use slow_interpreter::threading::java_thread::JavaThread;
use slow_interpreter::threading::jvm_startup::{bootstrap_main_thread, MainThreadStartInfo};

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
    let mut verbose = false;
    let mut debug = false;
    let mut main_class_name = "".to_string();
    let mut class_entries: Vec<String> = vec![];
    let mut args: Vec<String> = vec![];
    let mut properties: Vec<String> = vec!["java.security.egd".to_string(), "file:/dev/urandom".to_string()];
    let mut libjava: OsString = OsString::new();
    let mut enable_tracing = false;
    let mut enable_jvmti = false;
    let mut unittest_mode = false;
    let mut store_generated_options = false;
    let mut debug_print_exceptions = false;
    let mut assertions_enabled: bool = false;
    let mut libjdwp: OsString = OsString::from_str("/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/lib/amd64/libjdwp.so").unwrap();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("A jvm written partially in rust");
        ap.refer(&mut verbose).add_option(&["-v", "--verbose"], StoreTrue, "Be verbose");
        ap.refer(&mut debug).add_option(&["-v", "--verbose"], StoreTrue, "Log debug info");
        ap.refer(&mut main_class_name).add_option(&["--main"], Store, "Main class");
        ap.refer(&mut class_entries).add_option(&["--classpath"], List, "A list of directories from which to load classes");
        ap.refer(&mut args).add_option(&["--args"], List, "A list of args to pass to main");
        ap.refer(&mut libjava).add_option(&["--libjava"], Store, "");
        ap.refer(&mut libjdwp).add_option(&["--libjdwp"], Store, "");
        ap.refer(&mut enable_tracing).add_option(&["--tracing"], StoreTrue, "Enable debug tracing");
        ap.refer(&mut enable_jvmti).add_option(&["--jvmti_interface"], StoreTrue, "Enable JVMTI");
        ap.refer(&mut properties).add_option(&["--properties"], List, "Set JVM Properties");
        ap.refer(&mut unittest_mode).add_option(&["--unittest-mode"], StoreTrue, "Enable Unittest mode. This causes the main class to be ignored");
        ap.refer(&mut store_generated_options).add_option(&["--store-anon-class"], StoreTrue, "Enables writing out of classes defined with Unsafe.defineClass");
        ap.refer(&mut debug_print_exceptions).add_option(&["--debug-exceptions"], StoreTrue, "print exceptions even if caught");
        ap.refer(&mut assertions_enabled).add_option(&["--ea"], StoreTrue, "enable assertions");
        ap.parse_args_or_exit();
    }

    let classpath = Classpath::from_dirs(class_entries.iter().map(|x| Path::new(x).into()).collect());
    let main_class_name = ClassName::Str(main_class_name.replace('.', "/"));
    let jvm_options = JVMOptions::new(main_class_name, classpath, args, libjava, libjdwp, enable_tracing, enable_jvmti, properties, unittest_mode, store_generated_options, debug_print_exceptions, assertions_enabled);
    let gc: GC<'l> = GC::new(get_regions());
    std::thread::scope::<'env>(|scope: &Scope<'_, 'env>| {
        let gc_ref: &'l GC = unsafe { transmute(&gc) };//todo why do I need this?
        let scope_ref: &'l Scope<'l, 'l> = unsafe { transmute(scope) };
        within_thread_scope(scope_ref, jvm_options, gc_ref);
    });
    panic!();
}

fn within_thread_scope<'l>(scope: &'l Scope<'l, 'l>, jvm_options: JVMOptions, gc: &'l GC<'l>) {
    let (args, jvm): (Vec<String>, JVMState<'l>) = JVMState::new(jvm_options, scope, gc, CompressedClassfileStringPool::new());

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
    let main_thread: Arc<JavaThread> = bootstrap_main_thread(jvm_ref, &thread_state.threads, MainThreadStartInfo { args });
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
