use std::intrinsics::atomic_cxchg_seqcst_seqcst;

use jvmti_jni_bindings::{jboolean, jint, jlong, JNIEnv, jobject};
use crate::double_register_addressing::calc_address;


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jint, new: jint) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jint>(), old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapLong(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jlong, new: jlong) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jlong>(), old, new).1 as jboolean
}


#[no_mangle]
unsafe extern "C" fn Java_sun_misc_Unsafe_compareAndSwapObject(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, expected: jobject, new: jobject) -> jboolean {
    //todo make these intrinsics
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jobject>(), expected, new).1 as jboolean
}

