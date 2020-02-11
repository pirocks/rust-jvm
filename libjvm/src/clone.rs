use jni_bindings::{jobject, JNIEnv};

#[no_mangle]
unsafe extern "system" fn JVM_Clone(env: *mut JNIEnv, obj: jobject) -> jobject {
    unimplemented!()
}
