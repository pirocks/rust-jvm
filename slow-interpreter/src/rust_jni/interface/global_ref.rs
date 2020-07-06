use jvmti_jni_bindings::{JNIEnv, jobject, jweak};

use crate::rust_jni::native_util::{from_object, to_object};

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

pub unsafe extern "C" fn new_weak_global_ref(_env: *mut JNIEnv, lobj: jobject) -> jweak {
    let obj = from_object(lobj);
    match &obj {
        None => {}
        Some(o) => {
            Box::leak(Box::new(o.clone()));
        }
    }
    to_object(obj)
}


pub unsafe extern "C" fn delete_global_ref(_env: *mut JNIEnv, _gref: jobject) {
    //todo blocking on having a gc
}

pub unsafe extern "C" fn delete_weak_global_ref(_env: *mut JNIEnv, _ref_: jweak) {
    //todo blocking on having a gc
}