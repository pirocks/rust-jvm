use slow_interpreter::interpreter_util::{push_new_object, check_inited_class};

use std::cell::RefCell;
use std::sync::{Arc, RwLockWriteGuard, RwLock, Condvar};
use rust_jvm_common::classnames::ClassName;
use jvmti_jni_bindings::{JNIEnv, jclass, jobject, jlong, jint, jboolean, jobjectArray, jstring, jintArray};
use slow_interpreter::rust_jni::native_util::{get_state, get_frame, to_object, from_object, from_jclass};
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::{JVMState, JavaThread, InterpreterState, SuspendedStatus};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::stack_entry::StackEntry;
use std::ops::Deref;
use std::rc::Rc;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use parking_lot::Mutex;
use slow_interpreter::interpreter::run_function;
use descriptor_parser::MethodDescriptor;
use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::ptype::PType;
use nix::unistd::gettid;
use nix::sys::pthread::pthread_self;
use std::time::Duration;
use std::thread::sleep;

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
//    assert!(Arc::ptr_eq(MAIN_THREAD.as_ref().unwrap(),&from_object(thread).unwrap()));//todo why does this not pass?
    let jvm = get_state(env);
    let thread_object = JavaValue::Object(from_object(thread)).cast_thread();
    let tid = thread_object.tid();
    // dbg!("start");
    // dbg!(thread_object.name().to_rust_string());
    let mut all_threads_guard = jvm.thread_state.alive_threads.write().unwrap();
    if all_threads_guard.contains_key(&tid) || &jvm.main_thread().java_tid == &tid {
        //todo for now we ignore this, but irl we should only ignore this for main thread
    } else {
        let frame = get_frame(env);
        let thread_class = check_inited_class(jvm, &ClassName::thread().into(), frame.class_pointer.loader(jvm).clone());
        let thread_creation_complete = Arc::new(Condvar::new());
        let thread_creation_complete_copy = thread_creation_complete.clone();
        let mutex = std::sync::Mutex::new(());
        std::thread::spawn(move || {
            let thread_from_rust = Arc::new(JavaThread {
                java_tid: tid,
                call_stack: RefCell::new(vec![]),
                thread_object: RefCell::new(thread_object.into()),
                interpreter_state: InterpreterState::default(),
                unix_tid: gettid()
            });
            jvm.thread_state.alive_threads.write().unwrap().insert(tid, thread_from_rust.clone());
            jvm.init_signal_handler();
            thread_creation_complete.clone().notify_one();
            let new_thread_frame = Rc::new(StackEntry {
                class_pointer: thread_class.clone(),
                method_i: std::u16::MAX,
                local_vars: RefCell::new(vec![]),
                operand_stack: RefCell::new(vec![]),
                pc: RefCell::new(std::usize::MAX),
                pc_offset: RefCell::new(-1),
            });
            jvm.set_current_thread(thread_from_rust.clone());
            thread_from_rust.call_stack.borrow_mut().push(new_thread_frame.clone());
            thread_from_rust.thread_object.borrow().as_ref().unwrap().run(jvm, &new_thread_frame);
            thread_from_rust.call_stack.borrow_mut().pop();
        });
        //todo this whole thread start is very racy and needs fixing
        std::mem::drop(all_threads_guard);
        thread_creation_complete_copy.wait(mutex.lock().unwrap());
    }
}

