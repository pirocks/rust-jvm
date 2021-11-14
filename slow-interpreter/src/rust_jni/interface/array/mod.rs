use std::ops::Deref;
use std::os::raw::c_void;

use jvmti_jni_bindings::{
    jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jdouble, jdoubleArray,
    jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, JNI_ABORT, JNIEnv, jobject,
    jobjectArray, jshort, jshortArray, jsize,
};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::java_values::{JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use crate::utils::{throw_illegal_arg, throw_npe};

pub unsafe extern "C" fn get_array_length(env: *mut JNIEnv, array: jarray) -> jsize {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let temp = match from_object(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let non_null_array: &Object = temp.deref();
    let len = match non_null_array {
        Object::Array(a) => a.len(),
        Object::Object(_o) => {
            return throw_illegal_arg(jvm, int_state);
        }
    };
    len as jsize
}

pub unsafe extern "C" fn get_object_array_element(
    env: *mut JNIEnv,
    array: jobjectArray,
    index: jsize,
) -> jobject {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let notnull = match from_object(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let int_state = get_interpreter_state(env);
    let array = notnull.unwrap_array();
    new_local_ref_public(array.get_i(jvm, index).unwrap_object(), int_state)
}

pub unsafe extern "C" fn set_object_array_element(
    env: *mut JNIEnv,
    array: jobjectArray,
    index: jsize,
    val: jobject,
) {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let notnull = match from_object(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array = notnull.unwrap_array();
    array.set_i(jvm, index, from_object(jvm, val).into());
}

pub mod array_region;
pub mod new;

pub unsafe extern "C" fn release_primitive_array_critical(
    env: *mut JNIEnv,
    array: jarray,
    carray: *mut ::std::os::raw::c_void,
    mode: jint,
) {
    // assert_eq!(mode, 0);
    if mode == JNI_ABORT as i32 {
        return;
    }
    //todo handle JNI_COMMIT
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let not_null = match from_object(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array = not_null.unwrap_array();
    let array_type = &array.elem_type;
    for i in 0..array.len() {
        match array_type {
            CPDType::ByteType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Byte((carray as *const jbyte).offset(i as isize).read()),
                );
            }
            CPDType::CharType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Char((carray as *const jchar).offset(i as isize).read()),
                );
            }
            CPDType::DoubleType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Double((carray as *const jdouble).offset(i as isize).read()),
                );
            }
            CPDType::FloatType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Float((carray as *const jfloat).offset(i as isize).read()),
                );
            }
            CPDType::IntType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Int((carray as *const jint).offset(i as isize).read()),
                );
            }
            CPDType::LongType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Long((carray as *const jlong).offset(i as isize).read()),
                );
            }
            CPDType::Ref(_) => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Object(from_object(
                        jvm,
                        (carray as *const jobject).offset(i as isize).read(),
                    )),
                );
            }
            CPDType::ShortType => {
                array.set_i(
                    jvm,
                    i,
                    JavaValue::Short((carray as *const jshort).offset(i as isize).read()),
                );
            }
            CPDType::BooleanType => {
                let boolean = (carray as *const jboolean).offset(i as isize).read();
                assert!(boolean == 1 || boolean == 0);
                array.set_i(jvm, i, JavaValue::Boolean(boolean));
            }
            _ => panic!(),
        }
    }
}

pub unsafe extern "C" fn get_primitive_array_critical(
    env: *mut JNIEnv,
    array: jarray,
    is_copy: *mut jboolean,
) -> *mut c_void {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let not_null = match from_object(jvm, array) {
        Some(x) => x,
        None => {
            return throw_npe(jvm, int_state);
        }
    };
    let array = not_null.unwrap_array();
    if !is_copy.is_null() {
        is_copy.write(true as jboolean);
    }
    //dup but difficult to make into template so ehh
    match &array.elem_type {
        CPDType::ByteType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_byte())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::CharType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_char())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::DoubleType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_double())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::FloatType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_float())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::IntType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_int())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::LongType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_long())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::ShortType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_short())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::BooleanType => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| elem.unwrap_boolean())
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        CPDType::Ref(_) => {
            let res = array
                .array_iterator(jvm)
                .map(|elem| to_object(elem.unwrap_object()))
                .collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        _ => panic!(),
    }
}

pub unsafe extern "C" fn get_byte_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jbyte {
    get_primitive_array_critical(env, array, is_copy) as *mut jbyte
}

pub unsafe extern "C" fn get_char_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jchar {
    get_primitive_array_critical(env, array, is_copy) as *mut jchar
}

pub unsafe extern "C" fn get_double_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jdouble {
    get_primitive_array_critical(env, array, is_copy) as *mut jdouble
}

pub unsafe extern "C" fn get_float_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jfloat {
    get_primitive_array_critical(env, array, is_copy) as *mut jfloat
}

pub unsafe extern "C" fn get_int_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jint {
    get_primitive_array_critical(env, array, is_copy) as *mut jint
}

pub unsafe extern "C" fn get_short_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jshort {
    get_primitive_array_critical(env, array, is_copy) as *mut jshort
}

pub unsafe extern "C" fn get_boolean_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jboolean {
    get_primitive_array_critical(env, array, is_copy) as *mut jboolean
}

pub unsafe extern "C" fn get_object_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jobject {
    get_primitive_array_critical(env, array, is_copy) as *mut jobject
}

pub unsafe extern "C" fn get_long_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    is_copy: *mut jboolean,
) -> *mut jlong {
    get_primitive_array_critical(env, array, is_copy) as *mut jlong
}

pub unsafe extern "C" fn release_byte_array_elements(
    env: *mut JNIEnv,
    array: jbyteArray,
    elems: *mut jbyte,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_char_array_elements(
    env: *mut JNIEnv,
    array: jcharArray,
    elems: *mut jchar,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_double_array_elements(
    env: *mut JNIEnv,
    array: jdoubleArray,
    elems: *mut jdouble,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_float_array_elements(
    env: *mut JNIEnv,
    array: jfloatArray,
    elems: *mut jfloat,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_int_array_elements(
    env: *mut JNIEnv,
    array: jintArray,
    elems: *mut jint,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_short_array_elements(
    env: *mut JNIEnv,
    array: jshortArray,
    elems: *mut jshort,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_boolean_array_elements(
    env: *mut JNIEnv,
    array: jbooleanArray,
    elems: *mut jboolean,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_object_array_elements(
    env: *mut JNIEnv,
    array: jobjectArray,
    elems: *mut jobject,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_long_array_elements(
    env: *mut JNIEnv,
    array: jlongArray,
    elems: *mut jlong,
    mode: jint,
) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}