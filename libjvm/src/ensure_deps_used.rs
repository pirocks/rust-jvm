use std::ffi::c_void;
use sun_misc_unsafe::object_access::Java_sun_misc_Unsafe_registerNatives;

#[no_mangle]
pub fn __rust_jvm_use_deps(){
    std::hint::black_box(Java_sun_misc_Unsafe_registerNatives);
}
