use jvmti_jni_bindings::{jboolean, jint, JNIEnv, jthrowable};
use crate::NewAsObjectOrJavaValue;
use crate::rust_jni::interface::{get_interpreter_state, get_state};
use crate::rust_jni::interface::jni::get_throw;

use crate::rust_jni::interface::local_frame::{new_local_ref_public_new};

pub unsafe extern "C" fn exception_occured(env: *mut JNIEnv) -> jthrowable {
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let throw = throw.as_mut().map(|obj|obj.exception_obj.full_object_ref());
    new_local_ref_public_new(throw, int_state)
}

pub unsafe extern "C" fn exception_clear(env: *mut JNIEnv) {
    // let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    *throw = None;
}

pub unsafe extern "C" fn exception_check(env: *mut JNIEnv) -> jboolean {
    let throw = get_throw(env);
    u8::from(throw.is_some())
}

pub unsafe extern "C" fn throw(env: *mut JNIEnv, obj: jthrowable) -> jint {
    let jvm = get_state(env);
    let interpreter_state = get_interpreter_state(env);
    todo!()/*interpreter_state.set_throw(from_object_new(jvm, obj));
    0 as jint*/
}