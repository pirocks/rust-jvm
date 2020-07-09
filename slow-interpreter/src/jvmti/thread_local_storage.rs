use std::cell::RefMut;
use std::os::raw::c_void;

use jvmti_jni_bindings::{jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::java_values::JavaValue;
use crate::jvmti::get_state;
use crate::rust_jni::native_util::from_object;

pub unsafe extern "C" fn get_thread_local_storage(env: *mut jvmtiEnv, thread: jthread, data_ptr: *mut *mut ::std::os::raw::c_void) -> jvmtiError {
    let jvm = get_state(env);
    //todo this is wrong b/c it ignores thread
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadLocalStorage");
    let java_thread = JavaValue::Object(from_object(thread)).cast_thread().get_java_thread(jvm);
    data_ptr.write(*java_thread.thread_local_storage.read().unwrap());
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetThreadLocalStorage");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn set_thread_local_storage(env: *mut jvmtiEnv, thread: jthread, data: *const ::std::os::raw::c_void) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SetThreadLocalStorage");
    let java_thread = JavaValue::Object(from_object(thread)).cast_thread().get_java_thread(jvm);
    *java_thread.thread_local_storage.write().unwrap() = data as *mut c_void;
    jvm.tracing.trace_jdwp_function_exit(jvm, "SetThreadLocalStorage");
    jvmtiError_JVMTI_ERROR_NONE
}
