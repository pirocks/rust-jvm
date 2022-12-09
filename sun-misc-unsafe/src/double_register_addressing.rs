use std::ffi::c_void;
use jvmti_jni_bindings::{jlong, jobject};

pub unsafe fn calc_address(obj: jobject, offset: jlong) -> *mut c_void{
    obj.cast::<c_void>().offset(offset as isize)
}
