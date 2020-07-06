use std::mem::transmute;
use std::os::raw::c_char;

use jvmti_jni_bindings::{jboolean, jint};

#[no_mangle]
unsafe extern "system" fn JVM_RegisterSignal(sig: jint, handler: *mut ::std::os::raw::c_void) -> *mut ::std::os::raw::c_void {
    //todo unimpl for now
    transmute(0xdeaddeadbeafdead as usize)
}

#[no_mangle]
unsafe extern "system" fn JVM_RaiseSignal(sig: jint) -> jboolean {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_FindSignal(name: *const ::std::os::raw::c_char) -> jint {
    if name.offset(0).read() == 'H' as c_char && name.offset(1).read() == 'U' as c_char && name.offset(2).read() == 'P' as c_char {
        1 //todo bindgen signal.h
    } else if name.offset(0).read() == 'I' as c_char && name.offset(1).read() == 'N' as c_char && name.offset(2).read() == 'T' as c_char {
        2 //todo bindgen signal.h
    } else if name.offset(0).read() == 'T' as c_char && name.offset(1).read() == 'E' as c_char && name.offset(2).read() == 'R' as c_char && name.offset(3).read() == 'M' as c_char {
        15 //todo bindgen signal.h
    } else {
        unimplemented!()
    }
}