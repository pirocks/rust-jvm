use jvmti_jni_bindings::{jint, jlong};

#[no_mangle]
unsafe extern "system" fn JVM_NativePath(arg1: *mut ::std::os::raw::c_char) -> *mut ::std::os::raw::c_char {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Open(fname: *const ::std::os::raw::c_char, flags: jint, mode: jint) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_Close(fd: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Read(fd: jint, buf: *mut ::std::os::raw::c_char, nbytes: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Write(fd: jint, buf: *mut ::std::os::raw::c_char, nbytes: jint) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Available(fd: jint, pbytes: *mut jlong) -> jint {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Lseek(fd: jint, offset: jlong, whence: jint) -> jlong {
    unimplemented!()
}

#[no_mangle]
unsafe extern "system" fn JVM_Sync(fd: jint) -> jint {
    unimplemented!()
}


#[no_mangle]
unsafe extern "system" fn JVM_SetLength(fd: jint, length: jlong) -> jint {
    unimplemented!()
}
