use jvmti_jni_bindings::{jlong, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY};

use crate::jvmti::get_state;

pub unsafe extern "C" fn allocate(env: *mut jvmtiEnv, size: jlong, mem_ptr: *mut *mut ::std::os::raw::c_uchar) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "Allocate");
    if size == 0 {
        unimplemented!()
        // *mem_ptr = transmute(0xDEADDEADBEAF as usize);//just need to return a non-zero address//todo how isfreeing this handled?
        // return jvmtiError_JVMTI_ERROR_NONE
    }
    *mem_ptr = libc::malloc(size as usize) as *mut ::std::os::raw::c_uchar;
    let res = if *mem_ptr == std::ptr::null_mut() {
        jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY
    } else {
        jvmtiError_JVMTI_ERROR_NONE
    };
    jvm.tracing.trace_jdwp_function_exit(jvm, "Allocate");
    res
}

pub unsafe extern "C" fn deallocate(env: *mut jvmtiEnv, _mem: *mut ::std::os::raw::c_uchar) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "Deallocate");
    jvm.tracing.trace_jdwp_function_exit(jvm, "Deallocate");
    jvmtiError_JVMTI_ERROR_NONE//todo currently leaks a lot
}