#[no_mangle]
unsafe extern "system" fn JVM_StopThread(env: *mut JNIEnv, thread: jobject, exception: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsThreadAlive(env: *mut JNIEnv, thread: jobject) -> jboolean {
    let jvm = get_state(env);
    let thread_object = JavaValue::Object(from_object(thread)).cast_thread();
    let tid = thread_object.tid();
    let mut alive = jvm.thread_state.alive_threads.read().unwrap().get(&tid)
        //todo this is jank.
        .map(|thread| !thread.interpreter_state.suspended.read().unwrap().suspended)
        .unwrap_or(false);
    alive as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_SuspendThread(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_ResumeThread(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_SetThreadPriority(env: *mut JNIEnv, thread: jobject, prio: jint) {
    //todo threads not implemented, noop
}

#[no_mangle]
unsafe extern "system" fn JVM_Yield(env: *mut JNIEnv, threadClass: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Sleep(env: *mut JNIEnv, _threadClass: jclass, millis: jlong) {
    //todo handle negative millis
    if millis <0{
        unimplemented!()
    }
    //todo figure out what threadClass is for
    sleep(Duration::from_millis(millis as u64))
}

//todo get rid of this jankyness
static mut MAIN_THREAD: Option<Arc<Object>> = None;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentThread(env: *mut JNIEnv, threadClass: jclass) -> jobject {
    match MAIN_THREAD.clone() {
        None => {
            let jvm = get_state(env);
            let frame = get_frame(env);
            let runtime_thread_class = from_jclass(threadClass);
            make_thread(&runtime_thread_class.as_runtime_class(), jvm, &frame);
            let thread_object = frame.pop().unwrap_object();
            MAIN_THREAD = thread_object.clone();
            //todo get rid of that jankyness as well:
            jvm.main_thread().thread_object.borrow_mut().replace(JavaValue::Object(MAIN_THREAD.clone()).cast_thread().into());
            to_object(thread_object)
        }
        Some(_) => {
            to_object(MAIN_THREAD.clone())
        }
    }
    //threads are not a thing atm.
    //todo
}


// static mut SYSTEM_THREAD_GROUP: Option<Arc<Object>> = None;

fn init_system_thread_group(jvm: &JVMState, frame: &StackEntry) {
    let thread_group_class = check_inited_class(jvm, &ClassName::Str("java/lang/ThreadGroup".to_string()).into(), frame.class_pointer.loader(jvm).clone());
    push_new_object(jvm, frame.clone(), &thread_group_class,None);
    let object = frame.pop();
    let init = thread_group_class
        .view()
        .lookup_method(&"<init>".to_string(),
                &MethodDescriptor { parameter_types: vec![], return_type: PType::VoidType })
        .unwrap();
    let init_i = init.method_i();
    let new_frame = StackEntry {
        class_pointer: thread_group_class.clone(),
        method_i: init_i as u16,
        local_vars: RefCell::new(vec![object.clone()]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    jvm.thread_state.system_thread_group.write().unwrap().replace(object.unwrap_object().unwrap());
    // unsafe { SYSTEM_THREAD_GROUP = object.unwrap_object(); }
    jvm.get_current_thread().call_stack.borrow_mut().push(Rc::new(new_frame));
    run_function(jvm);
    jvm.get_current_thread().call_stack.borrow_mut().pop().unwrap();
    let interpreter_state = &jvm.get_current_thread().interpreter_state;
    if interpreter_state.throw.borrow().is_some() || *interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    if *interpreter_state.function_return.borrow() {
        interpreter_state.function_return.replace(false);
    }
}

unsafe fn make_thread(runtime_thread_class: &Arc<RuntimeClass>, jvm: &JVMState, frame: &StackEntry) {
    //todo refactor this at some point
    //first create thread group
    let match_guard = jvm.thread_state.system_thread_group.read().unwrap();
    let thread_group_object = match match_guard.clone() {
        None => {
            std::mem::drop(match_guard);
            init_system_thread_group(jvm, frame);
            jvm.thread_state.system_thread_group.read().unwrap().clone()
        }
        Some(_) => jvm.thread_state.system_thread_group.read().unwrap().clone(),
    };


    let thread_class = check_inited_class(jvm, &ClassName::Str("java/lang/Thread".to_string()).into(), frame.class_pointer.loader(jvm).clone());
    // if !Arc::ptr_eq(&thread_class, &runtime_thread_class) {
    // frame.print_stack_trace();
    // }
    assert!(Arc::ptr_eq(&thread_class, &runtime_thread_class));
    push_new_object(jvm, frame.clone(), &thread_class, None);
    let object = frame.pop();
    let init = thread_class.view().lookup_method(&"<init>".to_string(), &MethodDescriptor { parameter_types: vec![], return_type: PType::VoidType }).unwrap();
    let init_i = init.method_i();
    let new_frame = StackEntry {
        class_pointer: thread_class.clone(),
        method_i: init_i as u16,
        local_vars: RefCell::new(vec![object.clone()]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    MAIN_THREAD = object.unwrap_object().clone();
    MAIN_THREAD.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert("group".to_string(), JavaValue::Object(thread_group_object));
    //for some reason the constructor doesn't handle priority.
    let NORM_PRIORITY = 5;
    MAIN_THREAD.clone().unwrap().unwrap_normal_object().fields.borrow_mut().insert("priority".to_string(), JavaValue::Int(NORM_PRIORITY));
    jvm.get_current_thread().call_stack.borrow_mut().push(Rc::new(new_frame));
    run_function(jvm);
    jvm.get_current_thread().call_stack.borrow_mut().pop();
    frame.push(JavaValue::Object(MAIN_THREAD.clone()));
    let interpreter_state = &jvm.get_current_thread().interpreter_state;
    if interpreter_state.throw.borrow().is_some() || *interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    if *interpreter_state.function_return.borrow() {
        interpreter_state.function_return.replace(false);
    }
}


#[no_mangle]
unsafe extern "system" fn JVM_Interrupt(env: *mut JNIEnv, thread: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterrupted(env: *mut JNIEnv, thread: jobject, clearInterrupted: jboolean) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_HoldsLock(env: *mut JNIEnv, threadClass: jclass, obj: jobject) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpAllStacks(env: *mut JNIEnv, unused: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetAllThreads(env: *mut JNIEnv, dummy: jclass) -> jobjectArray {
    unimplemented!()//todo already mostly implemented as part of jvmti
}

#[no_mangle]
unsafe extern "system" fn JVM_SetNativeThreadName(env: *mut JNIEnv, jthread: jobject, name: jstring) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpThreads(env: *mut JNIEnv, threadClass: jclass, threads: jobjectArray) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateValues(env: *mut JNIEnv, javaThreadState: jint) -> jintArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateNames(env: *mut JNIEnv, javaThreadState: jint, values: jintArray) -> jobjectArray {
    unimplemented!()
}
