use std::os::raw::c_void;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jarray, jboolean, jbooleanArray, jbyte, jbyteArray, jchar, jcharArray, jdouble, jdoubleArray, jfloat, jfloatArray, jint, jintArray, jlong, jlongArray, JNI_ABORT, JNIEnv, jobject, jobjectArray, jshort, jshortArray, jsize};

use crate::java_values::{JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state, get_state, to_object};
use crate::utils::{throw_illegal_arg, throw_npe, throw_npe_res};

pub unsafe extern "C" fn get_array_length(env: *mut JNIEnv, array: jarray) -> jsize {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let non_null_array: &Object = &match from_object(array) {
        Some(x) => x,
        None => {
            throw_npe(jvm, int_state);
            return jsize::MAX
        },
    };
    let len = match non_null_array {
        Object::Array(a) => {
            a.mut_array().len()
        }
        Object::Object(_o) => {
            throw_illegal_arg(jvm, int_state);
            return jsize::MAX
        }
    };
    len as jsize
}

pub unsafe extern "C" fn get_object_array_element(env: *mut JNIEnv, array: jobjectArray, index: jsize) -> jobject {
    let notnull = from_object(array).unwrap();//todo handle npe
    let int_state = get_interpreter_state(env);
    let array = notnull.unwrap_array();
    let borrow = array.mut_array();
    new_local_ref_public(borrow[index as usize].unwrap_object(), int_state)
}

pub unsafe extern "C" fn set_object_array_element(_env: *mut JNIEnv, array: jobjectArray, index: jsize, val: jobject) {
    let notnull = from_object(array).unwrap();//todo handle npe
    let array = notnull.unwrap_array();
    let borrow_mut = array.mut_array();
    borrow_mut[index as usize] = from_object(val).into();
}

pub mod array_region;
pub mod new;


pub unsafe extern "C" fn release_primitive_array_critical(_env: *mut JNIEnv, array: jarray, carray: *mut ::std::os::raw::c_void, mode: jint) {
    // assert_eq!(mode, 0);
    if mode == JNI_ABORT as i32 {
        return;
    }
    //todo handle JNI_COMMIT
    let not_null = from_object(array).unwrap();//todo handle npe
    let array = not_null.unwrap_array();
    let array_type = &array.elem_type;
    let array = array.mut_array();
    for (i, elem) in array.iter_mut().enumerate() {
        match array_type {
            PTypeView::ByteType => {
                *elem = JavaValue::Byte((carray as *const jbyte).offset(i as isize).read());
            }
            PTypeView::CharType => {
                *elem = JavaValue::Char((carray as *const jchar).offset(i as isize).read());
            },
            PTypeView::DoubleType => {
                *elem = JavaValue::Double((carray as *const jdouble).offset(i as isize).read());
            },
            PTypeView::FloatType => {
                *elem = JavaValue::Float((carray as *const jfloat).offset(i as isize).read());
            },
            PTypeView::IntType => {
                *elem = JavaValue::Int((carray as *const jint).offset(i as isize).read());
            }
            PTypeView::LongType => {
                *elem = JavaValue::Long((carray as *const jlong).offset(i as isize).read());
            }
            PTypeView::Ref(_) => {
                *elem = JavaValue::Object(from_object((carray as *const jobject).offset(i as isize).read()));
            },
            PTypeView::ShortType => {
                *elem = JavaValue::Short((carray as *const jshort).offset(i as isize).read());
            },
            PTypeView::BooleanType => {
                *elem = JavaValue::Boolean((carray as *const jboolean).offset(i as isize).read());
            },
            _ => panic!()
        }
    }
}

pub unsafe extern "C" fn get_primitive_array_critical(_env: *mut JNIEnv, array: jarray, is_copy: *mut jboolean) -> *mut c_void {
    let not_null = from_object(array).unwrap();//todo handle npe
    let array = not_null.unwrap_array();
    if !is_copy.is_null() {
        is_copy.write(true as jboolean);
    }
    //dup but difficult to make into template so ehh
    match &array.elem_type {
        PTypeView::ByteType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_byte()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::CharType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_char()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::DoubleType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_double()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::FloatType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_float()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::IntType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_int()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::LongType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_long()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::ShortType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_short()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::BooleanType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_boolean()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::Ref(_) => {
            let res = array.mut_array().iter().map(|elem| to_object(elem.unwrap_object())).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        _ => panic!(),
    }
}

pub unsafe extern "C" fn get_byte_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jbyte {
    get_primitive_array_critical(env, array, is_copy) as *mut jbyte
}

pub unsafe extern "C" fn get_char_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jchar {
    get_primitive_array_critical(env, array, is_copy) as *mut jchar
}

pub unsafe extern "C" fn get_double_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jdouble {
    get_primitive_array_critical(env, array, is_copy) as *mut jdouble
}

pub unsafe extern "C" fn get_float_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jfloat {
    get_primitive_array_critical(env, array, is_copy) as *mut jfloat
}

pub unsafe extern "C" fn get_int_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jint {
    get_primitive_array_critical(env, array, is_copy) as *mut jint
}

pub unsafe extern "C" fn get_short_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jshort {
    get_primitive_array_critical(env, array, is_copy) as *mut jshort
}

pub unsafe extern "C" fn get_boolean_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jboolean {
    get_primitive_array_critical(env, array, is_copy) as *mut jboolean
}

pub unsafe extern "C" fn get_object_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jobject {
    get_primitive_array_critical(env, array, is_copy) as *mut jobject
}

pub unsafe extern "C" fn get_long_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jlong {
    get_primitive_array_critical(env, array, is_copy) as *mut jlong
}


pub unsafe extern "C" fn release_byte_array_elements(env: *mut JNIEnv, array: jbyteArray, elems: *mut jbyte, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_char_array_elements(env: *mut JNIEnv, array: jcharArray, elems: *mut jchar, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_double_array_elements(env: *mut JNIEnv, array: jdoubleArray, elems: *mut jdouble, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_float_array_elements(env: *mut JNIEnv, array: jfloatArray, elems: *mut jfloat, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_int_array_elements(env: *mut JNIEnv, array: jintArray, elems: *mut jint, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_short_array_elements(env: *mut JNIEnv, array: jshortArray, elems: *mut jshort, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_boolean_array_elements(env: *mut JNIEnv, array: jbooleanArray, elems: *mut jboolean, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_object_array_elements(env: *mut JNIEnv, array: jobjectArray, elems: *mut jobject, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}

pub unsafe extern "C" fn release_long_array_elements(env: *mut JNIEnv, array: jlongArray, elems: *mut jlong, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}