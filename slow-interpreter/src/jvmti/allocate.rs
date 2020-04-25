use jvmti_bindings::{jvmtiEnv, jvmtiError, jlong, jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY, jvmtiError_JVMTI_ERROR_NONE};

pub unsafe extern "C" fn allocate(_env: *mut jvmtiEnv, size: jlong, mem_ptr: *mut *mut ::std::os::raw::c_uchar) -> jvmtiError{
    if size == 0{
        unimplemented!()
        // *mem_ptr = transmute(0xDEADDEADBEAF as usize);//just need to return a non-zero address//todo how isfreeing this handled?
        // return jvmtiError_JVMTI_ERROR_NONE
    }
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

