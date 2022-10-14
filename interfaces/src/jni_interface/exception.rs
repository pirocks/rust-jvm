use jvmti_jni_bindings::{jboolean, jint, JNIEnv, jthrowable};

use slow_interpreter::new_java_values::owned_casts::OwnedCastAble;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::rust_jni::jni_utils::{get_throw, new_local_ref_public_new};
use slow_interpreter::rust_jni::native_util::from_object_new;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state};
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;

pub unsafe extern "C" fn exception_occured(env: *mut JNIEnv) -> jthrowable {
    let int_state = get_interpreter_state(env);
    let throw = get_throw(env);
    let throw = throw.as_mut().map(|obj| obj.exception_obj.full_object_ref());
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
    let throw = get_throw(env);
    *throw = Some(WasException {
        exception_obj: from_object_new(jvm, obj).unwrap().cast_throwable()
    });
    0 as jint
}