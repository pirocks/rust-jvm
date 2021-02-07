use std::collections::HashSet;
use std::iter::FromIterator;

use jvmti_jni_bindings::{jlocation, jmethodID, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};

use crate::jvmti::get_state;
use crate::method_table::from_jmethod_id;

pub unsafe extern "C" fn set_breakpoint(env: *mut jvmtiEnv, method: jmethodID, location: jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SetBreakpoint");
    let method_id = from_jmethod_id(method);
    // dbg!(&method_id);
    let lookup_res = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();//todo handle error
    let mv = lookup_res.0.view().method_view_i(lookup_res.1 as usize);
    dbg!(mv.name());
    dbg!(mv.classview().name());
    let mut breakpoint_guard = jvm.jvmti_state.as_ref().unwrap().break_points.write().unwrap();
    match breakpoint_guard.get_mut(&method_id) {
        None => {
            breakpoint_guard.insert(method_id, HashSet::from_iter(vec![location as isize].iter().cloned()));
        }
        Some(breakpoints) => {
            breakpoints.insert(location as isize);//todo should I cast here?
        }
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


pub unsafe extern "C" fn clear_breakpoint(env: *mut jvmtiEnv, method: jmethodID, location: jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracig_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ClearBreakpoint");
    let method_id = from_jmethod_id(method);
    jvm.jvmti_state.as_ref().unwrap().break_points.write().unwrap().get_mut(&method_id).unwrap().remove(&(location as isize));
    jvm.tracing.trace_jdwp_function_exit(tracig_guard, jvmtiError_JVMTI_ERROR_NONE)
}