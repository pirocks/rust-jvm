use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::null_mut;
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
use slow_interpreter::{InterpreterState, InterpreterStateGuard, JVMState, SuspendedStatus};
use slow_interpreter::interpreter::run_function;
use slow_interpreter::interpreter_util::{check_inited_class, push_new_object};
use slow_interpreter::java::lang::thread_group::JThreadGroup;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::stack_entry::StackEntry;
use slow_interpreter::threading::JavaThread;

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
    //todo need to assert not on main thread
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let thread_object = JavaValue::Object(from_object(thread)).cast_thread();
    jvm.thread_state.start_thread_from_obj(jvm, thread_object, false);
}

#[no_mangle]
unsafe extern "system" fn JVM_StopThread(env: *mut JNIEnv, thread: jobject, exception: jobject) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsThreadAlive(env: *mut JNIEnv, thread: jobject) -> jboolean {
    let jvm = get_state(env);

    let int_state = get_interpreter_state(env);
    // int_state.print_stack_trace();
    let java_thread = match JavaValue::Object(from_object(thread)).cast_thread().try_get_java_thread(jvm) {
        None => return 0 as jboolean,
        Some(jt) => jt,
    };
    // assert!(!java_thread.invisible_to_java);
    let alive = java_thread.is_alive();
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


#[no_mangle]
unsafe extern "system" fn JVM_CurrentThread(env: *mut JNIEnv, threadClass: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let current_thread = jvm.thread_state.get_current_thread();
    // if current_thread.invisible_to_java {
    //     int_state.print_stack_trace();
    // }
    // assert!(!current_thread.invisible_to_java);
    let res = new_local_ref_public(current_thread.thread_object().object().into(), int_state);
    assert_ne!(res, null_mut());
    res
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
