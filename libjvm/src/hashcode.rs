use jni_bindings::{jint, jobject, JNIEnv};
use std::mem::transmute;

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let _64bit: u64 = transmute(obj);
    ((_64bit >> 32) as i32 | _64bit as i32)
}
