use jvmti_jni_bindings::{JNIEnv, jthrowable};

use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{get_interpreter_state, to_object};

pub unsafe extern "C" fn exception_occured(env: *mut JNIEnv) -> jthrowable {
    let int_state = get_interpreter_state(env);
    new_local_ref_public(int_state.throw().clone(), int_state)
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv) {
    let int_state = get_interpreter_state(env);
    int_state.set_throw(None);
    assert!(int_state.throw().is_none());
}
