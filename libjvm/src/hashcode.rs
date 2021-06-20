use std::mem::transmute;
use std::sync::Arc;

use jvmti_jni_bindings::{jint, JNIEnv, jobject};
use slow_interpreter::rust_jni::native_util::{from_object, get_interpreter_state, get_state};
use slow_interpreter::utils::throw_npe;

#[no_mangle]
unsafe extern "system" fn JVM_IHashCode(env: *mut JNIEnv, obj: jobject) -> jint {
    let int_state = get_interpreter_state(env);
    let jvm = get_state(env);
    let object = from_object(jvm, obj);
    if object.is_none() {
        return throw_npe(jvm, int_state);
    }
    let _64bit: u64 = object.unwrap().raw_ptr_usize() as u64;
    let hashcode = ((_64bit >> 32) as i32 | _64bit as i32);
    hashcode
}
