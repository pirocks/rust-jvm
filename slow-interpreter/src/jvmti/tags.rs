use jvmti_bindings::{jvmtiEnv, jobject, jlong, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
use std::intrinsics::transmute;

pub unsafe extern "C" fn get_tag(env: *mut jvmtiEnv, object: jobject, tag_ptr: *mut jlong) -> jvmtiError{
    tag_ptr.write(transmute(object));
    jvmtiError_JVMTI_ERROR_NONE
}
