use crate::rust_jni::interface::call::{call_nonstatic_method, VarargProvider};
use jvmti_jni_bindings::{JNIEnv, jobject, jmethodID, jbyte, jboolean, jshort, jchar, jfloat, jint, jdouble, jlong, jvalue};
use crate::rust_jni::native_util::to_object;
use std::ffi::VaList;

pub unsafe extern "C" fn call_object_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jobject {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop().unwrap_object();
    to_object(res)
}

pub unsafe extern "C" fn call_void_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
}


pub unsafe extern "C" fn call_byte_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jbyte {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_byte()
}

pub unsafe extern "C" fn call_boolean_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jboolean {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_boolean()
}

pub unsafe extern "C" fn call_short_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jshort {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_short()
}

pub unsafe extern "C" fn call_char_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jchar {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_char()
}


pub unsafe extern "C" fn call_int_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jint {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_int()
}

pub unsafe extern "C" fn call_float_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jfloat {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_float()
}


pub unsafe extern "C" fn call_double_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jdouble {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_double()
}

pub unsafe extern "C" fn call_long_method(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut l: ...) -> jlong {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Dots(&mut l));
    let res = frame.pop();
    res.unwrap_long()
}


pub unsafe extern "C" fn call_object_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jobject {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop().unwrap_object();
    to_object(res)
}

pub unsafe extern "C" fn call_void_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) {
    call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
}

pub unsafe extern "C" fn call_byte_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jbyte {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_byte()
}

pub unsafe extern "C" fn call_boolean_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jboolean {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_boolean()
}

pub unsafe extern "C" fn call_short_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jshort {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_short()
}

pub unsafe extern "C" fn call_char_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jchar {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_char()
}

pub unsafe extern "C" fn call_int_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jint {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_int()
}

pub unsafe extern "C" fn call_float_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jfloat {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_float()
}

pub unsafe extern "C" fn call_double_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jdouble {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_double()
}

pub unsafe extern "C" fn call_long_method_a(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, args: *const jvalue) -> jlong {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::Array(args));
    let res = frame.pop();
    res.unwrap_long()
}







pub unsafe extern "C" fn call_object_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jobject {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop().unwrap_object();
    to_object(res)
}

pub unsafe extern "C" fn call_void_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) {
    call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
}

pub unsafe extern "C" fn call_byte_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jbyte {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_byte()
}

pub unsafe extern "C" fn call_boolean_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jboolean {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_boolean()
}

pub unsafe extern "C" fn call_short_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jshort {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_short()
}

pub unsafe extern "C" fn call_char_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jchar {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_char()
}

pub unsafe extern "C" fn call_int_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jint {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_int()
}

pub unsafe extern "C" fn call_float_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jfloat {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_float()
}

pub unsafe extern "C" fn call_double_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jdouble {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_double()
}

pub unsafe extern "C" fn call_long_method_v(env: *mut JNIEnv, obj: jobject, method_id: jmethodID, mut args: VaList) -> jlong {
    let frame = call_nonstatic_method(env, obj, method_id, VarargProvider::VaList(&mut args));
    let res = frame.pop();
    res.unwrap_long()
}