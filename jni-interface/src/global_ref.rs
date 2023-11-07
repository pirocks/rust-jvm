use jvmti_jni_bindings::{JNIEnv, jobject, jweak};

use slow_interpreter::rust_jni::native_util::{from_object_new, to_object_new};
use slow_interpreter::rust_jni::jni_utils::{get_state};

static mut TIMES: usize = 0;

pub unsafe extern "C" fn new_global_ref(env: *mut JNIEnv, lobj: jobject) -> jobject {
    let jvm = get_state(env);
    let obj = from_object_new(jvm, lobj);
    match &obj {
        None => {}
        Some(o) => {
            TIMES += 1;
            if TIMES % 1000000 == 0 {
                dbg!(TIMES);
            }
            Box::leak(Box::new(o));
        }
    }
    to_object_new(obj.as_ref().map(|handle| handle.as_allocated_obj()))
}

pub unsafe extern "C" fn new_weak_global_ref(env: *mut JNIEnv, lobj: jobject) -> jweak {
    let jvm = get_state(env);
    let obj = from_object_new(jvm, lobj);
    to_object_new(obj.as_ref().map(|inner|inner.as_allocated_obj()))
}

pub unsafe extern "C" fn delete_global_ref(_env: *mut JNIEnv, _gref: jobject) {
    //todo blocking on having a gc
}

pub unsafe extern "C" fn delete_weak_global_ref(_env: *mut JNIEnv, _ref_: jweak) {
    //todo blocking on having a gc
}