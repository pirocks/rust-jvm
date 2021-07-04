use std::ffi::CStr;
use std::intrinsics::transmute;
use std::os::raw::c_char;
use std::time::Duration;

use jvmti_jni_bindings::{jlong, jrawMonitorID, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_INVALID_MONITOR, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_NOT_MONITOR_OWNER};

use crate::jvmti::{get_interpreter_state, get_state};
use crate::threading::monitors::Monitor;
use crate::threading::safepoints::Monitor2;

pub unsafe fn monitor_to_raw(monitor: &Monitor2) -> jrawMonitorID {
    transmute(monitor.id)
}

/// Create Raw Monitor
///
/// jvmtiError
/// CreateRawMonitor(jvmtiEnv* env,
/// const char* name,
/// jrawMonitorID* monitor_ptr)
///
/// Create a raw monitor.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the OnLoad or the live phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	31	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// name	const char*	A name to identify the monitor, encoded as a modified UTF-8 string.
///
/// Agent passes in an array of char.
/// monitor_ptr	jrawMonitorID*	On return, points to the created monitor.
///
/// Agent passes a pointer to a jrawMonitorID. On return, the jrawMonitorID has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NULL_POINTER	name is NULL.
/// JVMTI_ERROR_NULL_POINTER	monitor_ptr is NULL.
pub unsafe extern "C" fn create_raw_monitor(env: *mut jvmtiEnv, name: *const c_char, monitor_ptr: *mut jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "CreateRawMonitor");
    null_check!(name);
    null_check!(monitor_ptr);
    let monitor_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let res_monitor = jvm.thread_state.new_monitor(monitor_name);
    monitor_ptr.write(monitor_to_raw(&res_monitor));
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


///Raw Monitor Enter
///
///     jvmtiError
///     RawMonitorEnter(jvmtiEnv* env,
///                 jrawMonitorID monitor)
///
/// Gain exclusive ownership of a raw monitor.
/// The same thread may enter a monitor more then once.
/// The thread must exit the monitor the same number of times as it is entered.
/// If a monitor is entered during OnLoad (before attached threads exist) and has not exited when attached threads come into existence, the enter is considered to have occurred on the main thread.
///
/// Phase	Callback Safe	Position	Since
/// may be called during any phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	33	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// monitor	jrawMonitorID	The monitor
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_MONITOR	monitor is not a jrawMonitorID.
pub unsafe extern "C" fn raw_monitor_enter(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorEnter");
    let monitor = match jvm.thread_state.try_get_monitor(monitor_id) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_MONITOR),
        Some(m) => m,
    };
    monitor.lock(jvm, get_interpreter_state(env)).unwrap();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


///Raw Monitor Exit
///
///     jvmtiError
///     RawMonitorExit(jvmtiEnv* env,
///                 jrawMonitorID monitor)
///
/// Release exclusive ownership of a raw monitor.
///
/// Phase	Callback Safe	Position	Since
/// may be called during any phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	34	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// monitor	jrawMonitorID	The monitor
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NOT_MONITOR_OWNER	Not monitor owner
/// JVMTI_ERROR_INVALID_MONITOR	monitor is not a jrawMonitorID.
pub unsafe extern "C" fn raw_monitor_exit(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorExit");
    let monitor = match jvm.thread_state.try_get_monitor(monitor_id) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_MONITOR),
        Some(m) => m,
    };
    monitor.unlock(jvm, get_interpreter_state(env)).unwrap();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


