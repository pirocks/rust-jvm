use std::ptr::null_mut;

use another_jit_vm_ir::WasException;
use jvmti_jni_bindings::{_jobject, JNIEnv, jstring};
use slow_interpreter::rust_jni::interface::string::intern_impl_unsafe;
use slow_interpreter::rust_jni::native_util::{get_interpreter_state, get_state};

#[no_mangle]
unsafe extern "system" fn JVM_InternString(env: *mut JNIEnv, str_unsafe: jstring) -> jstring {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    match intern_impl_unsafe(jvm, int_state, str_unsafe) {
        Ok(res) => res,
        Err(WasException {}) => null_mut(),
    }
}