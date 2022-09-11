use std::collections::HashSet;
use std::iter::FromIterator;

use jvmti_jni_bindings::{jlocation, jmethodID, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
use method_table::from_jmethod_id;
use rust_jvm_common::ByteCodeOffset;

use crate::rust_jni::jvmti_interface::get_state;

pub unsafe extern "C" fn set_breakpoint(env: *mut jvmtiEnv, method: jmethodID, location: jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "SetBreakpoint");
    let method_id = from_jmethod_id(method);
    let mut breakpoint_guard = jvm.jvmti_state().unwrap().break_points.write().unwrap();
    match breakpoint_guard.get_mut(&method_id) {
        None => {
            breakpoint_guard.insert(method_id, HashSet::from_iter(vec![ByteCodeOffset(location as u16)].iter().cloned()));
        }
        Some(breakpoints) => {
            breakpoints.insert(ByteCodeOffset(location as u16)); //todo should I cast here?
        }
    }
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn clear_breakpoint(env: *mut jvmtiEnv, method: jmethodID, location: jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracig_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "ClearBreakpoint");
    let method_id = from_jmethod_id(method);
    jvm.jvmti_state().unwrap().break_points.write().unwrap().get_mut(&method_id).unwrap().remove(&ByteCodeOffset(location as u16));
    jvm.config.tracing.trace_jdwp_function_exit(tracig_guard, jvmtiError_JVMTI_ERROR_NONE)
}
