use jvmti_jni_bindings::{JNIEnv, jsize, jarray, jobjectArray, jobject};
use crate::rust_jni::native_util::{from_object, to_object};
use crate::java_values::Object;


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

pub unsafe extern "C" fn get_object_array_element(_env: *mut JNIEnv, array: jobjectArray, index: jsize) -> jobject{
    let notnull = from_object(array).unwrap();
    let array = notnull.unwrap_array();
    let borrow = array.elems.borrow();
    to_object(borrow[index as usize].unwrap_object().clone())
}

pub unsafe extern "C" fn set_object_array_element(_env: *mut JNIEnv, array: jobjectArray, index: jsize, val: jobject){
    let notnull = from_object(array).unwrap();
    let array = notnull.unwrap_array();
    let mut borrow_mut = array.elems.borrow_mut();
    borrow_mut[index as usize] = from_object(val).into();
}

pub mod array_region;
pub mod new;


