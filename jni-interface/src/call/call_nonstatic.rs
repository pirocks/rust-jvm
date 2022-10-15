use std::ffi::VaList;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort, jvalue};
use slow_interpreter::exceptions::WasException;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;

use slow_interpreter::rust_jni::jni_utils::new_local_ref_public_new;
use crate::call::{call_nonstatic_method, VarargProvider};

pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
    let res = match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    }
        .unwrap()
        .unwrap_object();
    let interpreter_state = get_interpreter_state(env);
    new_local_ref_public_new(res.as_ref().map(|handle| handle.as_allocated_obj()), interpreter_state)
}

pub unsafe extern "C" fn call_void_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) {
    let res = match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return;
        }
    };
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_byte_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jbyte {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jbyte::MAX;
        }
    }
        .unwrap()
        .unwrap_byte_strict()
}

pub unsafe extern "C" fn call_boolean_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jboolean {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jboolean::MAX;
        }
    }
        .unwrap()
        .unwrap_bool_strict()
}

pub unsafe extern "C" fn call_short_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jshort {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jshort::MAX;
        }
    }
        .unwrap()
        .unwrap_short_strict()
}

pub unsafe extern "C" fn call_char_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jchar {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jchar::MAX;
        }
    }
        .unwrap()
        .unwrap_char_strict()
}

pub unsafe extern "C" fn call_int_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jint {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jint::MAX;
        }
    }
        .unwrap()
        .unwrap_int_strict()
}

pub unsafe extern "C" fn call_float_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jfloat {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jfloat::MAX;
        }
    }
        .unwrap()
        .unwrap_float_strict()
}

pub unsafe extern "C" fn call_double_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jdouble {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jdouble::MAX;
        }
    }
        .unwrap()
        .unwrap_double_strict()
}

pub unsafe extern "C" fn call_long_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jlong {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jlong::MAX;
        }
    }
        .unwrap()
        .unwrap_long_strict()
}

pub unsafe extern "C" fn call_object_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jobject {
    let res = match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    }
        .unwrap()
        .unwrap_object();
    let interpreter_state = get_interpreter_state(env);
    new_local_ref_public_new(res.as_ref().map(|handle| handle.as_allocated_obj()), todo!()/*interpreter_state*/)
}

pub unsafe extern "C" fn call_void_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) {
    let res = match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return;
        }
    };
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_byte_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jbyte {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jbyte::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_byte_strict()
}

pub unsafe extern "C" fn call_boolean_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jboolean {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jboolean::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_bool_strict()
}

pub unsafe extern "C" fn call_short_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jshort {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jshort::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_short_strict()
}

pub unsafe extern "C" fn call_char_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jchar {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jchar::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_char_strict()
}

pub unsafe extern "C" fn call_int_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jint {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jint::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_int_strict()
}

pub unsafe extern "C" fn call_float_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jfloat {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jfloat::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_float_strict()
}

pub unsafe extern "C" fn call_double_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jdouble {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jdouble::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_double_strict()
}

pub unsafe extern "C" fn call_long_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jlong {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jlong::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_long_strict()
}

pub unsafe extern "C" fn call_object_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jobject {
    let res = match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    }
        .unwrap()
        .unwrap_object();
    let interpreter_state = get_interpreter_state(env);
    new_local_ref_public_new(res.as_ref().map(|handle| handle.as_allocated_obj()), interpreter_state)
}

pub unsafe extern "C" fn call_void_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) {
    let res = match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return;
        }
    };
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_byte_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jbyte {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jbyte::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_byte_strict()
}

pub unsafe extern "C" fn call_boolean_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jboolean {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jboolean::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_bool_strict()
}

pub unsafe extern "C" fn call_short_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jshort {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jshort::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_short_strict()
}

pub unsafe extern "C" fn call_char_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jchar {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jchar::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_char_strict()
}

pub unsafe extern "C" fn call_int_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jint {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jint::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_int_strict()
}

pub unsafe extern "C" fn call_float_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jfloat {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jfloat::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_float_strict()
}

pub unsafe extern "C" fn call_double_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jdouble {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jdouble::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_double_strict()
}

pub unsafe extern "C" fn call_long_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jlong {
    match call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jlong::MAX;
        }
    }
        .unwrap()
        .as_njv()
        .unwrap_long_strict()
}