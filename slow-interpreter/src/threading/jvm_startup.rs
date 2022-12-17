use std::ops::Deref;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::channel;
use libloading::Symbol;
use wtf8::Wtf8Buf;
use jvmti_jni_bindings::{JVMTI_THREAD_NORM_PRIORITY};
use jvmti_jni_bindings::invoke_interface::JNIInvokeInterfaceNamedReservedPointers;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::loading::LoaderName;
use crate::{check_initing_or_inited_class, check_loaded_class, JVMState, MethodResolverImpl, NewJavaValue, NewJavaValueHandle, PushableFrame, run_function, run_main, set_properties, StackEntryPush, WasException};
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::new_object_full;
use crate::rust_jni::invoke_interface::get_invoke_interface_new;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::lang::system::System;
use crate::stdlib::java::lang::thread::JThread;
use crate::stdlib::java::lang::thread_group::JThreadGroup;
use crate::threading::java_thread::JavaThread;

pub struct MainThreadStartInfo {
    pub args: Vec<String>,
}


fn jvm_init_from_main_thread<'l, 'gc>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) {
    let main_thread = jvm.thread_state.get_main_thread();
    main_thread.thread_object().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
    let system_class = assert_inited_or_initing_class(jvm, CClassName::system().into());


    let system = &system_class;
    let system_view = system.view();
    let method_views = system_view.lookup_method_name(MethodName::method_initializeSystemClass());
    let init_method_view = method_views.first().unwrap().clone();
    let method_id = jvm.method_table.write().unwrap().get_method_id(system_class.clone(), init_method_view.method_i());
    jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolverImpl { jvm, loader: LoaderName::BootstrapLoader }, method_id, false);
    let mut locals = vec![];
    for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
        locals.push(NewJavaValue::Top);
    }
    let initialize_system_frame = StackEntryPush::new_java_frame(jvm, system_class.clone(), init_method_view.method_i() as u16, locals);
    let _init_frame_guard: Result<(), WasException<'gc>> = int_state.push_frame_java(initialize_system_frame, |java_frame| {
        assert!(Arc::ptr_eq(&main_thread, &jvm.thread_state.get_current_thread()));
        match run_function(&jvm, java_frame) {
            Ok(_) => {}
            Err(WasException{ exception_obj }) => {
                exception_obj.print_stack_trace(jvm,java_frame).expect("exception printing exception");
                todo!();
            },
        }
        Ok(())
    });
    set_properties(jvm, int_state).expect("todo");
    //todo read and copy props here
    // let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("java.home".to_string())).expect("todo");
    // let value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("/home/francis/builds/jvm-dep-dir/jdk8u/build/linux-x86_64-normal-server-fastdebug/jdk/".to_string())).expect("todo");
    // System::props(jvm, int_state).set_property(jvm, int_state, key, value).expect("todo");

    let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("log4j2.disable.jmx".to_string())).expect("todo");
    let value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("true".to_string())).expect("todo");
    System::props(jvm).set_property(jvm, int_state, key, value).expect("todo");
    eprintln!("JVM INIT COMPLETE")
}


pub fn bootstrap_main_thread<'vm>(jvm: &'vm JVMState<'vm>, main_thread_start_info: MainThreadStartInfo) -> Arc<JavaThread<'vm>> {
    let main_jthread = JavaThread::new_with_stack_on_this_thread(jvm, None, true, move |bootstrap_thread, opaque_frame| {
        unsafe {
            jvm.native_libaries.load(jvm, &jvm.native_libaries.libjava_path, "java".to_string());
            {
                let native_libs_guard = jvm.native_libaries.native_libs.read().unwrap();
                let libjava_native_lib = native_libs_guard.get("java").unwrap();
                let setup_hack_symbol: Symbol<unsafe extern "system" fn(*const JNIInvokeInterfaceNamedReservedPointers)> = libjava_native_lib.library.get("setup_jvm_pointer_hack".as_bytes()).unwrap();
                (*setup_hack_symbol.deref())(get_invoke_interface_new(jvm))
            }
        }
        let frame = StackEntryPush::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "bootstrapping opaque frame");
        opaque_frame.push_frame_opaque(frame, |java_stack_guard| {
            let object_rc = check_loaded_class(jvm, java_stack_guard, CClassName::object().into()).expect("This should really never happen, since it is equivalent to a class not found exception on java/lang/Object");
            jvm.verify_class_and_object(object_rc, jvm.classes.read().unwrap().class_class.clone());
            let thread_classfile = check_initing_or_inited_class(jvm, java_stack_guard, CClassName::thread().into()).expect("couldn't load thread class");

            let thread_object = NewJavaValueHandle::Object(new_object_full(jvm, java_stack_guard, &thread_classfile)).cast_thread(jvm);
            thread_object.set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
            *bootstrap_thread.thread_object.write().unwrap() = thread_object.into();
            let thread_group_class = check_initing_or_inited_class(jvm, java_stack_guard, CClassName::thread_group().into()).expect("couldn't load thread group class");
            let system_thread_group = JThreadGroup::init(jvm, java_stack_guard, thread_group_class).expect("todo");
            *jvm.thread_state.system_thread_group.write().unwrap() = system_thread_group.clone().into();
            let main_jthread = JThread::new(jvm, java_stack_guard, system_thread_group, "Main".to_string()).expect("todo");
            Ok(main_jthread)
        })
    }).unwrap();
    let (main_send, main_recv) = channel();
    let res = JavaThread::background_new_with_stack(jvm, Some(main_jthread), false, move |main_thread, opaque_frame| {
        *jvm.thread_state.main_thread.write().unwrap() = main_thread.clone().into();
        main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        main_thread.notify_alive(); //is this too early?
        jvm.jvmti_state().map(|jvmti| jvmti.built_in_jdwp.agent_load(jvm, opaque_frame)); // technically this is to late and should have been called earlier, but needs to be on this thread.
        jvm_init_from_main_thread(jvm, opaque_frame);

        assert!(!jvm.live.load(Ordering::SeqCst));
        jvm.live.store(true, Ordering::SeqCst);
        if let Some(jvmti) = jvm.jvmti_state() {
            jvmti.built_in_jdwp.vm_inited(jvm, todo!()/*&mut int_state*/, main_thread.clone())
        }
        let MainThreadStartInfo { args } = main_recv.recv().unwrap();
        //from the jvmti_interface spec:
        //"The thread start event for the main application thread is guaranteed not to occur until after the handler for the VM initialization event returns. "
        if let Some(jvmti) = jvm.jvmti_state() {
            jvmti.built_in_jdwp.thread_start(jvm, opaque_frame, main_thread.thread_object())
        }
        opaque_frame.push_frame_opaque(StackEntryPush::new_completely_opaque_frame(jvm, LoaderName::BootstrapLoader, vec![], "main thread main frame"), |opaque_frame| {
            run_main(args, jvm, opaque_frame).unwrap();
            Ok(())
        }).unwrap();
        //todo handle exception exit from main
        main_thread.notify_terminated();
        Ok(())
    }).expect("todo");
    main_send.send(main_thread_start_info).unwrap();
    res
}