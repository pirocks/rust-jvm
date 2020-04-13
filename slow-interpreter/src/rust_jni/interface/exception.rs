use jni_bindings::{jthrowable, JNIEnv};
use crate::rust_jni::native_util::{get_frame, get_state};

pub unsafe extern "C" fn exception_occured(_env: *mut JNIEnv) -> jthrowable {
    //exceptions don't happen yet todo
    std::ptr::null_mut()
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv){
    //todo not implemented yet
    let state = get_state(env);
    assert!(state.get_current_thread().interpreter_state.throw.borrow().is_none());
}
