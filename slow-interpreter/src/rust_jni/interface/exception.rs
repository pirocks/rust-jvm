use jvmti_jni_bindings::{jboolean, jint, JNIEnv, jthrowable};

use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state};

pub unsafe extern "C" fn exception_occured(env: *mut JNIEnv) -> jthrowable {
    let int_state = get_interpreter_state(env);
    new_local_ref_public(int_state.throw().clone(), int_state)
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv) {
    let int_state = get_interpreter_state(env);
    int_state.set_throw(None);
    assert!(int_state.throw().is_none());
}

pub unsafe extern "C" fn exception_check(_env: *mut JNIEnv) -> jboolean {
    false as jboolean//todo exceptions are not needed for hello world so if we encounter an exception we just pretend it didn't happen
}


pub unsafe extern "C" fn throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
    // let jvm = get_state(env);
    let interpreter_state = get_interpreter_state(env);
    interpreter_state.set_throw(from_object(obj));
    0 as jint
}
