use jvmti_jni_bindings::{jboolean, jint, jlong, jmethodID, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_DTraceGetVersion(env: *mut JNIEnv) -> jint {
    -1
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsProbeEnabled(env: *mut JNIEnv, method: jmethodID) -> jboolean {
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceDispose(env: *mut JNIEnv, activation_handle: jlong) {

}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsSupported(env: *mut JNIEnv) -> jboolean {
    u8::from(false)
}

