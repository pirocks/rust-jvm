use std::os::raw::c_void;

use jvmti_jni_bindings::{jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::jvmti::get_state;
use crate::rust_jni::native_util::from_object;

///Get Thread Local Storage
///
///     jvmtiError
///     GetThreadLocalStorage(jvmtiEnv* env,
///                 jthread thread,
///                 void** data_ptr)
///
/// Called by the agent to get the value of the JVM TI thread-local storage.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	102	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	Retrieve from this thread. If thread is NULL, the current thread is used.
/// data_ptr	void**	Pointer through which the value of the thread local storage is returned. If thread-local storage has not been set with SetThreadLocalStorage the returned pointer is NULL.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
/// JVMTI_ERROR_NULL_POINTER	data_ptr is NULL.
pub unsafe extern "C" fn get_thread_local_storage(env: *mut jvmtiEnv, thread: jthread, data_ptr: *mut *mut ::std::os::raw::c_void) -> jvmtiError {
    let jvm = get_state(env);
    assert!(jvm.vm_live());
    null_check!(data_ptr);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadLocalStorage");
    let java_thread = get_thread_or_error!(thread).get_java_thread(jvm);
    data_ptr.write(*java_thread.thread_local_storage.read().unwrap());
    //todo so I'm not sure thread's aliveness is all that relevant in this implementation
    // if !java_thread.is_alive() {
    //     return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE);
    // }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Set Thread Local Storage
///
///     jvmtiError
///     SetThreadLocalStorage(jvmtiEnv* env,
///                 jthread thread,
///                 const void* data)
///
/// The VM stores a pointer value associated with each environment-thread pair.
/// This pointer value is called thread-local storage.
/// This value is NULL unless set with this function.
/// Agents can allocate memory in which they store thread specific information.
/// By setting thread-local storage it can then be accessed with GetThreadLocalStorage.
///
/// This function is called by the agent to set the value of the JVM TI thread-local storage.
/// JVM TI supplies to the agent a pointer-size thread-local storage that can be used to record per-thread information.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	103	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	Store to this thread. If thread is NULL, the current thread is used.
/// data	const void *	The value to be entered into the thread-local storage.
///
/// Agent passes in a pointer. If data is NULL, value is set to NULL.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
pub unsafe extern "C" fn set_thread_local_storage(env: *mut jvmtiEnv, thread: jthread, data: *const ::std::os::raw::c_void) -> jvmtiError {
    let jvm = get_state(env);
    assert!(jvm.vm_live());
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SetThreadLocalStorage");
    let java_thread = get_thread_or_error!(thread).get_java_thread(jvm);
    *java_thread.thread_local_storage.write().unwrap() = data as *mut c_void;
    //todo so I'm not sure thread's aliveness is all that relevant in this implementation
    // if !java_thread.is_alive() {
    //     return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE);
    // }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
