use std::ptr::null_mut;
use std::sync::{Arc};
use std::thread::sleep;
use std::time::Duration;

use wtf8::Wtf8Buf;

use jvmti_jni_bindings::{JAVA_THREAD_STATE_BLOCKED, JAVA_THREAD_STATE_NEW, JAVA_THREAD_STATE_RUNNABLE, JAVA_THREAD_STATE_TERMINATED, JAVA_THREAD_STATE_TIMED_WAITING, JAVA_THREAD_STATE_WAITING, jboolean, jclass, jint, jintArray, jlong, JNIEnv, jobject, jobjectArray, jstring};

use slow_interpreter::exceptions::WasException;
use slow_interpreter::java_values::{JavaValue};
use slow_interpreter::new_java_values::NewJavaValueHandle;


use slow_interpreter::rust_jni::jni_utils::{new_local_ref_public, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::stdlib::java::lang::string::JString;
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;
use slow_interpreter::threading::safepoints::Monitor2;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_StartThread(env: *mut JNIEnv, thread: jobject) {
    //todo need to assert not on main thread
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let thread_object = NewJavaValueHandle::Object(from_object_new(jvm, thread).unwrap()).cast_thread(jvm);
    jvm.thread_state.start_thread_from_obj(jvm, int_state, thread_object, false);
}

#[no_mangle]
unsafe extern "system" fn JVM_StopThread(env: *mut JNIEnv, thread: jobject, _exception: jobject) {
    //todo do not print ThreadDeath on reaching top of thread
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let target_thread = NewJavaValueHandle::Object(from_object_new(jvm, thread).unwrap()).cast_thread(jvm).get_java_thread(jvm);
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

    let java_thread = match NewJavaValueHandle::Object(from_object_new(jvm, thread).unwrap())
        .cast_thread(jvm)
        .try_get_java_thread(jvm) {
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
    let java_thread = NewJavaValueHandle::Object(from_object_new(jvm, thread).unwrap()).cast_thread(jvm).get_java_thread(jvm);
    let _ = java_thread.suspend_thread(jvm, int_state, false);
    //javadoc doesn't say anything about error handling so we just don't anything
}

#[no_mangle]
unsafe extern "system" fn JVM_ResumeThread(env: *mut JNIEnv, thread: jobject) {
    let jvm = get_state(env);
    let java_thread = NewJavaValueHandle::Object(from_object_new(jvm, thread).unwrap()).cast_thread(jvm).get_java_thread(jvm);
    let _ = java_thread.resume_thread();
    //javadoc doesn't say anything about error handling so we just don't anything
}

#[no_mangle]
unsafe extern "system" fn JVM_SetThreadPriority(_env: *mut JNIEnv, _thread: jobject, _prio: jint) {
    //todo threads not implemented, noop
}

#[no_mangle]
unsafe extern "system" fn JVM_Yield(_env: *mut JNIEnv, _threadClass: jclass) {
    std::thread::yield_now();
    //todo actually do something here maybe
}

#[no_mangle]
unsafe extern "system" fn JVM_Sleep(_env: *mut JNIEnv, _threadClass: jclass, millis: jlong) {
    //todo handle negative millis
    if millis < 0 {
        unimplemented!()
    }
    //todo figure out what threadClass is for
    //todo this should sleep mechanism in safepoint
    sleep(Duration::from_millis(millis as u64))
}

#[no_mangle]
unsafe extern "system" fn JVM_CurrentThread(env: *mut JNIEnv, _threadClass: jclass) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let current_thread = jvm.thread_state.get_current_thread();
    let current_thread_allocated_object_handle = current_thread.thread_object().object();
    // assert_eq!(current_thread_allocated_object_handle.as_allocated_obj().runtime_class(jvm).cpdtype().jvm_representation(&jvm.string_pool), "Ljava/lang/Thread;");
    let res = new_local_ref_public_new(current_thread_allocated_object_handle.as_allocated_obj().into(), int_state);
    assert_ne!(res, null_mut());
    res
}

#[no_mangle]
unsafe extern "system" fn JVM_Interrupt(env: *mut JNIEnv, thread: jobject) {
    let jvm = get_state(env);
    let thread_object = from_object_new(jvm, thread).unwrap().new_java_value_handle().cast_thread(jvm);
    let thread = thread_object.get_java_thread(jvm);
    thread.interrupt_thread();
}

#[no_mangle]
unsafe extern "system" fn JVM_IsInterrupted(env: *mut JNIEnv, thread: jobject, _clearInterrupted: jboolean) -> jboolean {
    //todo clearInterrupted??
    let jvm = get_state(env);
    let thread_object = from_object_new(jvm, thread).unwrap().new_java_value_handle().cast_thread(jvm);
    let thread = thread_object.get_java_thread(jvm);
    thread.safepoint_state.is_interrupted() as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_HoldsLock(env: *mut JNIEnv, _threadClass: jclass, obj: jobject) -> jboolean {
    let jvm = get_state(env);
    let monitor: Arc<Monitor2> = jvm.monitor_for(from_object_new(jvm, obj).unwrap().ptr().as_ptr());
    monitor.this_thread_holds_lock(jvm) as jboolean
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpAllStacks(_env: *mut JNIEnv, _unused: jclass) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetAllThreads(env: *mut JNIEnv, _dummy: jclass) -> jobjectArray {
    //the dummy appears b/c stuff gets called from static native fucntion in jni, and someone didn't want to get rid of param and just have a direct function pointer
    let jvm = get_state(env);
    let _int_state = get_interpreter_state(env);
    let _jobjects = jvm
        .thread_state
        .get_all_alive_threads()
        .into_iter()
        .map(|_java_thread| {
            JavaValue::Object(todo!() /*java_thread.try_thread_object().map(|jthread| jthread.object())*/)
        })
        .collect::<Vec<_>>();
    let _object_array = todo!()/*JavaValue::new_vec_from_vec(jvm, jobjects, CClassName::thread().into()).unwrap_object()*/;
    new_local_ref_public(todo!()/*object_array*/, _int_state)
}

#[no_mangle]
unsafe extern "system" fn JVM_SetNativeThreadName(env: *mut JNIEnv, _jthread: jobject, _name: jstring) {
    let _jvm = get_state(env);
    let _int_state = get_interpreter_state(env);
    todo!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DumpThreads(_env: *mut JNIEnv, _threadClass: jclass, _threads: jobjectArray) -> jobjectArray {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateValues(env: *mut JNIEnv, javaThreadState: jint) -> jintArray {
    let _jvm = get_state(env);
    let _int_state = get_interpreter_state(env);
    let _names = match javaThreadState as u32 {
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
        _ => return null_mut(),
    }
        .into_iter()
        .map(|int| JavaValue::Int(int))
        .collect::<Vec<_>>();
}

#[no_mangle]
unsafe extern "system" fn JVM_GetThreadStateNames(env: *mut JNIEnv, javaThreadState: jint, _values: jintArray) -> jobjectArray {
    match GetThreadStateNames_impl(env, javaThreadState) {
        Ok(res) => res,
        Err(_) => null_mut(),
    }
}

unsafe fn GetThreadStateNames_impl<'gc>(env: *mut JNIEnv, javaThreadState: i32) -> Result<jobject, WasException<'gc>> {
    //don't check values for now. They should be correct and from JVM_GetThreadStateValues
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let _names = match javaThreadState as u32 {
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
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("WAITING.OBJECT_WAIT"))?, JString::from_rust(jvm, int_state, Wtf8Buf::from_str("WAITING.PARKED"))?]
        }
        JAVA_THREAD_STATE_TIMED_WAITING => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TIMED_WAITING.SLEEPING"))?, JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TIMED_WAITING.OBJECT_WAIT"))?, JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TIMED_WAITING.PARKED"))?]
        }
        JAVA_THREAD_STATE_TERMINATED => {
            vec![JString::from_rust(jvm, int_state, Wtf8Buf::from_str("TERMINATED"))?]
        }
        _ => return Ok(null_mut()),
    }
        .into_iter()
        .map(|jstring| jstring.java_value())
        .collect::<Vec<_>>();
    let _res = todo!()/*JavaValue::new_vec_from_vec(jvm, names, CClassName::string().into()).unwrap_object()*/;
    Ok(new_local_ref_public(todo!()/*res*/, int_state))
}

#[no_mangle]
unsafe extern "system" fn JVM_CountStackFrames(_env: *mut JNIEnv, _thread: jobject) -> jint {
    todo!()
}