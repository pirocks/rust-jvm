use std::intrinsics::atomic_cxchg_seqcst_seqcst;

use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, JNIEnv, jobject, jshort};
use crate::double_register_addressing::calc_address;


#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapBoolean(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jboolean, new: jboolean) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jboolean>(), old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapByte(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jbyte, new: jbyte) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jbyte>(), old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapShort(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jshort, new: jshort) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jshort>(), old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapChar(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jchar, new: jchar) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jchar>(), old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapInt(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jint, new: jint) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<jint>(), old, new).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapFloat(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jfloat, new: jfloat) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<u32>(), old.to_bits(), new.to_bits()).1 as jboolean
}

#[no_mangle]
unsafe extern "system" fn Java_sun_misc_Unsafe_compareAndSwapDouble(_env: *mut JNIEnv, _the_unsafe: jobject, target_obj: jobject, offset: jlong, old: jdouble, new: jdouble) -> jboolean {
    atomic_cxchg_seqcst_seqcst(calc_address(target_obj, offset).cast::<u64>(), old.to_bits(), new.to_bits()).1 as jboolean
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

