use std::ffi::CStr;
use std::intrinsics::transmute;
use std::os::raw::c_char;

use jvmti_jni_bindings::{jlong, jrawMonitorID, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::jvmti::get_state;

pub unsafe extern "C" fn create_raw_monitor(env: *mut jvmtiEnv, name: *const c_char, monitor_ptr: *mut jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "CreateRawMonitor");
    //todo handle name
    let monitor_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let res_monitor = jvm.new_monitor(monitor_name);
    monitor_ptr.write(transmute(res_monitor.monitor_i));//todo check that this is acceptable
    jvm.tracing.trace_jdwp_function_exit(jvm, "CreateRawMonitor");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_enter(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorEnter");
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.lock(jvm);
    jvm.tracing.trace_jdwp_function_exit(jvm, "RawMonitorEnter");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_exit(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorExit");
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.unlock(jvm);
    jvm.tracing.trace_jdwp_function_exit(jvm, "RawMonitorExit");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_wait(env: *mut jvmtiEnv, monitor_id: jrawMonitorID, millis: jlong) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorWait");
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.wait(millis, jvm);
    jvm.tracing.trace_jdwp_function_exit(jvm, "RawMonitorWait");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_notify_all(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorNotifyAll");
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.notify_all(jvm);
    jvm.tracing.trace_jdwp_function_exit(jvm, "RawMonitorNotifyAll");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn raw_monitor_notify(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "RawMonitorNotify");
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.notify(jvm);
    jvm.tracing.trace_jdwp_function_exit(jvm, "RawMonitorNotify");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn destroy_raw_monitor(_env: *mut jvmtiEnv, _monitor: jrawMonitorID) -> jvmtiError {
    //todo for now no-op
    jvmtiError_JVMTI_ERROR_NONE
}