use std::mem::transmute;

use jvmti_jni_bindings::{jint, JNIEnv, jobject};

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let _64bit: u64 = obj as u64;
    ((_64bit >> 32) as i32 | _64bit as i32)
}
