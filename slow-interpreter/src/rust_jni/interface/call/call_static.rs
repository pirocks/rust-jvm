use crate::rust_jni::interface::call::{call_static_method_impl, VarargProvider};
use crate::rust_jni::native_util::{get_frame, to_object};
use jvmti_jni_bindings::{JNIEnv, jclass, jmethodID, jboolean, jbyte, jshort, jchar, jfloat, jlong, jint, jdouble, jobject};
use std::ffi::VaList;

pub unsafe extern "C" fn call_static_boolean_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jboolean {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jboolean
}

pub unsafe extern "C" fn call_static_byte_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jbyte {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jbyte
}

pub unsafe extern "C" fn call_static_short_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jshort {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jshort
}

pub unsafe extern "C" fn call_static_char_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jchar {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jchar
}

pub unsafe extern "C" fn call_static_int_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jint {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int()
}

pub unsafe extern "C" fn call_static_long_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jlong {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_long()
}

pub unsafe extern "C" fn call_static_float_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jfloat {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_float()
}

pub unsafe extern "C" fn call_static_double_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jdouble {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_double()
}

pub unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jobject {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    let res = get_frame(env).pop();
    to_object(res.unwrap_object())
}

pub unsafe extern "C" fn call_static_void_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
}


pub unsafe extern "C" fn call_static_object_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jobject {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    to_object(res.unwrap_object())
}

pub unsafe extern "C" fn call_static_boolean_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jboolean {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jboolean
}


pub unsafe extern "C" fn call_static_byte_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jbyte {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jbyte
}


pub unsafe extern "C" fn call_static_short_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jshort {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jshort
}


pub unsafe extern "C" fn call_static_char_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jchar {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jchar
}

pub unsafe extern "C" fn call_static_int_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jint {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_int() as jint
}


pub unsafe extern "C" fn call_static_float_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jfloat {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_float()
}


pub unsafe extern "C" fn call_static_double_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jdouble {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_double()
}


pub unsafe extern "C" fn call_static_long_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jlong {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    let res = get_frame(env).pop();
    res.unwrap_long()
}


pub unsafe extern "C" fn call_static_void_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...)  {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
}



