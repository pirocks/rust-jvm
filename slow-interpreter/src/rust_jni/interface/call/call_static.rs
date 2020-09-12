use std::ffi::VaList;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jclass, jdouble, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort, jvalue};

use crate::rust_jni::interface::call::{call_static_method_impl, VarargProvider};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::get_interpreter_state;

pub unsafe extern "C" fn call_static_boolean_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jboolean {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_boolean()
}

pub unsafe extern "C" fn call_static_byte_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jbyte {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_byte()
}

pub unsafe extern "C" fn call_static_short_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jshort {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_short()
}

pub unsafe extern "C" fn call_static_char_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jchar {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_char()
}

pub unsafe extern "C" fn call_static_int_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jint {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_int()
}

pub unsafe extern "C" fn call_static_long_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jlong {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_long()
}

pub unsafe extern "C" fn call_static_float_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jfloat {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_float()
}

pub unsafe extern "C" fn call_static_double_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jdouble {
    call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap().unwrap_double()
}

pub unsafe extern "C" fn call_static_object_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) -> jobject {
    let res = call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l)).unwrap();
    new_local_ref_public(res.unwrap_object(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_static_void_method_v(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: VaList) {
    let res = call_static_method_impl(env, method_id, VarargProvider::VaList(&mut l));
    assert_eq!(res, None);
}


pub unsafe extern "C" fn call_static_object_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jobject {
    let res = call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap();
    new_local_ref_public(res.unwrap_object(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_static_boolean_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jboolean {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_boolean()
}


pub unsafe extern "C" fn call_static_byte_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jbyte {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_byte()
}


pub unsafe extern "C" fn call_static_short_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jshort {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_short()
}


pub unsafe extern "C" fn call_static_char_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jchar {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_char()
}

pub unsafe extern "C" fn call_static_int_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jint {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_int()
}


pub unsafe extern "C" fn call_static_float_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jfloat {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_float()
}


pub unsafe extern "C" fn call_static_double_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jdouble {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_double()
}


pub unsafe extern "C" fn call_static_long_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) -> jlong {
    call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_long()
}


pub unsafe extern "C" fn call_static_void_method(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, mut l: ...) {
    let res = call_static_method_impl(env, method_id, VarargProvider::Dots(&mut l));
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_static_object_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jobject {
    let res = call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap();
    new_local_ref_public(res.unwrap_object(), get_interpreter_state(env))
}

pub unsafe extern "C" fn call_static_boolean_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jboolean {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_boolean()
}


pub unsafe extern "C" fn call_static_byte_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jbyte {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_byte()
}


pub unsafe extern "C" fn call_static_short_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jshort {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_short()
}


pub unsafe extern "C" fn call_static_char_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jchar {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_char()
}

pub unsafe extern "C" fn call_static_int_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jint {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_int()
}


pub unsafe extern "C" fn call_static_float_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jfloat {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_float()
}


pub unsafe extern "C" fn call_static_double_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jdouble {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_double()
}


pub unsafe extern "C" fn call_static_long_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) -> jlong {
    call_static_method_impl(env, method_id, VarargProvider::Array(args)).unwrap().unwrap_long()
}

pub unsafe extern "C" fn call_static_void_method_a(env: *mut JNIEnv, _clazz: jclass, method_id: jmethodID, args: *const jvalue) {
    let res = call_static_method_impl(env, method_id, VarargProvider::Array(args));
    assert_eq!(res, None);
}