/// Raw Monitor Wait
///
/// jvmtiError
/// RawMonitorWait(jvmtiEnv* env,
/// jrawMonitorID monitor,
/// jlong millis)
///
/// Wait for notification of the raw monitor.
///
/// Causes the current thread to wait until either another thread calls RawMonitorNotify or RawMonitorNotifyAll for the specified raw monitor, or the specified timeout has elapsed.
///
/// Phase	Callback Safe	Position	Since
/// may be called during any phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	35	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// monitor	jrawMonitorID	The monitor
/// millis	jlong	The timeout, in milliseconds. If the timeout is zero, then real time is not taken into consideration and the thread simply waits until notified.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NOT_MONITOR_OWNER	Not monitor owner
/// JVMTI_ERROR_INTERRUPT	Wait was interrupted, try again
/// JVMTI_ERROR_INVALID_MONITOR	monitor is not a jrawMonitorID.
pub unsafe extern "C" fn raw_monitor_wait(env: *mut jvmtiEnv, monitor_id: jrawMonitorID, millis: jlong) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorWait");
    let monitor = match jvm.thread_state.try_get_monitor(monitor_id) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_MONITOR),
        Some(m) => m,
    };
    let duration = if millis == 0 {
        None
    } else {
        Some(Duration::from_millis(millis as u64))
    };//todo dup, everywhere we call wait
    monitor.wait(jvm, int_state, duration).unwrap();//todo handle interrupted waits at a later date
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


/// Raw Monitor Notify
///
/// jvmtiError
/// RawMonitorNotify(jvmtiEnv* env,
/// jrawMonitorID monitor)
///
/// Notify a single thread waiting on the raw monitor.
///
/// Phase	Callback Safe	Position	Since
/// may be called during any phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	36	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// monitor	jrawMonitorID	The monitor
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NOT_MONITOR_OWNER	Not monitor owner
/// JVMTI_ERROR_INVALID_MONITOR	monitor is not a jrawMonitorID.
pub unsafe extern "C" fn raw_monitor_notify(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorNotify");
    let monitor = match jvm.thread_state.try_get_monitor(monitor_id) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_MONITOR),
        Some(m) => m,
    };
    monitor.notify(jvm).unwrap();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Raw Monitor Notify All
///
///     jvmtiError
///     RawMonitorNotifyAll(jvmtiEnv* env,
///                 jrawMonitorID monitor)
///
/// Notify all threads waiting on the raw monitor.
///
/// Phase	Callback Safe	Position	Since
/// may be called during any phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	37	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// monitor	jrawMonitorID	The monitor
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NOT_MONITOR_OWNER	Not monitor owner
/// JVMTI_ERROR_INVALID_MONITOR	monitor is not a jrawMonitorID.
pub unsafe extern "C" fn raw_monitor_notify_all(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorNotifyAll");
    let monitor = match jvm.thread_state.try_get_monitor(monitor_id) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_MONITOR),
        Some(m) => m,
    };
    monitor.notify_all(jvm).unwrap();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Destroy Raw Monitor
///
///     jvmtiError
///     DestroyRawMonitor(jvmtiEnv* env,
///                 jrawMonitorID monitor)
///
/// Destroy the raw monitor. If the monitor being destroyed has been entered by this thread, it will be exited before it is destroyed.
/// If the monitor being destroyed has been entered by another thread, an error will be returned and the monitor will not be destroyed.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the OnLoad or the live phase 	This function may be called from the callbacks to the Heap iteration functions, or from the event handlers for the GarbageCollectionStart, GarbageCollectionFinish, and ObjectFree events. 	32	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// monitor	jrawMonitorID	The monitor
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NOT_MONITOR_OWNER	Not monitor owner
/// JVMTI_ERROR_INVALID_MONITOR	monitor is not a jrawMonitorID.
pub unsafe extern "C" fn destroy_raw_monitor(env: *mut jvmtiEnv, monitor: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorNotifyAll");
    let monitor = match jvm.thread_state.try_get_monitor(monitor) {
        None => return jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_MONITOR),
        Some(m) => m,
    };
    todo!()
    /*match monitor.destroy(jvm) {
        Ok(_) => jvmtiError_JVMTI_ERROR_NONE,
        Err(_) => jvmtiError_JVMTI_ERROR_NOT_MONITOR_OWNER
    }*/
}