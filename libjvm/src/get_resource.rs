use std::ffi::CStr;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jintArray, JNIEnv, jobject, jobjectArray};
use slow_interpreter::rust_jni::interface::local_frame::new_local_ref_public;
use slow_interpreter::rust_jni::native_util::{get_interpreter_state, to_object};

//so it appears hotspot implements both of these as null.

#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCacheURLs(
    env: *mut JNIEnv,
    loader: jobject,
) -> jobjectArray {
    null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCache(
    env: *mut JNIEnv,
    loader: jobject,
    resource_name: *const ::std::os::raw::c_char,
) -> jintArray {
    dbg!(CStr::from_ptr(resource_name).to_str().unwrap());
    null_mut()
}