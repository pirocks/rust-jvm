use jvmti_jni_bindings::{jboolean, jint, JNIEnv, jthrowable};

use crate::rust_jni::interface::local_frame::{new_local_ref_public_new};
use crate::rust_jni::native_util::{get_interpreter_state, get_state};

pub unsafe extern "C" fn exception_occured(env: *mut JNIEnv) -> jthrowable {
    let int_state = get_interpreter_state(env);
    let throw_owned = int_state.throw().map(|obj|obj.duplicate_discouraged());
    let throw = throw_owned.as_ref().map(|obj|obj.as_allocated_obj());
    new_local_ref_public_new(throw, int_state)
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv) {
    let int_state = get_interpreter_state(env);
    int_state.set_throw(None);
    assert!(int_state.throw().is_none());
}

pub unsafe extern "C" fn exception_check(env: *mut JNIEnv) -> jboolean {
    let int_state = get_interpreter_state(env);
    u8::from(int_state.throw().is_some())
}

pub unsafe extern "C" fn throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
    let jvm = get_state(env);
    let interpreter_state = get_interpreter_state(env);
    interpreter_state.set_throw(todo!()/*from_object(jvm, obj)*/);
    0 as jint
}