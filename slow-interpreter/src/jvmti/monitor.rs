use jvmti_bindings::{jvmtiEnv, jrawMonitorID, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
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
    let monitor  = &jvm.monitors.read().unwrap()[monitor as usize];
    monitor.lock();
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn raw_monitor_exit(env: *mut jvmtiEnv, monitor: jrawMonitorID) -> jvmtiError{
    let jvm = get_state(env);
    let monitor  = &jvm.monitors.read().unwrap()[monitor as usize];
    monitor.unlock();
    jvmtiError_JVMTI_ERROR_NONE
}