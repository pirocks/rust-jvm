use jvmti_jni_bindings::{JNIEnv, jsize, jarray};
use crate::rust_jni::native_util::from_object;
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

pub mod array_region;
pub mod new;


