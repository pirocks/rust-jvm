use std::ffi::VaList;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jmethodID, JNIEnv, jobject, jshort, jvalue};

use crate::rust_jni::interface::call::{call_nonstatic_method, VarargProvider};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::get_interpreter_state;

pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
    let res = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_object();
    new_local_ref_public(res, get_interpreter_state(env))
}

pub unsafe extern "C" fn call_void_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) {
    let res = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    assert_eq!(res, None);
}


pub unsafe extern "C" fn call_byte_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jbyte {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_byte()
}

pub unsafe extern "C" fn call_boolean_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jboolean {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_boolean()
}

pub unsafe extern "C" fn call_short_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jshort {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_short()
}

pub unsafe extern "C" fn call_char_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jchar {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_char()
}


pub unsafe extern "C" fn call_int_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jint {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_int()
}

pub unsafe extern "C" fn call_float_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jfloat {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_float()
}


pub unsafe extern "C" fn call_double_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jdouble {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_double()
}

pub unsafe extern "C" fn call_long_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jlong {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l)).unwrap().unwrap_long()
}


pub unsafe extern "C" fn call_object_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jobject {
    let res = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_object();
    new_local_ref_public(res, get_interpreter_state(env))
}

pub unsafe extern "C" fn call_void_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) {
    let res = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_byte_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jbyte {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_byte()
}

pub unsafe extern "C" fn call_boolean_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jboolean {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_boolean()
}

pub unsafe extern "C" fn call_short_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jshort {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_short()
}

pub unsafe extern "C" fn call_char_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jchar {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_char()
}

pub unsafe extern "C" fn call_int_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jint {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_int()
}

pub unsafe extern "C" fn call_float_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jfloat {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_float()
}

pub unsafe extern "C" fn call_double_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jdouble {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_double()
}

pub unsafe extern "C" fn call_long_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jlong {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args)).unwrap().unwrap_long()
}


pub unsafe extern "C" fn call_object_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jobject {
    let res = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_object();
    new_local_ref_public(res, get_interpreter_state(env))
}

pub unsafe extern "C" fn call_void_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) {
    let res = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    assert_eq!(res, None);
}

pub unsafe extern "C" fn call_byte_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jbyte {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_byte()
}

pub unsafe extern "C" fn call_boolean_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jboolean {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_boolean()
}

pub unsafe extern "C" fn call_short_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jshort {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_short()
}

pub unsafe extern "C" fn call_char_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jchar {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_char()
}

pub unsafe extern "C" fn call_int_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jint {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_int()
}

pub unsafe extern "C" fn call_float_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jfloat {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_float()
}

pub unsafe extern "C" fn call_double_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jdouble {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_double()
}

pub unsafe extern "C" fn call_long_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jlong {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args)).unwrap().unwrap_long()
}