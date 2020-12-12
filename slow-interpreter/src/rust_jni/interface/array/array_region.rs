use std::ops::Deref;

use num::NumCast;

use jvmti_jni_bindings::{jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jdouble, jdoubleArray, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, JNIEnv, jshort, jshortArray, jsize};

use crate::java_values::JavaValue;
use crate::rust_jni::native_util::from_object;

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


unsafe fn array_region_integer_types<T: NumCast>(array: jarray, start: jsize, len: jsize, buf: *mut T) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().mut_array();
    let array = array_ref.deref();
    for i in 0..len {
        let elem = T::from(array[(start + i) as usize].unwrap_int()).unwrap();
        buf.offset(i as isize).write(elem)
    }
}

//todo a lot of duplication here, but hard to template out.
//should be templated out at the .unwrap_type() level
pub unsafe extern "C" fn get_float_array_region(_env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *mut jfloat) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().mut_array();
    let array = array_ref.deref();
    for i in 0..len {
        let float = array[(start + i) as usize].unwrap_float() as jfloat;
        buf.offset(i as isize).write(float)
    }
}

pub unsafe extern "C" fn get_double_array_region(_env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *mut jdouble) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().mut_array();
    let array = array_ref.deref();
    for i in 0..len {
        let double = array[(start + i) as usize].unwrap_double() as jdouble;
        buf.offset(i as isize).write(double)
    }
}

pub unsafe extern "C" fn get_long_array_region(_env: *mut JNIEnv, array: jlongArray, start: jsize, len: jsize, buf: *mut jlong) {
    let non_null_array_obj = from_object(array).unwrap();
    let array_ref = non_null_array_obj.unwrap_array().mut_array();
    let array = array_ref.deref();
    for i in 0..len {
        let long = array[(start + i) as usize].unwrap_long() as jlong;
        buf.offset(i as isize).write(long)
    }
}


pub unsafe extern "C" fn set_boolean_array_region(_env: *mut JNIEnv, array: jbooleanArray, start: jsize, len: jsize, buf: *const jboolean) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Boolean(buf.offset(index).read())//todo bool need to be u8
    })
}

pub unsafe extern "C" fn set_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *const jbyte) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Byte(buf.offset(index).read() as i8)
    })
}

pub unsafe extern "C" fn set_char_array_region(_env: *mut JNIEnv, array: jcharArray, start: jsize, len: jsize, buf: *const jchar) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Char(buf.offset(index).read())//todo instead of char use u16
    })
}


pub unsafe extern "C" fn set_short_array_region(_env: *mut JNIEnv, array: jshortArray, start: jsize, len: jsize, buf: *const jshort) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Short(buf.offset(index).read() as i16)
    })
}

pub unsafe extern "C" fn set_int_array_region(_env: *mut JNIEnv, array: jintArray, start: jsize, len: jsize, buf: *const jint) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Int(buf.offset(index).read() as i32)
    })
}


pub unsafe extern "C" fn set_float_array_region(_env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *const jfloat) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Float(buf.offset(index).read() as f32)
    })
}

pub unsafe extern "C" fn set_double_array_region(_env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *const jdouble) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Double(buf.offset(index).read() as f64)
    })
}


pub unsafe extern "C" fn set_long_array_region(_env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *const jlong) {
    set_array_region(array, start, len, &mut |index: isize| {
        JavaValue::Long(buf.offset(index).read() as i64)
    })
}


unsafe fn set_array_region(array: jarray, start: i32, len: i32, java_value_getter: &mut dyn FnMut(isize) -> JavaValue) {
    let non_nullarray = from_object(array)
        .unwrap();
    let vec_mut = non_nullarray
        .unwrap_array().mut_array();
    for i in 0..len {
        vec_mut[(start + i) as usize] = java_value_getter(i as isize);
    }
}