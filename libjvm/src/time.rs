use jni_bindings::{jlong, jclass, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_CurrentTimeMillis(env: *mut JNIEnv, ignored: jclass) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_NanoTime(env: *mut JNIEnv, ignored: jclass) -> jlong {
    unimplemented!()
}
