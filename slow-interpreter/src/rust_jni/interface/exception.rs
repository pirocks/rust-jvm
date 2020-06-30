use jvmti_jni_bindings::{jthrowable, JNIEnv};
use crate::rust_jni::native_util::get_state;

pub unsafe extern "C" fn exception_occured(_env: *mut JNIEnv) -> jthrowable {
    //exceptions don't happen yet todo
    std::ptr::null_mut()
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv){
    //todo not implemented yet
    let jvm = get_state(env);
    assert!(jvm.thread_state.get_current_thread().interpreter_state.throw.read().unwrap().is_none());
}
