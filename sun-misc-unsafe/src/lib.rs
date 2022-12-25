#![feature(core_intrinsics)]

use libc::{_SC_PAGESIZE};
use jvmti_jni_bindings::{jclass, jint, JNIEnv, jobject};

pub mod compare_and_swap;
pub mod define_anonymous_class;
pub mod object_access_volatile;
pub mod object_access_offsets;
pub mod object_access_normal;
pub mod reflection;
pub mod raw_pointer;
pub mod define_class;
pub mod park;
pub mod double_register_addressing;
pub mod fences;
pub mod initializing;


#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(_env: *mut JNIEnv, _cb: jclass) {
    //todo for now register nothing, register later as needed.
}

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_pageSize(_env: *mut JNIEnv, _the_unsafe: jobject) -> jint {
    libc::sysconf(_SC_PAGESIZE) as jint
}