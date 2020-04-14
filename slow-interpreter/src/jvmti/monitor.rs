use jvmti_bindings::{jvmtiEnv, jrawMonitorID, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jlong};
use std::os::raw::c_char;
use crate::jvmti::get_state;
use std::intrinsics::transmute;

pub unsafe extern "C" fn create_raw_monitor(env: *mut jvmtiEnv, name: *const c_char, monitor_ptr: *mut jrawMonitorID) -> jvmtiError{
    let jvm = get_state(env);
    //todo handle name
    let res_monitor = jvm.new_monitor();
    monitor_ptr.write(transmute(res_monitor.monitor_i));//todo check that this is acceptable
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_enter(env: *mut jvmtiEnv, monitor: jrawMonitorID) -> jvmtiError{
    let jvm = get_state(env);
    let monitors_guard = jvm.monitors.read().unwrap();
    let monitor  = monitors_guard[monitor as usize].clone();
    std::mem::drop(monitors_guard);
    monitor.lock();
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_exit(env: *mut jvmtiEnv, monitor: jrawMonitorID) -> jvmtiError{
    let jvm = get_state(env);
    let monitor  = jvm.monitors.read().unwrap()[monitor as usize].clone();
    monitor.unlock();
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_wait(env: *mut jvmtiEnv, monitor: jrawMonitorID, millis: jlong) -> jvmtiError{
    let jvm = get_state(env);
    let monitors_read_guard = jvm.monitors.read().unwrap();
    let monitor  = monitors_read_guard[monitor as usize].clone();
    std::mem::drop(monitors_read_guard);
    assert_eq!(millis, -1);
    monitor.wait();
    jvmtiError_JVMTI_ERROR_NONE
}