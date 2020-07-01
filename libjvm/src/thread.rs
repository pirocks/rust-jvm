use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, Condvar, RwLock, RwLockWriteGuard};
use std::thread::sleep;
use std::time::Duration;

use nix::sys::pthread::pthread_self;
use nix::unistd::gettid;
use parking_lot::Mutex;

use classfile_view::view::ptype_view::PTypeView;
use descriptor_parser::MethodDescriptor;
use jvmti_jni_bindings::{jboolean, jclass, jint, jintArray, jlong, JNIEnv, jobject, jobjectArray, jstring};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType;
use slow_interpreter::{InterpreterState, JavaThread, JVMState, SuspendedStatus};
use slow_interpreter::interpreter::run_function;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object};
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_frame, get_state, to_object};
use slow_interpreter::stack_entry::StackEntry;
use slow_interpreter::threading::JavaThread;

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
    //todo need to assert not on main thread
    let jvm = get_state(env);
    let frame = get_frame(env);
    let thread_object = JavaValue::Object(from_object(thread)).cast_thread();
    jvm.thread_state.start_thread_from_obj(jvm,thread_object, frame.deref());
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
    if millis < 0 {
        unimplemented!()
    }
    //todo figure out what threadClass is for
    sleep(Duration::from_millis(millis as u64))
}

//todo get rid of this jankyness
// static mut MAIN_THREAD: Option<Arc<Object>> = None;

#[no_mangle]
unsafe extern "system" fn JVM_CurrentThread(env: *mut JNIEnv, threadClass: jclass) -> jobject {
    // match MAIN_THREAD.clone() {
    //     None => {
    //         let jvm = get_state(env);
    //         let frame = get_frame(env);
    //         let runtime_thread_class = from_jclass(threadClass);
    //         make_thread(&runtime_thread_class.as_runtime_class(), jvm, &frame);
    //         let thread_object = frame.pop().unwrap_object();
    //         MAIN_THREAD = thread_object.clone();
    //         todo get rid of that jankyness as well:
    // jvm.main_thread().thread_object.borrow_mut().replace(JavaValue::Object(MAIN_THREAD.clone()).cast_thread().into());
    // to_object(thread_object)
    // }
    // Some(_) => {
    //     to_object(MAIN_THREAD.clone())
    // }
    // }
}


fn init_system_thread_group(jvm: &'static JVMState, frame: &StackEntry) {
    let thread_group_class = check_inited_class(jvm, &ClassName::Str("java/lang/ThreadGroup".to_string()).into(), frame.class_pointer.loader(jvm).clone());
    push_new_object(jvm, frame.clone(), &thread_group_class, None);
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
    jvm.thread_state.get_current_thread().call_stack.borrow_mut().push(Rc::new(new_frame));
    run_function(jvm);
    jvm.thread_state.get_current_thread().call_stack.borrow_mut().pop().unwrap();
    let interpreter_state = &jvm.thread_state.get_current_thread().interpreter_state;
    if interpreter_state.throw.borrow().is_some() || *interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    if *interpreter_state.function_return.borrow() {
        interpreter_state.function_return.replace(false);
    }
}

unsafe fn make_thread(runtime_thread_class: &Arc<RuntimeClass>, jvm: &'static JVMState, frame: &StackEntry) {
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
    jvm.thread_state.get_current_thread().call_stack.borrow_mut().push(Rc::new(new_frame));
    run_function(jvm);
    jvm.thread_state.get_current_thread().call_stack.borrow_mut().pop();
    frame.push(JavaValue::Object(MAIN_THREAD.clone()));
    let interpreter_state = &jvm.thread_state.get_current_thread().interpreter_state;
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
