use jvmti_jni_bindings::{jboolean, jint, jlong, jmethodID, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_DTraceGetVersion(_env: *mut JNIEnv) -> jint {
    -1
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsProbeEnabled(_env: *mut JNIEnv, _method: jmethodID) -> jboolean {
    u8::from(false)
}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceDispose(_env: *mut JNIEnv, _activation_handle: jlong) {}

#[no_mangle]
unsafe extern "system" fn JVM_DTraceIsSupported(_env: *mut JNIEnv) -> jboolean {
    u8::from(false)
}
