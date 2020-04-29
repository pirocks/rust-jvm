use jvmti_bindings::{jvmtiError_JVMTI_ERROR_NONE, jvmtiEnv, jthread, jvmtiError};
use jni_bindings::jint;
use crate::jvmti::get_state;
use crate::rust_jni::native_util::from_object;
use std::intrinsics::transmute;
use crate::java_values::JavaValue;

pub unsafe extern "C" fn get_frame_count(env: *mut jvmtiEnv, thread: jthread, count_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetFrameCount");

    let jthread = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let tid = jthread.tid();
    let java_thread = jvm.thread_state.alive_threads.read().unwrap().get(&tid).unwrap().clone();
    let frame_count = java_thread.call_stack.borrow().len();
    count_ptr.write(frame_count as i32);

    jvm.tracing.trace_jdwp_function_enter(jvm,"GetFrameCount");
    jvmtiError_JVMTI_ERROR_NONE
}
