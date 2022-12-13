use jvmti_jni_bindings::{jclass, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_ResolveClass(_env: *mut JNIEnv, _cls: jclass) {
    unimplemented!()
}
