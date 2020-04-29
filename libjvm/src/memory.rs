use jvmti_jni_bindings::jlong;

#[no_mangle]
unsafe extern "system" fn JVM_TotalMemory() -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FreeMemory() -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MaxMemory() -> jlong {
    unimplemented!()
}

