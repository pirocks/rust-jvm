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
use wtf8::Wtf8Buf;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{_jobject, JAVA_THREAD_STATE_BLOCKED, JAVA_THREAD_STATE_NEW, JAVA_THREAD_STATE_RUNNABLE, JAVA_THREAD_STATE_TERMINATED, JAVA_THREAD_STATE_TIMED_WAITING, JAVA_THREAD_STATE_WAITING, jboolean, jclass, jint, jintArray, jlong, JNIEnv, jobject, jobjectArray, jstring, JVM_Available};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::ptype::PType;
use slow_interpreter::interpreter::{run_function, WasException};
use slow_interpreter::interpreter_util::push_new_object;
use slow_interpreter::invoke_interface::get_env;
use slow_interpreter::java::lang::string::JString;
use slow_interpreter::java::lang::thread::JThread;
use slow_interpreter::java::lang::thread_group::JThreadGroup;
use slow_interpreter::java_values::{JavaValue, Object};
use slow_interpreter::runtime_class::RuntimeClass;
use slow_interpreter::rust_jni::interface::local_frame::{new_local_ref, new_local_ref_public};
use slow_interpreter::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state, to_object};
use slow_interpreter::stack_entry::StackEntry;
use slow_interpreter::threading::{JavaThread, SuspendError};
use slow_interpreter::threading::safepoints::Monitor2;

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
    //todo need to assert not on main thread
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let thread_object = JavaValue::Object(from_object(jvm, thread)).cast_thread();
    jvm.thread_state.start_thread_from_obj(jvm, int_state, thread_object, false);
}

#[no_mangle]
unsafe extern "system" fn JVM_StopThread(env: *mut JNIEnv, thread: jobject, exception: jobject) {
    //todo do not print ThreadDeath on reaching top of thread
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let target_thread = JavaValue::Object(from_object(jvm, thread)).cast_thread().get_java_thread(jvm);
    if let Err(_err) = target_thread.suspend_thread(jvm, int_state, false) {
        // it appears we should ignore any errors here.
        //todo unclear what happens when one calls start on stopped thread. javadoc says terminate immediately, but what does that mean/ do we do this
    }
    //todo throw?
    // target_thread.interpreter_state.write().unwrap().throw = from_jclass(jvm,exception); //todo use set_throw? //todo handle npe
    todo!()
}

