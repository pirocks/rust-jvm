use jni_bindings::{jint, jobject, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_FillInStackTrace(env: *mut JNIEnv, throwable: jobject) {
    //todo no stacktraces for now.
//    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceDepth(env: *mut JNIEnv, throwable: jobject) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_GetStackTraceElement(env: *mut JNIEnv, throwable: jobject, index: jint) -> jobject {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_CountStackFrames(env: *mut JNIEnv, thread: jobject) -> jint {
    unimplemented!()
}
