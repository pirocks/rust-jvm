use jni_bindings::{JNIEnv, jclass};

#[no_mangle]
unsafe extern "system" fn JVM_ResolveClass(env: *mut JNIEnv, cls: jclass) {
    unimplemented!()
}
