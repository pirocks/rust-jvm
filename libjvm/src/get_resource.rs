use std::ffi::{c_char};
use std::ptr::null_mut;

use jvmti_jni_bindings::{jintArray, JNIEnv, jobject, jobjectArray};

//so it appears hotspot implements both of these as null.

#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCacheURLs(_env: *mut JNIEnv, _loader: jobject) -> jobjectArray {
    null_mut()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCache(_env: *mut JNIEnv, _loader: jobject, _resource_name: *const c_char) -> jintArray {
    null_mut()
}