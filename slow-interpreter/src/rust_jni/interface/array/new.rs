use std::ptr::null_mut;

use jvmti_jni_bindings::{
    jarray, jbooleanArray, jbyteArray, jcharArray, jclass, jdoubleArray, jfloatArray, jintArray,
    jlongArray, JNIEnv, jobject, jobjectArray, jshortArray, jsize,
};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::interpreter::WasException;
use crate::java_values::{ArrayObject, default_value, JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use crate::utils::throw_npe;

pub unsafe extern "C" fn new_object_array(
    env: *mut JNIEnv,
    len: jsize,
    clazz: jclass,
    init: jobject,
) -> jobjectArray {
    let jvm = get_state(env);
    let type_ = from_jclass(jvm, clazz).as_type(jvm);
    let res = new_array(env, len, type_);
    let res_safe = match from_object(jvm, res) {
        Some(x) => x,
        None => return throw_npe(jvm, get_interpreter_state(env)),
    };
    let array = res_safe.unwrap_array();
    for i in 0..array.len() {
        array.set_i(jvm, i, JavaValue::Object(from_object(jvm, init)));
    }
    res
}

pub unsafe extern "C" fn new_boolean_array(env: *mut JNIEnv, len: jsize) -> jbooleanArray {
    new_array(env, len, CPDType::BooleanType)
}

pub unsafe extern "C" fn new_byte_array(env: *mut JNIEnv, len: jsize) -> jbyteArray {
    new_array(env, len, CPDType::ByteType)
}

pub unsafe extern "C" fn new_short_array(env: *mut JNIEnv, len: jsize) -> jshortArray {
    new_array(env, len, CPDType::ShortType)
}

pub unsafe extern "C" fn new_char_array(env: *mut JNIEnv, len: jsize) -> jcharArray {
    new_array(env, len, CPDType::CharType)
}

pub unsafe extern "C" fn new_int_array(env: *mut JNIEnv, len: jsize) -> jintArray {
    new_array(env, len, CPDType::IntType)
}

pub unsafe extern "C" fn new_long_array(env: *mut JNIEnv, len: jsize) -> jlongArray {
    new_array(env, len, CPDType::LongType)
}

pub unsafe extern "C" fn new_float_array(env: *mut JNIEnv, len: jsize) -> jfloatArray {
    new_array(env, len, CPDType::FloatType)
}

pub unsafe extern "C" fn new_double_array(env: *mut JNIEnv, len: jsize) -> jdoubleArray {
    new_array(env, len, CPDType::DoubleType)
}

unsafe fn new_array(env: *mut JNIEnv, len: i32, elem_type: CPDType) -> jarray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut the_vec = vec![];
    for _ in 0..len {
        the_vec.push(default_value(elem_type.clone()))
    }
    new_local_ref_public(
        Some(
            jvm.allocate_object(Object::Array(
                match ArrayObject::new_array(
                    jvm,
                    int_state,
                    the_vec,
                    elem_type,
                    jvm.thread_state
                        .new_monitor("monitor for jni created byte array".to_string()),
                ) {
                    Ok(arr) => arr,
                    Err(WasException {}) => return null_mut(),
                },
            )),
        ),
        int_state,
    )
}