use std::cell::RefMut;
use std::os::raw::c_void;

use jvmti_jni_bindings::{jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::jvmti::get_state;

pub unsafe extern "C" fn get_thread_local_storage(env: *mut jvmtiEnv, thread: jthread, data_ptr: *mut *mut ::std::os::raw::c_void) -> jvmtiError {
    let jvm = get_state(env);
    //todo this is wrong b/c it ignores thread
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadLocalStorage");
    jvm.jvmti_state.as_ref().unwrap().jvmti_thread_local_storage.with(|tls_ptr| {
        data_ptr.write(*tls_ptr.borrow());
    });
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetThreadLocalStorage");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn set_thread_local_storage(env: *mut jvmtiEnv, _thread: jthread, data: *const ::std::os::raw::c_void) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SetThreadLocalStorage");

    jvm.jvmti_state.as_ref().unwrap().jvmti_thread_local_storage.with(|tls_ptr| {
        let mut ref_mut: RefMut<*mut c_void> = tls_ptr.borrow_mut();
        *ref_mut = data as *mut c_void;
    });
    jvm.tracing.trace_jdwp_function_exit(jvm, "SetThreadLocalStorage");
    jvmtiError_JVMTI_ERROR_NONE
}
