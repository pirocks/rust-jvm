use jvmti_jni_bindings::{JNIEnv, jsize, jbyteArray, jbooleanArray, jshortArray, jcharArray, jintArray, jlongArray, jfloatArray, jdoubleArray, jarray};
use classfile_view::view::ptype_view::PTypeView;
use crate::java_values::{default_value, Object, ArrayObject};
use crate::rust_jni::native_util::{to_object, get_state};
use std::cell::RefCell;
use std::sync::Arc;

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
    let mut the_vec = vec![];
    for _ in 0..len {
        the_vec.push(default_value(elem_type.clone()))
    }
    to_object(Some(Arc::new(Object::Array(ArrayObject {
        elems: RefCell::new(the_vec),
        elem_type,
        monitor: jvm.new_monitor("monitor for jni created byte array".to_string())
    }))))
}
