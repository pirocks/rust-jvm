use std::ffi::{c_void, CStr};

use jvmti_jni_bindings::{fopen, jint, jlong};

use crate::util::retry_on_eintr;

#[no_mangle]
unsafe extern "system" fn JVM_NativePath(arg1: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char {
    arg1
}

#[no_mangle]
unsafe extern "system" fn JVM_Open(fname: *const ::std::os::raw::c_char, flags: jint, mode: jint) -> jint {
    libc::open(fname, mode)
}


#[no_mangle]
unsafe extern "system" fn JVM_Close(fd: jint) -> jint {
    libc::close(fd)
}

#[no_mangle]
unsafe extern "system" fn JVM_Read(fd: jint, buf: *mut ::std::os::raw::c_char, nbytes: jint) -> jint {
    retry_on_eintr(|| libc::read(fd, buf as *mut c_void, nbytes as usize) as i32)
}

#[no_mangle]
unsafe extern "system" fn JVM_Write(fd: jint, buf: *mut ::std::os::raw::c_char, nbytes: jint) -> jint {
    retry_on_eintr(|| libc::write(fd, buf as *mut c_void, nbytes as usize) as i32)
}

#[no_mangle]
unsafe extern "system" fn JVM_Available(fd: jint, pbytes: *mut jlong) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Lseek(fd: jint, offset: jlong, whence: jint) -> jlong {
    libc::lseek(fd, offset, whence)
}

#[no_mangle]
unsafe extern "system" fn JVM_Sync(fd: jint) -> jint {
    libc::fsync(fd)
}


#[no_mangle]
unsafe extern "system" fn JVM_SetLength(fd: jint, length: jlong) -> jint {
    unimplemented!()
}