#[no_mangle]
unsafe extern "system" fn JVM_IsThreadAlive(env: *mut JNIEnv, thread: jobject) -> jboolean {
    let jvm = get_state(env);

    let int_state = get_interpreter_state(env);
    let java_thread = match JavaValue::Object(from_object(jvm, thread)).cast_thread().try_get_java_thread(jvm) {
        None => return 0 as jboolean,
        Some(jt) => jt,
    };
    let alive = java_thread.is_alive();
    alive as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_SuspendThread(env: *mut JNIEnv, thread: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let java_thread = JavaValue::Object(from_object(jvm, thread)).cast_thread().get_java_thread(jvm);
    let _ = java_thread.suspend_thread(jvm, int_state, false);
    //javadoc doesn't say anything about error handling so we just don't anything
}

#[no_mangle]
unsafe extern "system" fn JVM_ResumeThread(env: *mut JNIEnv, thread: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let java_thread = JavaValue::Object(from_object(jvm, thread)).cast_thread().get_java_thread(jvm);
    let _ = java_thread.resume_thread();
    //javadoc doesn't say anything about error handling so we just don't anything
}

#[no_mangle]
unsafe extern "system" fn JVM_SetThreadPriority(env: *mut JNIEnv, thread: jobject, prio: jint) {
    //todo threads not implemented, noop
}

#[no_mangle]
unsafe extern "system" fn JVM_Yield(env: *mut JNIEnv, threadClass: jclass) {
    //todo actually do something here maybe
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
    let res = new_local_ref_public(current_thread.thread_object().object().into(), int_state);
    assert_ne!(res, null_mut());
    res
}


#[no_mangle]
unsafe extern "system" fn JVM_Interrupt(env: *mut JNIEnv, thread: jobject) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    todo!("This seems to need signals or some shit. Seems hard to implement")
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterrupted(env: *mut JNIEnv, thread: jobject, clearInterrupted: jboolean) -> jboolean {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let thread_object = JavaValue::Object(from_object(jvm, thread)).cast_thread();
    let thread = thread_object.get_java_thread(jvm);
    let guard = thread.thread_status.read().unwrap();
    guard.interrupted as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_HoldsLock(env: *mut JNIEnv, threadClass: jclass, obj: jobject) -> jboolean {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let monitor: Arc<Monitor2> = JavaValue::Object(from_object(jvm, obj)).unwrap_normal_object().monitor.clone();
    monitor.this_thread_holds_lock(jvm) as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpAllStacks(env: *mut JNIEnv, unused: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetAllThreads(env: *mut JNIEnv, _dummy: jclass) -> jobjectArray {
    //the dummy appears b/c stuff gets called from static native fucntion in jni, and someone didn't want to get rid of param and just have a direct function pointer
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let jobjects = jvm.thread_state.get_all_alive_threads().into_iter().map(|java_thread| JavaValue::Object(todo!()/*java_thread.try_thread_object().map(|jthread| jthread.object())*/)).collect::<Vec<_>>();
    let object_array = JavaValue::new_vec_from_vec(jvm, jobjects, CClassName::thread().into()).unwrap_object();
    new_local_ref_public(object_array, int_state)
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
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let names = match javaThreadState as u32 {
        JAVA_THREAD_STATE_NEW => {
            vec![todo!()]
        }
        JAVA_THREAD_STATE_RUNNABLE => {
            vec![todo!()]
        }
        JAVA_THREAD_STATE_BLOCKED => {
            vec![todo!()]
        }
        JAVA_THREAD_STATE_WAITING => {
            vec![todo!()]
        }
        JAVA_THREAD_STATE_TIMED_WAITING => {
            vec![todo!()]
        }
        JAVA_THREAD_STATE_TERMINATED => {
            vec![todo!()]
        }
        _ => return null_mut()
    }.into_iter().map(|int| JavaValue::Int(int)).collect::<Vec<_>>();
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateNames(env: *mut JNIEnv, javaThreadState: jint, _values: jintArray) -> jobjectArray {
    match GetThreadStateNames_impl(env, javaThreadState) {
        Ok(res) => res,
        Err(_) => null_mut()
    }
}

unsafe fn GetThreadStateNames_impl(env: *mut JNIEnv, javaThreadState: i32) -> Result<jobject, WasException> {
    //don't check values for now. They should be correct and from JVM_GetThreadStateValues
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let names = match javaThreadState as u32 {
        JAVA_THREAD_STATE_NEW => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("NEW"))?]
        }
        JAVA_THREAD_STATE_RUNNABLE => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("RUNNABLE"))?]
        }
        JAVA_THREAD_STATE_BLOCKED => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("BLOCKED"))?]
        }
        JAVA_THREAD_STATE_WAITING => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("WAITING.OBJECT_WAIT"))?,
                 JString::from_rust(jvm, int_state, Wtf8Buf::from_str("WAITING.PARKED"))?]
        }
        JAVA_THREAD_STATE_TIMED_WAITING => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TIMED_WAITING.SLEEPING"))?,
                 JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TIMED_WAITING.OBJECT_WAIT"))?,
                 JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TIMED_WAITING.PARKED"))?]
        }
        JAVA_THREAD_STATE_TERMINATED => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TERMINATED"))?]
        }
        _ => return Ok(null_mut())
    }.into_iter().map(|jstring| jstring.java_value()).collect::<Vec<_>>();
    let res = JavaValue::new_vec_from_vec(jvm, names, CClassName::string().into()).unwrap_object();
    Ok(new_local_ref_public(res, int_state))
}


#[no_mangle]
unsafe extern "system" fn JVM_CountStackFrames(env: *mut JNIEnv, thread: jobject) -> jint {
    todo!()
}
