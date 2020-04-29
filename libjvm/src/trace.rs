use jvmti_jni_bindings::jboolean;

#[no_mangle]
unsafe extern "system" fn JVM_TraceInstructions(on: jboolean) {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_TraceMethodCalls(on: jboolean) {
    unimplemented!()
}
