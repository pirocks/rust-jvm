use jvmti_bindings::{jvmtiEnv, jrawMonitorID, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jlong};
use std::os::raw::c_char;
use crate::jvmti::get_state;
use std::intrinsics::transmute;
use std::ffi::CStr;

pub unsafe extern "C" fn create_raw_monitor(env: *mut jvmtiEnv, name: *const c_char, monitor_ptr: *mut jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    //todo handle name
    let monitor_name = CStr::from_ptr(name).to_str().unwrap().to_string();
    let res_monitor = jvm.new_monitor(monitor_name);
    monitor_ptr.write(transmute(res_monitor.monitor_i));//todo check that this is acceptable
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_enter(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.lock(jvm);
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_exit(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.unlock(jvm);
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_wait(env: *mut jvmtiEnv, monitor_id: jrawMonitorID, millis: jlong) -> jvmtiError {
    let jvm = get_state(env);
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.wait(millis,jvm);
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_notify_all(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError {
    let jvm = get_state(env);
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.notify_all(jvm);
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn raw_monitor_notify(env: *mut jvmtiEnv, monitor_id: jrawMonitorID) -> jvmtiError{
    let jvm = get_state(env);
    let monitor = jvm.thread_state.get_monitor(monitor_id);
    monitor.notify(jvm);
    jvmtiError_JVMTI_ERROR_NONE
}