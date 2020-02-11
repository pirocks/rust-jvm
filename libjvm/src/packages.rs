use jni_bindings::{jstring, JNIEnv, jobjectArray};

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackage(env: *mut JNIEnv, name: jstring) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackages(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}
