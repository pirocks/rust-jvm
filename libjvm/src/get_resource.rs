use jni_bindings::{jobject, JNIEnv, jintArray, jobjectArray};
use slow_interpreter::rust_jni::native_util::to_object;

#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCacheURLs(env: *mut JNIEnv, loader: jobject) -> jobjectArray {
    to_object(None)//todo not implemented for now

}


#[no_mangle]
unsafe extern "system" fn JVM_GetResourceLookupCache(
    env: *mut JNIEnv,
    loader: jobject,
    resource_name: *const ::std::os::raw::c_char,
) -> jintArray {
    unimplemented!()
}
