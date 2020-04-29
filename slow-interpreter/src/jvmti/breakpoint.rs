use jvmti_jni_bindings::{jvmtiError_JVMTI_ERROR_NONE, jvmtiEnv, jlocation, jvmtiError, jmethodID};
use crate::jvmti::get_state;
use crate::rust_jni::MethodId;
use std::collections::HashSet;
use std::iter::FromIterator;

pub unsafe extern "C" fn set_breakpoint(env: *mut jvmtiEnv, method: jmethodID, location: jlocation) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SetBreakpoint");
    let method_id = (method as *mut MethodId).as_ref().unwrap();
    dbg!(&method_id);
    let mut breakpoint_guard = jvm.jvmti_state.break_points.write().unwrap();
    match breakpoint_guard.get_mut(method_id) {
        None => {
            breakpoint_guard.insert(method_id.clone(), HashSet::from_iter(vec![location as isize].iter().cloned()));
        }
        Some(breakpoints) => {
            breakpoints.insert(location as isize);//todo should I cast here?
        }
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "SetBreakpoint");
    jvmtiError_JVMTI_ERROR_NONE
}
