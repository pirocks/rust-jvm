use jni_bindings::jlong;

#[no_mangle]
unsafe extern "system" fn JVM_GC() {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_MaxObjectInspectionAge() -> jlong {
    unimplemented!()
}
