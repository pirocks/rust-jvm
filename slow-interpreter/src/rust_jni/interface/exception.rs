use jvmti_jni_bindings::{JNIEnv, jthrowable};

use crate::rust_jni::native_util::get_interpreter_state;

pub unsafe extern "C" fn exception_occured(_env: *mut JNIEnv) -> jthrowable {
    //exceptions don't happen yet todo
    std::ptr::null_mut()
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv) {
    //todo not implemented yet
    // let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    assert!(int_state.throw_mut().is_none());
}
