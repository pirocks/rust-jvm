use jvmti_jni_bindings::{jboolean, jclass, JNIEnv, jobject};

#[no_mangle]
unsafe extern "system" fn JVM_DesiredAssertionStatus(env: *mut JNIEnv, unused: jclass, cls: jclass) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_AssertionStatusDirectives(env: *mut JNIEnv, unused: jclass) -> jobject {
    unimplemented!()
}
