use jvmti_bindings::{jvmtiEnv, jvmtiError, jlong, jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY, jvmtiError_JVMTI_ERROR_NONE};

pub unsafe extern "C" fn allocate(_env: *mut jvmtiEnv, size: jlong, mem_ptr: *mut *mut ::std::os::raw::c_uchar) -> jvmtiError{
    *mem_ptr = libc::malloc(size as usize) as *mut ::std::os::raw::c_uchar;
    if *mem_ptr == std::ptr::null_mut(){
        jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY
    }else {
        jvmtiError_JVMTI_ERROR_NONE
    }
}

pub unsafe extern "C" fn deallocate(_env: *mut jvmtiEnv, _mem: *mut ::std::os::raw::c_uchar) -> jvmtiError{
    jvmtiError_JVMTI_ERROR_NONE//todo currently leaks a lot
}

