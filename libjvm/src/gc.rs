use jvmti_jni_bindings::{jboolean, jlong};

#[no_mangle]
unsafe extern "system" fn JVM_GC() {
    // todo!("Blocking on GC impl")
}

#[no_mangle]
unsafe extern "system" fn JVM_MaxObjectInspectionAge() -> jlong {
    todo!("Blocking on GC impl")
}


#[no_mangle]
unsafe extern "system" fn JVM_IsUseContainerSupport() -> jboolean {
    jboolean::from(false)
}