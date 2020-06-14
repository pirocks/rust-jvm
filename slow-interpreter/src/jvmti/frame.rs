use std::mem::{size_of, transmute};
use std::ops::Deref;

use jvmti_jni_bindings::{_jvmtiLocalVariableEntry, jlocation, jmethodID, jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jvmtiLocalVariableEntry};
use jvmti_jni_bindings::jint;

use crate::java_values::JavaValue;
use crate::jvmti::get_state;
use crate::method_table::MethodId;
use crate::rust_jni::native_util::from_object;
use std::ffi::CString;
use std::ptr::null_mut;

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
    let call_stack_guard = thread.call_stack.borrow();
    let stack_entry = call_stack_guard[depth as usize].deref();
    let meth_id = jvm.method_table.write().unwrap().get_method_id(stack_entry.class_pointer.clone(), stack_entry.method_i);
    method_ptr.write(transmute(meth_id));
    location_ptr.write(*stack_entry.pc.borrow() as i64);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetFrameLocation");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_local_variable_table(
    env: *mut jvmtiEnv,
    method: jmethodID,
    entry_count_ptr: *mut jint,
    table_ptr: *mut *mut jvmtiLocalVariableEntry,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetLocalVariableTable");
    let method_id: MethodId = transmute(method);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let method_view = class.view().method_view_i(method_i as usize);
    let num_locals = method_view.code_attribute().unwrap().max_locals as usize;
    entry_count_ptr.write(num_locals as i32);
    let res_table = jvm.native_interface_allocations.allocate_malloc(size_of::<_jvmtiLocalVariableEntry>()* num_locals) as *mut _jvmtiLocalVariableEntry;
    let local_vars = method_view.local_variable_attribute().unwrap();
    assert_eq!(num_locals, local_vars.len());
    for (i,local_variable_view) in local_vars.iter().enumerate() {
        let name = local_variable_view.name();
        let allocated_name = jvm.native_interface_allocations.allocate_cstring(CString::new(name).unwrap());
        let signature = local_variable_view.desc_str();
        let allocated_signature = jvm.native_interface_allocations.allocate_cstring(CString::new(signature).unwrap());
        let entry = _jvmtiLocalVariableEntry {
            start_location: local_variable_view.variable_start_pc() as i64,
            length: local_variable_view.variable_length() as i32,
            name: allocated_name,
            signature: allocated_signature,
            generic_signature :null_mut(),//todo impl
            slot: local_variable_view.local_var_slot() as i32
        };
        res_table.offset(i as isize).write(entry);
    }
    table_ptr.write(res_table);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetLocalVariableTable");
    jvmtiError_JVMTI_ERROR_NONE
}