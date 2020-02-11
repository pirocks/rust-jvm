use crate::rust_jni::native_util::{to_object, from_object};
use jni_bindings::{jobject, JNIEnv};

pub unsafe extern "C" fn new_global_ref(_env: *mut JNIEnv, lobj: jobject) -> jobject {
    let obj = from_object(lobj);
    match &obj {
        None => {}
        Some(o) => {
            Box::leak(Box::new(o.clone()));
        }
    }
    to_object(obj)
}

pub unsafe extern "C" fn delete_local_ref(_env: *mut JNIEnv, _obj: jobject) {
    //todo no gc, just leak
}

