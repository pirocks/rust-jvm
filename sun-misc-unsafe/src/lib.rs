#![feature(core_intrinsics)]

use jvmti_jni_bindings::{jclass, JNIEnv};

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

#[no_mangle]
pub unsafe extern "system" fn Java_sun_misc_Unsafe_registerNatives(_env: *mut JNIEnv, _cb: jclass) {
    //todo for now register nothing, register later as needed.
}