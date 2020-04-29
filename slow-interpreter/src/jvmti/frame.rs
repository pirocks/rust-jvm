use jvmti_jni_bindings::{jvmtiError_JVMTI_ERROR_NONE, jvmtiEnv, jthread, jvmtiError, jmethodID, jlocation};
use jvmti_jni_bindings::jint;
use crate::jvmti::get_state;
use crate::rust_jni::native_util::from_object;
use std::intrinsics::transmute;
use crate::java_values::JavaValue;
use std::ops::Deref;
use crate::rust_jni::MethodId;

pub unsafe extern "C" fn get_frame_count(env: *mut jvmtiEnv, thread: jthread, count_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetFrameCount");

    let jthread = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let tid = jthread.tid();
    let java_thread = jvm.thread_state.alive_threads.read().unwrap().get(&tid).unwrap().clone();
    let frame_count = java_thread.call_stack.borrow().len();
    count_ptr.write(frame_count as i32);

    jvm.tracing.trace_jdwp_function_enter(jvm, "GetFrameCount");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_frame_location(env: *mut jvmtiEnv, thread: jthread, depth: jint, method_ptr: *mut jmethodID, location_ptr: *mut jlocation) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetFrameLocation");
    let tid = JavaValue::Object(from_object(transmute(thread))).cast_thread().tid();
    let thread = jvm.thread_state.alive_threads.read().unwrap().get(&tid).unwrap().clone();
    let call_stack_guard =  thread.call_stack.borrow();
    let stack_entry = call_stack_guard[depth as usize].deref();
    let meth_id = box MethodId {
        class: stack_entry.class_pointer.clone(),
        method_i: stack_entry.method_i as usize
    };
    method_ptr.write(Box::leak(meth_id) as *mut MethodId as jmethodID);//todo leak
    location_ptr.write(*stack_entry.pc.borrow() as i64);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetFrameLocation");
    jvmtiError_JVMTI_ERROR_NONE
}
