use jvmti_jni_bindings::{jint, JNIEnv, jobject};

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(_env: *mut JNIEnv, obj: jobject) -> jint {
    let _64bit = obj as u64;
    let hashcode = (_64bit >> 32) as u32 ^ _64bit as u32;
    hashcode as jint
    //don't change without also changing intrinsics setup.
}