use std::intrinsics::atomic_cxchg_seqcst_seqcst;
use std::ptr::null_mut;

use libc::c_void;

use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jint, new: jint) -> jboolean {
    if target_obj == null_mut() {
        todo!()
    }
    atomic_cxchg_seqcst_seqcst((target_obj as *mut c_void).offset(offset as isize) as *mut jint, old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jlong, new: jlong) -> jboolean {
    if target_obj == null_mut() {
        todo!()
    }
    atomic_cxchg_seqcst_seqcst((target_obj as *mut c_void).offset(offset as isize) as *mut jlong, old, new).1 as jboolean
}


#[no_mangle]
unsafe extern "C" fn Java_sun_misc_Unsafe_compareAndSwapObject(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, expected: jobject, new: jobject) -> jboolean {
    //todo make these intrinsics
    if target_obj == null_mut() {
        todo!()
    }
    let target = (target_obj as *mut c_void).offset(offset as isize) as *mut jobject;
    atomic_cxchg_seqcst_seqcst(target, expected, new).1 as jboolean
}

