use jni_bindings::{jobject, JNIEnv, jintArray, jobjectArray};

#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCacheURLs(env: *mut JNIEnv, loader: jobject) -> jobjectArray {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCache(
    env: *mut JNIEnv,
    loader: jobject,
    resource_name: *const ::std::os::raw::c_char,
) -> jintArray {
    unimplemented!()
}
