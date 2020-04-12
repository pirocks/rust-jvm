use jvmti_bindings::{jvmtiEnv, jrawMonitorID, jvmtiError};
use std::os::raw::c_char;

pub unsafe extern "C" fn create_raw_monitor(env: *mut jvmtiEnv, name: *const c_char, monitor_ptr: *mut jrawMonitorID) -> jvmtiError{
    unimplemented!()
}
