use jvmti_jni_bindings::{JNIEnv, jthrowable};

use crate::rust_jni::native_util::{get_interpreter_state, to_object};

pub unsafe extern "C" fn exception_occured(env: *mut JNIEnv) -> jthrowable {
    let int_state = get_interpreter_state(env);
    to_object(int_state.throw().clone())
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv) {
    let int_state = get_interpreter_state(env);
    *int_state.throw_mut() = None;
    assert!(int_state.throw_mut().is_none());
}
