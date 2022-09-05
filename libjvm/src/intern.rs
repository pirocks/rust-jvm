use std::ptr::null_mut;

use jvmti_jni_bindings::{_jobject, JNIEnv, jstring};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::rust_jni::interface::{get_interpreter_state, get_state};
use slow_interpreter::rust_jni::interface::string::intern_impl_unsafe;

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match intern_impl_unsafe(jvm, int_state, str_unsafe) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            null_mut()
        }
    }
}