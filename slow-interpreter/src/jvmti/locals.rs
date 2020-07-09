use std::ptr::null_mut;

use jvmti_jni_bindings::{jdouble, jfloat, jint, jlong, jobject, jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::java_values::JavaValue;
use crate::jvmti::get_state;
use crate::rust_jni::native_util::{from_object, to_object};

pub unsafe extern "C" fn get_local_object(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jobject) -> jvmtiError {
    let var = get_local_t(env, thread, depth, slot);
    match var {
        JavaValue::Top => value_ptr.write(null_mut()),
        _ => {
            value_ptr.write(to_object(var.unwrap_object()));
        }
    }
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_local_int(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jint) -> jvmtiError {
    let var = get_local_t(env, thread, depth, slot);
    value_ptr.write(var.unwrap_int());
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_local_float(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jfloat) -> jvmtiError {
    let var = get_local_t(env, thread, depth, slot);
    value_ptr.write(var.unwrap_float());
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_local_double(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jdouble) -> jvmtiError {
    let var = get_local_t(env, thread, depth, slot);
    value_ptr.write(var.unwrap_double());
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_local_long(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value_ptr: *mut jlong) -> jvmtiError {
    let var = get_local_t(env, thread, depth, slot);
    value_ptr.write(var.unwrap_long());
    jvmtiError_JVMTI_ERROR_NONE
}


unsafe fn get_local_t(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint) -> JavaValue {
    let jthread = JavaValue::Object(from_object(thread)).cast_thread();
    let jvm = get_state(env);
    let java_thread = jthread.get_java_thread(jvm);
    let call_stack = &java_thread.interpreter_state.read().unwrap().call_stack;
    let stack_frame = &call_stack[depth as usize];
    let var = stack_frame.local_vars[slot as usize].clone();
    var
}
