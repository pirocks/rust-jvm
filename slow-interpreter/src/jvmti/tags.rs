use std::intrinsics::transmute;

use jvmti_jni_bindings::{jlong, jobject, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_NOT_FOUND, jvmtiError_JVMTI_ERROR_NULL_POINTER};

use crate::jvmti::get_state;

pub unsafe extern "C" fn get_tag(env: *mut jvmtiEnv, object: jobject, tag_ptr: *mut jlong) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetTag");
    if object == std::ptr::null_mut() {
        return jvmtiError_JVMTI_ERROR_NULL_POINTER;
    }
    let res = match jvm.jvmti_state.as_ref().unwrap().tags.read().unwrap().get(transmute(object)) {
        None => { jvmtiError_JVMTI_ERROR_NOT_FOUND }
        Some(tag) => {
            tag_ptr.write(*tag);
            jvmtiError_JVMTI_ERROR_NONE
        }
    };
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetTag");
    res
}

pub unsafe extern "C" fn set_tag(env: *mut jvmtiEnv, object: jobject, tag: jlong) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SetTag");
    if object == std::ptr::null_mut() {
        return jvmtiError_JVMTI_ERROR_NULL_POINTER;
    }
    jvm.jvmti_state.as_ref().unwrap().tags.write().unwrap().insert(transmute(object), tag);
    jvm.tracing.trace_jdwp_function_exit(jvm, "SetTag");
    jvmtiError_JVMTI_ERROR_NONE
}