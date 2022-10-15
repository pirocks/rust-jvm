use std::ffi::c_void;

use jvmti_jni_bindings::{jlong, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT, jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY};
use slow_interpreter::rust_jni::jvmti::{get_state};

pub unsafe extern "C" fn allocate(env: *mut jvmtiEnv, size: jlong, mem_ptr: *mut *mut ::std::os::raw::c_uchar) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "Allocate");
    if size < 0 {
        return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT;
    }
    *mem_ptr = jvm.native.native_interface_allocations.allocate_malloc(size as usize) as *mut ::std::os::raw::c_uchar;
    let res = if (*mem_ptr).is_null() { jvmtiError_JVMTI_ERROR_OUT_OF_MEMORY } else { jvmtiError_JVMTI_ERROR_NONE };
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, res)
}

pub unsafe extern "C" fn deallocate(env: *mut jvmtiEnv, mem: *mut ::std::os::raw::c_uchar) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "Deallocate");
    jvm.native.native_interface_allocations.free(mem as *mut c_void);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn dispose_environment(env: *mut jvmtiEnv) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "DisposeEnvironment");
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY)
}
