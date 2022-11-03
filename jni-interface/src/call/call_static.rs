use std::ffi::VaList;
use std::ptr::null_mut;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort, jvalue};
use slow_interpreter::better_java_stack::frames::HasFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;

use slow_interpreter::rust_jni::jni_utils::{get_state, new_local_ref_public_new};
use crate::call::{call_static_method_impl, VarargProvider};
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state};
use slow_interpreter::stdlib::java::NewAsObjectOrJavaValue;

pub unsafe extern "C" fn call_static_boolean_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jboolean {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            let jvm = get_state(env);
            let interpreter_state = get_interpreter_state(env);
            interpreter_state.debug_print_stack_trace(jvm);
            dbg!(exception_obj.to_string(jvm,interpreter_state).unwrap().unwrap().to_rust_string(jvm));
            todo!();
            return jboolean::MAX;
        }
    }
        .unwrap()
        .unwrap_int() as jboolean
}

pub unsafe extern "C" fn call_static_byte_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jbyte {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jbyte::MAX;
        }
    }
        .unwrap()
        .unwrap_byte_strict()
}

pub unsafe extern "C" fn call_static_short_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jshort {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jshort::MAX;
        }
    }
        .unwrap()
        .unwrap_short_strict()
}

pub unsafe extern "C" fn call_static_char_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jchar {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jchar::MAX;
        }
    }
        .unwrap()
        .unwrap_char_strict()
}

pub unsafe extern "C" fn call_static_int_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jint {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jint::MAX;
        }
    }
        .unwrap()
        .unwrap_int_strict()
}

pub unsafe extern "C" fn call_static_long_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jlong {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jlong::MAX;
        }
    }
        .unwrap()
        .unwrap_long_strict()
}

pub unsafe extern "C" fn call_static_float_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jfloat {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jfloat::MAX;
        }
    }
        .unwrap()
        .unwrap_float_strict()
}

pub unsafe extern "C" fn call_static_double_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jdouble {
    match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jdouble::MAX;
        }
    }
        .unwrap()
        .unwrap_double_strict()
}

pub unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jobject {
    let res = match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    }
        .unwrap();
    new_local_ref_public_new(res.as_njv().unwrap_object_alloc(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_static_void_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) {
    let res = match call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return;
        }
    };
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_static_object_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jobject {
    let res = match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    }
        .unwrap();
    let interpreter_state = get_interpreter_state(env);
    new_local_ref_public_new(res.as_njv().unwrap_object_alloc(), todo!()/*interpreter_state*/)
}

pub unsafe extern "C" fn call_static_boolean_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jboolean {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jboolean::MAX;
        }
    }
        .unwrap()
        .unwrap_int() as jboolean
}

pub unsafe extern "C" fn call_static_byte_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jbyte {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jbyte::MAX;
        }
    }
        .unwrap()
        .unwrap_byte_strict()
}

pub unsafe extern "C" fn call_static_short_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jshort {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jshort::MAX;
        }
    }
        .unwrap()
        .unwrap_short_strict()
}

pub unsafe extern "C" fn call_static_char_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jchar {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jchar::MAX;
        }
    }
        .unwrap()
        .unwrap_char_strict()
}

pub unsafe extern "C" fn call_static_int_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jint {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jint::MAX;
        }
    }
        .unwrap()
        .unwrap_int_strict()
}

pub unsafe extern "C" fn call_static_float_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jfloat {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jfloat::MAX;
        }
    }
        .unwrap()
        .unwrap_float_strict()
}

pub unsafe extern "C" fn call_static_double_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jdouble {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jdouble::MAX;
        }
    }
        .unwrap()
        .unwrap_double_strict()
}

pub unsafe extern "C" fn call_static_long_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jlong {
    match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jlong::MAX;
        }
    }
        .unwrap()
        .unwrap_long_strict()
}

pub unsafe extern "C" fn call_static_void_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) {
    let res = match call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return;
        }
    };
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_static_object_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jobject {
    let res = match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return null_mut();
        }
    }
        .unwrap();
    let interpreter_state = get_interpreter_state(env);
    new_local_ref_public_new(res.as_njv().unwrap_object_alloc(), todo!()/*interpreter_state*/)
}

pub unsafe extern "C" fn call_static_boolean_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jboolean {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jboolean::MAX;
        }
    }
        .unwrap()
        .unwrap_bool_strict()
}

pub unsafe extern "C" fn call_static_byte_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jbyte {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jbyte::MAX;
        }
    }
        .unwrap()
        .unwrap_byte_strict()
}

pub unsafe extern "C" fn call_static_short_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jshort {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jshort::MAX;
        }
    }
        .unwrap()
        .unwrap_short_strict()
}

pub unsafe extern "C" fn call_static_char_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jchar {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jchar::MAX;
        }
    }
        .unwrap()
        .unwrap_char_strict()
}

pub unsafe extern "C" fn call_static_int_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jint {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jint::MAX;
        }
    }
        .unwrap()
        .unwrap_int_strict()
}

pub unsafe extern "C" fn call_static_float_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jfloat {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jfloat::MAX;
        }
    }
        .unwrap()
        .unwrap_float_strict()
}

pub unsafe extern "C" fn call_static_double_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jdouble {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jdouble::MAX;
        }
    }
        .unwrap()
        .unwrap_double_strict()
}

pub unsafe extern "C" fn call_static_long_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jlong {
    match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!();
            return jlong::MAX;
        }
    }
        .unwrap()
        .unwrap_long_strict()
}

pub unsafe extern "C" fn call_static_void_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) {
    let res = match call_static_method_impl(env, method_id, VarargProvider::Array(args)) {
        Ok(res) => res,
        Err(WasException { exception_obj }) => {
            todo!()
        }
    };
    assert_eq!(res, None);
}