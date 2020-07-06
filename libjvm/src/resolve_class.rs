use jvmti_jni_bindings::{jclass, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_ResolveClass(env: *mut JNIEnv, cls: jclass) {
    unimplemented!()
}
