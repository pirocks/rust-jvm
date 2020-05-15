use jvmti_jni_bindings::{JNIEnv, jbyte, jsize, jbyteArray, jarray};
use std::cell::RefCell;
use std::sync::Arc;
use crate::rust_jni::native_util::{to_object, from_object, get_state};
use std::ops::Deref;
use classfile_view::view::ptype_view::PTypeView;
use crate::java_values::{JavaValue, Object, ArrayObject, default_value};


pub unsafe extern "C" fn get_array_length(_env: *mut JNIEnv, array: jarray) -> jsize {
    let non_null_array: &Object = &from_object(array).unwrap();
    let len = match non_null_array {
        Object::Array(a) => {
            a.elems.borrow().len()
        }
        Object::Object(_o) => {
            unimplemented!()
        }
    };
    len as jsize
}

pub mod array_region;
pub mod new;


