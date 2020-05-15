use jvmti_jni_bindings::{jbooleanArray, JNIEnv, jbyte, jsize, jboolean, jbyteArray, jshort, jshortArray, jarray, jchar, jfloatArray, jintArray, jcharArray, jfloat, jdouble, jdoubleArray, jlong, jlongArray};
use crate::rust_jni::native_util::from_object;
use std::ops::Deref;

pub unsafe extern "C" fn get_boolean_array_region(_env: *mut JNIEnv, array: jbooleanArray, start: jsize, len: jsize, buf: *mut jboolean) {
    array_region_integer_types(array, start, len, buf)
}

pub unsafe extern "C" fn get_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte) {
    array_region_integer_types(array, start, len, buf)
}

pub unsafe extern "C" fn get_short_array_region(_env: *mut JNIEnv, array: jshortArray, start: jsize, len: jsize, buf: *mut jshort) {
    array_region_integer_types(array, start, len, buf)
}

pub unsafe extern "C" fn get_char_array_region(_env: *mut JNIEnv, array: jcharArray, start: jsize, len: jsize, buf: *mut jchar) {
    array_region_integer_types(array, start, len, buf)
}

pub unsafe extern "C" fn get_int_array_region(_env: *mut JNIEnv, array: jintArray, start: jsize, len: jsize, buf: *mut jint) {
    array_region_integer_types(array, start, len, buf)
}


unsafe fn array_region_integer_types<T>(array: jarray, start: jsize, len: jsize, buf: *mut T) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().elems.borrow();
    let array = array_ref.deref();
    for i in 0..len {
        let elem = array[(start + i) as usize].unwrap_int() as T;
        buf.offset(i as isize).write(elem)
    }
}

//todo a lot of duplication here, but hard to template out.
//should be templated out at the .unwrap_type() level
pub unsafe extern "C" fn get_float_array_region(_env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *mut jfloat) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().elems.borrow();
    let array = array_ref.deref();
    for i in 0..len {
        let float = array[(start + i) as usize].unwrap_float() as jfloat;
        buf.offset(i as isize).write(float)
    }
}

pub unsafe extern "C" fn get_double_array_region(_env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *mut jdouble) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().elems.borrow();
    let array = array_ref.deref();
    for i in 0..len {
        let double = array[(start + i) as usize].unwrap_double() as jdouble;
        buf.offset(i as isize).write(double)
    }
}

pub unsafe extern "C" fn get_long_array_region(_env: *mut JNIEnv, array: jlongArray, start: jsize, len: jsize, buf: *mut jlong) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().elems.borrow();
    let array = array_ref.deref();
    for i in 0..len {
        let long = array[(start + i) as usize].unwrap_long() as jlong;
        buf.offset(i as isize).write(long)
    }
}
