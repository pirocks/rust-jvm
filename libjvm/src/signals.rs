use std::ffi::CStr;
use std::mem::transmute;
use std::os::raw::{c_char, c_void};

use jvmti_jni_bindings::{jboolean, jint};

#[no_mangle]
unsafe extern "system" fn JVM_RegisterSignal(sig: jint, handler: *mut c_void) -> *mut c_void {
    //todo unimpl for now
    transmute(0xdeaddeadbeafdead as usize)
}

#[no_mangle]
unsafe extern "system" fn JVM_RaiseSignal(sig: jint) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindSignal(name: *const c_char) -> jint {
    let name = CStr::from_ptr(name);
    if name.to_bytes() == b"HUP" {
        1 //todo bindgen signal.h
    } else if name.to_bytes() == b"INT" {
        2 //todo bindgen signal.h
    } else if name.to_bytes() == b"TERM" {
        15 //todo bindgen signal.h
    } else {
        unimplemented!()
    }
}
