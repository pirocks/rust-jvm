use std::ptr::null_mut;
use std::sync::Arc;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jarray, jbooleanArray, jbyteArray, jcharArray, jclass, jdoubleArray, jfloatArray, jintArray, jlongArray, JNIEnv, jobject, jobjectArray, jshortArray, jsize};

use crate::interpreter::WasException;
use crate::java_values::{ArrayObject, default_value, JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_jclass, from_object, get_interpreter_state, get_state};
use crate::utils::throw_npe;

pub unsafe extern "C" fn new_object_array(env: *mut JNIEnv, len: jsize, clazz: jclass, init: jobject) -> jobjectArray {
    let jvm = get_state(env);
    let type_ = from_jclass(clazz).as_type(jvm);
    let res = new_array(env, len, type_);
    let res_safe = match from_object(res) {
        Some(x) => x,
        None => return throw_npe(jvm, get_interpreter_state(env)),
    };
    for jv in res_safe.unwrap_array().mut_array().iter_mut() {
        *jv = JavaValue::Object(todo!()/*from_object(init)*/);
    }
    res
}

pub unsafe extern "C" fn new_boolean_array(env: *mut JNIEnv, len: jsize) -> jbooleanArray {
    new_array(env, len, PTypeView::BooleanType)
}

pub unsafe extern "C" fn new_byte_array(env: *mut JNIEnv, len: jsize) -> jbyteArray {
    new_array(env, len, PTypeView::ByteType)
}

pub unsafe extern "C" fn new_short_array(env: *mut JNIEnv, len: jsize) -> jshortArray {
    new_array(env, len, PTypeView::ShortType)
}

pub unsafe extern "C" fn new_char_array(env: *mut JNIEnv, len: jsize) -> jcharArray {
    new_array(env, len, PTypeView::CharType)
}

pub unsafe extern "C" fn new_int_array(env: *mut JNIEnv, len: jsize) -> jintArray {
    new_array(env, len, PTypeView::IntType)
}

pub unsafe extern "C" fn new_long_array(env: *mut JNIEnv, len: jsize) -> jlongArray {
    new_array(env, len, PTypeView::LongType)
}

pub unsafe extern "C" fn new_float_array(env: *mut JNIEnv, len: jsize) -> jfloatArray {
    new_array(env, len, PTypeView::FloatType)
}

pub unsafe extern "C" fn new_double_array(env: *mut JNIEnv, len: jsize) -> jdoubleArray {
    new_array(env, len, PTypeView::DoubleType)
}

unsafe fn new_array(env: *mut JNIEnv, len: i32, elem_type: PTypeView) -> jarray {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let mut the_vec = vec![];
    for _ in 0..len {
        the_vec.push(default_value(elem_type.clone()))
    }
    new_local_ref_public(Some(Arc::new(Object::Array(match ArrayObject::new_array(jvm, int_state,
                                                                                  the_vec,
                                                                                  elem_type,
                                                                                  jvm.thread_state.new_monitor("monitor for jni created byte array".to_string()),
    ) {
        Ok(arr) => arr,
        Err(WasException {}) => return null_mut()
    }))), int_state)
}
