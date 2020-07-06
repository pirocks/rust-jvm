use jvmti_jni_bindings::{JNIEnv, jobjectArray, jstring};

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackage(env: *mut JNIEnv, name: jstring) -> jstring {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetSystemPackages(env: *mut JNIEnv) -> jobjectArray {
    unimplemented!()
}
