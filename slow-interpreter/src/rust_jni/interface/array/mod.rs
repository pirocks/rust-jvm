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


pub unsafe extern "C" fn set_byte_array_region(_env: *mut JNIEnv, array: jbyteArray, start: jsize, len: jsize, buf: *const jbyte) {
    for i in 0..len {
        from_object(array)
            .unwrap()
            .unwrap_array()
            .elems
            .borrow_mut()
            .insert((start + i) as usize, JavaValue::Byte(buf.offset(i as isize).read() as i8));
    }
}