use std::ffi::CStr;
use std::mem::transmute;
use std::os::raw::{c_int, c_void};

use jvmti_jni_bindings::{JavaVM, JNI_VERSION_1_8};

#[no_mangle]
unsafe extern "system" fn JVM_LoadLibrary(name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_UnloadLibrary(handle: *mut ::std::os::raw::c_void) {
    unimplemented!()
}

unsafe extern "system" fn provide_jni_version(jvm: *mut *mut JavaVM, something: *mut c_void) -> c_int {
    //todo I'm confused as to why this is returned from JVM_FindLibraryEntry, and I wrote this
    JNI_VERSION_1_8 as c_int
}

#[no_mangle]
unsafe extern "system" fn JVM_FindLibraryEntry(handle: *mut ::std::os::raw::c_void, name: *const ::std::os::raw::c_char) -> *mut ::std::os::raw::c_void {
//    unimplemented!();
    //todo not implemented for now
    dbg!(CStr::from_ptr(name).to_str().unwrap());
    transmute(provide_jni_version as *mut c_void)
}
