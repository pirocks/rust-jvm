use jvmti_jni_bindings::{JNIEnv, jobject, jweak};

use crate::rust_jni::native_util::{from_object, get_state, to_object};

static mut TIMES: usize = 0;

pub unsafe extern "C" fn new_global_ref(env: *mut JNIEnv, lobj: jobject) -> jobject {
    let jvm = get_state(env);
    let obj = from_object(jvm, lobj);
    match &obj {
        None => {}
        Some(o) => {
            TIMES += 1;
            if TIMES % 1000000 == 0 {
                dbg!(TIMES);
            }
            Box::leak(Box::new(o.clone()));
        }
    }
    to_object(obj)
}

pub unsafe extern "C" fn new_weak_global_ref(env: *mut JNIEnv, lobj: jobject) -> jweak {
    let jvm = get_state(env);
    let obj = from_object(jvm, lobj);
    match &obj {
        None => {}
        Some(o) => {
            TIMES += 1;
            if TIMES % 1000000 == 0 {
                dbg!(TIMES);
            }
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