use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::ptr::null_mut;
use std::sync::Arc;

use jvmti_jni_bindings::{jint, JNI_OK, JNIEnv, jobject};

use crate::java_values::Object;
use crate::rust_jni::native_util::from_object;

thread_local! {
static JNI_INTERFACE_LOCAL_REF: RefCell<Vec<HashMap<jobject,Arc<Object>>>> = RefCell::new(vec![HashMap::new()]);
}


pub fn get_local_refs<T>(to_run: &dyn Fn(&mut Vec<HashMap<jobject, Arc<Object>>>) -> T) -> T {
    JNI_INTERFACE_LOCAL_REF.with(|refcell| {
        to_run(refcell.borrow_mut().deref_mut())
    })
}

pub fn clear_local_refs(up_to_len: usize) {
    get_local_refs(&move |local_frames| {
        drop(local_frames.drain(up_to_len..).collect::<Vec<_>>());
    })
}

pub fn local_refs_len() -> usize {
    get_local_refs(&move |local_frames| {
        local_frames.len()
    })
}

pub unsafe extern "C" fn pop_local_frame(_env: *mut JNIEnv, result: jobject) -> jobject {
    get_local_refs(&move |local_frames| {
        let popped = local_frames.pop().unwrap();
        if result == std::ptr::null_mut() {
            null_mut()
        } else {
            let to_be_preserved = popped.get(&result).unwrap();
            local_frames.last_mut().unwrap().insert(result, to_be_preserved.clone());
            result
        }
    })
}

pub unsafe extern "C" fn push_local_frame(_env: *mut JNIEnv, _capacity: jint) -> jint {
    get_local_refs(&move |local_frames| {
        local_frames.push(HashMap::new());
        JNI_OK as i32
    })
}


pub unsafe extern "C" fn new_local_ref(_env: *mut JNIEnv, ref_: jobject) -> jobject {
    get_local_refs(&move |local_frames| {
        local_frames.last_mut().unwrap().insert(ref_, from_object(ref_).unwrap());
        ref_
    })
}


pub unsafe extern "C" fn delete_local_ref(_env: *mut JNIEnv, obj: jobject) {
    get_local_refs(&move |local_frames| {
        local_frames.last_mut().unwrap().remove(&obj);
    })
}