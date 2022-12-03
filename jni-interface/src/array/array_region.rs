use std::ptr::NonNull;
use libc::c_void;
use array_memory_layout::accessor::Accessor;
use array_memory_layout::layout::{ArrayAccessor, ArrayMemoryLayout};

use jvmti_jni_bindings::{jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jdouble, jdoubleArray, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, JNIEnv, jshort, jshortArray, jsize};
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType};
use slow_interpreter::new_java_values::java_value_common::JavaValueCommon;
use slow_interpreter::rust_jni::jni_utils::{get_interpreter_state, get_state, get_throw};
use slow_interpreter::rust_jni::native_util::{from_object_new};
use slow_interpreter::utils::throw_npe;

pub unsafe extern "C" fn get_boolean_array_region(env: *mut JNIEnv, array: jbooleanArray, start: jsize, len: jsize, buf: *mut jboolean) {
    array_region_integer_types(env, array, start, len, buf)
}

pub unsafe extern "C" fn get_byte_array_region(env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *mut jbyte) {
    array_region_integer_types(env, array, start, len, buf)
}

pub unsafe extern "C" fn get_short_array_region(env: *mut JNIEnv, array: jshortArray, start: jsize, len: jsize, buf: *mut jshort) {
    array_region_integer_types(env, array, start, len, buf)
}

pub unsafe extern "C" fn get_char_array_region(env: *mut JNIEnv, array: jcharArray, start: jsize, len: jsize, buf: *mut jchar) {
    array_region_integer_types(env, array, start, len, buf)
}

pub unsafe extern "C" fn get_int_array_region(env: *mut JNIEnv, array: jintArray, start: jsize, len: jsize, buf: *mut jint) {
    array_region_integer_types(env, array, start, len, buf)
}

unsafe fn array_region_integer_types<T>(env: *mut JNIEnv, raw_array: jarray, start: jsize, len: jsize, buf: *mut T) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);

    let non_null_array_obj = match from_object_new(jvm, raw_array) {
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
        Some(x) => x,
    };
    let array = non_null_array_obj.unwrap_array();
    let array_subtype = array.elem_cpdtype();
    let layout = ArrayMemoryLayout::from_cpdtype(array_subtype);
    for i in 0..len {
        let elem = layout.calculate_index_address(NonNull::new(raw_array as *mut c_void).unwrap(),i);
        buf.offset(i as isize).write(elem.read_impl())
    }
}

//todo a lot of duplication here, but hard to template out, have an unwrap type_ closure
pub unsafe extern "C" fn get_float_array_region(env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *mut jfloat) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let non_null_array_obj = match from_object_new(jvm, array) {
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
        Some(x) => x,
    };
    let array = non_null_array_obj.unwrap_array();
    for i in 0..len {
        let float = array.get_i(start + i).unwrap_float_strict() as jfloat;
        buf.offset(i as isize).write(float)
    }
}

pub unsafe extern "C" fn get_double_array_region(env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *mut jdouble) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let non_null_array_obj = match from_object_new(jvm, array) {
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
        Some(x) => x,
    };
    let array = non_null_array_obj.unwrap_array();
    for i in 0..len {
        let double = array.get_i(start + i).unwrap_double_strict() as jdouble;
        buf.offset(i as isize).write(double)
    }
}

pub unsafe extern "C" fn get_long_array_region(env: *mut JNIEnv, array: jlongArray, start: jsize, len: jsize, buf: *mut jlong) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let non_null_array_obj = match from_object_new(jvm, array) {
        None => {
            return throw_npe(jvm, int_state,get_throw(env));
        }
        Some(x) => x,
    };
    let array = non_null_array_obj.unwrap_array();
    for i in 0..len {
        let long = array.get_i(start + i).unwrap_long_strict() as jlong;
        buf.offset(i as isize).write(long)
    }
}

pub unsafe extern "C" fn set_boolean_array_region(env: *mut JNIEnv, array: jbooleanArray, start: jsize, len: jsize, buf: *const jboolean) {
    set_array_region(env, array, CPDType::BooleanType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_boolean(buf.offset(index).read()))
}

pub unsafe extern "C" fn set_byte_array_region(env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *const jbyte) {
    set_array_region(env, array, CPDType::ByteType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_byte(buf.offset(index).read() as i8))
}

pub unsafe extern "C" fn set_char_array_region(env: *mut JNIEnv, array: jcharArray, start: jsize, len: jsize, buf: *const jchar) {
    set_array_region(env, array, CPDType::CharType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_char(buf.offset(index).read()))
}

pub unsafe extern "C" fn set_short_array_region(env: *mut JNIEnv, array: jshortArray, start: jsize, len: jsize, buf: *const jshort) {
    set_array_region(env, array, CPDType::ShortType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_short(buf.offset(index).read() as i16))
}

pub unsafe extern "C" fn set_int_array_region(env: *mut JNIEnv, array: jintArray, start: jsize, len: jsize, buf: *const jint) {
    set_array_region(env, array, CPDType::IntType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_int(buf.offset(index).read() as i32))
}

pub unsafe extern "C" fn set_float_array_region(env: *mut JNIEnv, array: jfloatArray, start: jsize, len: jsize, buf: *const jfloat) {
    set_array_region(env, array, CPDType::FloatType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_float(buf.offset(index).read() as f32))
}

pub unsafe extern "C" fn set_double_array_region(env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *const jdouble) {
    set_array_region(env, array, CPDType::DoubleType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_double(buf.offset(index).read() as f64))
}

pub unsafe extern "C" fn set_long_array_region(env: *mut JNIEnv, array: jdoubleArray, start: jsize, len: jsize, buf: *const jlong) {
    set_array_region(env, array, CPDType::LongType, start, len, &mut |index: isize, write_to: ArrayAccessor| write_to.write_long(buf.offset(index).read() as i64))
}

unsafe fn set_array_region<'gc>(env: *mut JNIEnv, array: jarray, array_sub_type: CPDType, start: i32, len: i32, java_value_setter: &mut dyn FnMut(isize, ArrayAccessor)) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    if let None = from_object_new(jvm, array) {
        return throw_npe(jvm, int_state,get_throw(env));
    }
    let memory_layout = ArrayMemoryLayout::from_cpdtype(array_sub_type);
    let array_pointer = NonNull::new(array as *mut c_void).unwrap();
    for i in 0..len {
        let write_to = memory_layout.calculate_index_address(array_pointer, i);
        java_value_setter(i as isize, write_to);
    }
}