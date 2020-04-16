use jvmti_bindings::{jvmtiEnv, jobject, jlong, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_NULL_POINTER, jvmtiError_JVMTI_ERROR_NOT_FOUND};
use crate::jvmti::get_state;
use std::intrinsics::transmute;

pub unsafe extern "C" fn get_tag(env: *mut jvmtiEnv, object: jobject, tag_ptr: *mut jlong) -> jvmtiError{
    // tag_ptr.write(transmute(object));
    // jvmtiError_JVMTI_ERROR_NONE
    let jvm = get_state(env);
    if object == std::ptr::null_mut(){
        return  jvmtiError_JVMTI_ERROR_NULL_POINTER
    }
    match jvm.jvmti_state.tags.read().unwrap().get(transmute(object)){
        None => {jvmtiError_JVMTI_ERROR_NOT_FOUND},
        Some(tag) => {
            tag_ptr.write(*tag);
            jvmtiError_JVMTI_ERROR_NONE
        },
    }
}

pub unsafe extern "C" fn set_tag(env: *mut jvmtiEnv, object: jobject, tag: jlong) -> jvmtiError{
    let jvm = get_state(env);
    if object == std::ptr::null_mut(){
        return jvmtiError_JVMTI_ERROR_NULL_POINTER
    }
    jvm.jvmti_state.tags.write().unwrap().insert(transmute(object),tag);
    jvmtiError_JVMTI_ERROR_NONE
}