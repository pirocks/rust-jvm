use jni_bindings::{jboolean, JNIEnv, jint, jlong, jmethodID};

#[no_mangle]
unsafe extern "system" fn JVM_DTraceGetVersion(env: *mut JNIEnv) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsProbeEnabled(env: *mut JNIEnv, method: jmethodID) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceDispose(env: *mut JNIEnv, activation_handle: jlong) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsSupported(env: *mut JNIEnv) -> jboolean {
    unimplemented!()
}

