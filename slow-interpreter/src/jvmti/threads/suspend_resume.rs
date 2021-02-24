use jvmti_jni_bindings::{jint, jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT, jvmtiError_JVMTI_ERROR_INVALID_THREAD, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE, jvmtiError_JVMTI_ERROR_THREAD_NOT_SUSPENDED, jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED};

use crate::interpreter_state::{InterpreterStateGuard, SuspendedStatus};
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::native_util::from_object;
use crate::threading::SuspendError;

///Suspend Thread List
///
///     jvmtiError
///     SuspendThreadList(jvmtiEnv* env,
///                 jint request_count,
///                 const jthread* request_list,
///                 jvmtiError* results)
///
/// Suspend the request_count threads specified in the request_list array. Threads may be resumed with ResumeThreadList or ResumeThread.
/// If the calling thread is specified in the request_list array, this function will not return until some other thread resumes it.
/// Errors encountered in the suspension of a thread are returned in the results array, not in the return value of this function.
/// Threads that are currently suspended do not change state.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	92	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// request_count	jint	The number of threads to suspend.
/// request_list	const jthread*	The list of threads to suspend.
///
/// Agent passes in an array of request_count elements of jthread.
/// results	jvmtiError*	An agent supplied array of request_count elements. On return, filled with the error code for the suspend of the corresponding thread.
/// The error code will be JVMTI_ERROR_NONE if the thread was suspended by this call. Possible error codes are those specified for SuspendThread.
///
/// Agent passes an array large enough to hold request_count elements of jvmtiError.
/// The incoming values of the elements of the array are ignored. On return, the elements are set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	request_count is less than 0.
/// JVMTI_ERROR_NULL_POINTER	request_list is NULL.
/// JVMTI_ERROR_NULL_POINTER	results is NULL.
pub unsafe extern "C" fn suspend_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThreadList");
    null_check!(request_list);
    null_check!(results);
    assert!(jvm.vm_live());
    if request_count < 0 {
        return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT);
    }
    //todo handle checking capabilities
    for i in 0..request_count {
        let thread_object_raw = request_list.offset(i as isize).read();
        let suspend_res = suspend_thread_impl(thread_object_raw, jvm, int_state);
        results.offset(i as isize).write(suspend_res);
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

unsafe fn suspend_thread_impl(thread_object_raw: jthread, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> jvmtiError {
    let jthread = get_thread_or_error!(thread_object_raw);
    let java_thread = jthread.get_java_thread(jvm);
    match java_thread.suspend_thread(int_state) {
        Ok(_) => jvmtiError_JVMTI_ERROR_NONE,
        Err(err) => match err {
            SuspendError::AlreadySuspended => jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED,
            SuspendError::NotAlive => jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE
        }
    }
}


pub unsafe extern "C" fn interrupt_thread(env: *mut jvmtiEnv, _thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    let _tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    unimplemented!();
    // jvm.tracing.trace_jdwp_function_exit(tracing_guard, suspend_thread(env, thread))//todo this is an ugly hack.
}

///Suspend Thread
///
///     jvmtiError
///     SuspendThread(jvmtiEnv* env,
///                 jthread thread)
///
/// Suspend the specified thread.
/// If the calling thread is specified, this function will not return until some other thread calls ResumeThread.
/// If the thread is currently suspended, this function does nothing and returns an error.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	5	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to suspend. If thread is NULL, the current thread is used.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_THREAD_SUSPENDED	Thread already suspended.
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
pub unsafe extern "C" fn suspend_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    //todo check capabilities
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    let res = suspend_thread_impl(thread, jvm, int_state);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, res)
}

///Resume Thread
///
///     jvmtiError
///     ResumeThread(jvmtiEnv* env,
///                 jthread thread)
///
/// Resume a suspended thread. Any threads currently suspended through a JVM TI suspend function (eg. SuspendThread) or java.lang.Thread.suspend() will resume execution; all other threads are unaffected.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	6	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to resume.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_THREAD_NOT_SUSPENDED	Thread was not suspended.
/// JVMTI_ERROR_INVALID_TYPESTATE	The state of the thread has been modified, and is now inconsistent.
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
pub unsafe extern "C" fn resume_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    //todo handle capabilities for this
    assert!(jvm.vm_live());
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThread");
    let res = resume_thread_impl(jvm, thread);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, res)
}

/// Resume Thread List
///
/// jvmtiError
/// ResumeThreadList(jvmtiEnv* env,
/// jint request_count,
/// const jthread* request_list,
/// jvmtiError* results)
///
/// Resume the request_count threads specified in the request_list array. Any thread suspended through a JVM TI suspend function (eg. SuspendThreadList) or java.lang.Thread.suspend() will resume execution.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	93	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_suspend	Can suspend and resume threads
///
/// Parameters
/// Name 	Type 	Description
/// request_count	jint	The number of threads to resume.
/// request_list	const jthread*	The threads to resume.
///
/// Agent passes in an array of request_count elements of jthread.
/// results	jvmtiError*	An agent supplied array of request_count elements.
/// On return, filled with the error code for the resume of the corresponding thread.
/// The error code will be JVMTI_ERROR_NONE if the thread was suspended by this call. Possible error codes are those specified for ResumeThread.
///
/// Agent passes an array large enough to hold request_count elements of jvmtiError. The incoming values of the elements of the array are ignored. On return, the elements are set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_suspend. Use AddCapabilities.
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	request_count is less than 0.
/// JVMTI_ERROR_NULL_POINTER	request_list is NULL.
/// JVMTI_ERROR_NULL_POINTER	results is NULL.
pub unsafe extern "C" fn resume_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThreadList");
    assert!(jvm.vm_live());
    null_check!(request_list);
    null_check!(results);
    if request_count < 0 {
        return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT;
    }
    //todo handle capabilities;
    for i in 0..request_count {
        let jthreadp = request_list.offset(i as isize).read();
        results.offset(i as isize).write(resume_thread_impl(jvm, jthreadp));
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


unsafe fn resume_thread_impl(jvm: &JVMState, thread_raw: jthread) -> jvmtiError {
    let thread_object_raw = from_object(thread_raw);
    let jthread = match JavaValue::Object(thread_object_raw).try_cast_thread() {
        None => {
            assert!(false);
            return jvmtiError_JVMTI_ERROR_INVALID_THREAD;
        }
        Some(jthread) => jthread,
    };
    let java_thread = jthread.get_java_thread(jvm);
    let SuspendedStatus { suspended, suspend_condvar } = &java_thread.suspended;
    let mut suspend_guard = suspended.lock().unwrap();
    if !*suspend_guard {
        jvmtiError_JVMTI_ERROR_THREAD_NOT_SUSPENDED
    } else {
        *suspend_guard = false;
        suspend_condvar.notify_one();//notify one and notify all should be the same here
        jvmtiError_JVMTI_ERROR_NONE
    }
}
