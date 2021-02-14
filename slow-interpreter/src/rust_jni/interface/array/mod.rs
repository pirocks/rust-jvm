use std::os::raw::c_void;

use classfile_view::view::ptype_view::PTypeView;
use jvmti_jni_bindings::{jarray, jboolean, jbyte, jint, jlong, jlongArray, JNI_ABORT, JNIEnv, jobject, jobjectArray, jsize};

use crate::java_values::{JavaValue, Object};
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::{from_object, get_interpreter_state};

pub unsafe extern "C" fn get_array_length(_env: *mut JNIEnv, array: jarray) -> jsize {
    let non_null_array: &Object = &from_object(array).unwrap();
    let len = match non_null_array {
        Object::Array(a) => {
            a.mut_array().len()
        }
        Object::Object(_o) => {
            unimplemented!()
        }
    };
    len as jsize
}

pub unsafe extern "C" fn get_object_array_element(env: *mut JNIEnv, array: jobjectArray, index: jsize) -> jobject {
    let notnull = from_object(array).unwrap();
    let int_state = get_interpreter_state(env);
    let array = notnull.unwrap_array();
    let borrow = array.mut_array();
    new_local_ref_public(borrow[index as usize].unwrap_object(), int_state)
}

pub unsafe extern "C" fn set_object_array_element(_env: *mut JNIEnv, array: jobjectArray, index: jsize, val: jobject) {
    let notnull = from_object(array).unwrap();
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
    let not_null = from_object(array).unwrap();
    let array = not_null.unwrap_array();
    let array_type = &array.elem_type;
    let array = array.mut_array();
    for (i, elem) in array.iter_mut().enumerate() {
        match array_type {
            PTypeView::ByteType => {
                *elem = JavaValue::Byte((carray as *const jbyte).offset(i as isize).read());
            }
            PTypeView::CharType => todo!(),
            PTypeView::DoubleType => todo!(),
            PTypeView::FloatType => todo!(),
            PTypeView::IntType => {
                *elem = JavaValue::Int((carray as *const jint).offset(i as isize).read());
            }
            PTypeView::LongType => {
                *elem = JavaValue::Long((carray as *const jlong).offset(i as isize).read());
            }
            PTypeView::Ref(_) => todo!(),
            PTypeView::ShortType => todo!(),
            PTypeView::BooleanType => todo!(),
            PTypeView::VoidType => todo!(),
            _ => todo!()
        }
    }
}

pub unsafe extern "C" fn get_primitive_array_critical(_env: *mut JNIEnv, array: jarray, is_copy: *mut jboolean) -> *mut c_void {
    let not_null = from_object(array).unwrap();
    let array = not_null.unwrap_array();
    if !is_copy.is_null() {
        is_copy.write(true as jboolean);
    }
    match array.elem_type {
        PTypeView::ByteType => {
            let res = array.mut_array().iter().map(|elem| elem.unwrap_byte()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::CharType => todo!(),
        PTypeView::DoubleType => todo!(),
        PTypeView::FloatType => todo!(),
        PTypeView::IntType => todo!(),
        PTypeView::LongType => {
            //todo dup
            let res = array.mut_array().iter().map(|elem| elem.unwrap_long()).collect::<Vec<_>>();
            return res.leak().as_mut_ptr() as *mut c_void;
        }
        PTypeView::Ref(_) => todo!(),
        PTypeView::ShortType => todo!(),
        PTypeView::BooleanType => todo!(),
        PTypeView::VoidType => todo!(),
        PTypeView::TopType => todo!(),
        PTypeView::NullType => todo!(),
        PTypeView::Uninitialized(_) => todo!(),
        PTypeView::UninitializedThis => todo!(),
        PTypeView::UninitializedThisOrClass(_) => todo!(),
    }
}


pub unsafe extern "C" fn get_long_array_elements(env: *mut JNIEnv, array: jlongArray, is_copy: *mut jboolean) -> *mut jlong {
    get_primitive_array_critical(env, array, is_copy) as *mut jlong
}

pub unsafe extern "C" fn release_long_array_elements(env: *mut JNIEnv, array: jlongArray, elems: *mut jlong, mode: jint) {
    release_primitive_array_critical(env, array, elems as *mut c_void, mode)
}