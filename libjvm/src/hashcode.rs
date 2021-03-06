use std::mem::transmute;
use std::sync::Arc;

use jvmti_jni_bindings::{jint, JNIEnv, jobject};
use slow_interpreter::rust_jni::native_util::from_object;

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let object = from_object(obj);
    if object.is_none() {
        return 0;
    }
    // todo handle npe, though invoke virtual should handle
    let _64bit: u64 = Arc::as_ptr(&object.unwrap()) as u64;
    let hashcode = ((_64bit >> 32) as i32 | _64bit as i32);
    hashcode
}
