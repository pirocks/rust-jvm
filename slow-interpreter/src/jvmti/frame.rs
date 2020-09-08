use std::ffi::CString;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;

use jvmti_jni_bindings::{_jvmtiLineNumberEntry, _jvmtiLocalVariableEntry, jlocation, jmethodID, jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION, jvmtiError_JVMTI_ERROR_NONE, jvmtiLineNumberEntry, jvmtiLocalVariableEntry};
use jvmti_jni_bindings::jint;
use rust_jvm_common::classnames::ClassName;

use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::method_table::MethodId;
use crate::rust_jni::native_util::from_object;

pub unsafe extern "C" fn get_frame_count(env: *mut jvmtiEnv, thread: jthread, count_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetFrameCount");

    let jthread = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let java_thread = jthread.get_java_thread(jvm);
    assert!(*java_thread.suspended.suspended.lock().unwrap());
    let frame_count = java_thread.interpreter_state.read().unwrap().call_stack.len();
    dbg!(java_thread.thread_object().name().to_rust_string());
    count_ptr.write(frame_count as i32);

    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


pub unsafe extern "C" fn get_frame_location(env: *mut jvmtiEnv, thread: jthread, depth: jint, method_ptr: *mut jmethodID, location_ptr: *mut jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetFrameLocation");
    let jthread = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let thread = jthread.get_java_thread(jvm);
    let call_stack_guard = &thread.interpreter_state.read().unwrap().call_stack;
    let stack_entry = &call_stack_guard[call_stack_guard.len() - 1 - depth as usize];
    let meth_id = match stack_entry.method_i {
        None => {
            let int_state = get_interpreter_state(env);
            let thread_class = check_inited_class(jvm, int_state, &ClassName::thread().into(), int_state.current_loader(jvm));
            let possible_starts = thread_class.view().lookup_method_name(&"start".to_string());
            let thread_start_view = possible_starts.iter().next().unwrap();
            jvm.method_table.write().unwrap().get_method_id(thread_class.clone(), thread_start_view.method_i() as u16)
        },
        Some(method_i) => {
            jvm.method_table.write().unwrap().get_method_id(stack_entry.class_pointer.clone(), method_i)
        },
    };

    method_ptr.write(transmute(meth_id));
    location_ptr.write(stack_entry.pc as i64);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


pub unsafe extern "C" fn get_local_variable_table(
    env: *mut jvmtiEnv,
    method: jmethodID,
    entry_count_ptr: *mut jint,
    table_ptr: *mut *mut jvmtiLocalVariableEntry,
) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetLocalVariableTable");
    let method_id: MethodId = transmute(method);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let method_view = class.view().method_view_i(method_i as usize);
    let num_locals = method_view.code_attribute().unwrap().max_locals as usize;
    let local_vars = match method_view.local_variable_attribute() {
        None => {
            dbg!(method_view.name());
            dbg!(class.view().name());

            return jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION;
        }
        Some(lva) => lva,
    };
    entry_count_ptr.write(num_locals as i32);
    let res_table = jvm.native_interface_allocations.allocate_malloc(size_of::<_jvmtiLocalVariableEntry>() * num_locals) as *mut _jvmtiLocalVariableEntry;
    assert_eq!(num_locals, local_vars.len());
    for (i, local_variable_view) in local_vars.iter().enumerate() {
        let name = local_variable_view.name();
        let allocated_name = jvm.native_interface_allocations.allocate_cstring(CString::new(name).unwrap());
        let signature = local_variable_view.desc_str();
        let allocated_signature = jvm.native_interface_allocations.allocate_cstring(CString::new(signature).unwrap());
        let entry = _jvmtiLocalVariableEntry {
            start_location: local_variable_view.variable_start_pc() as i64,
            length: local_variable_view.variable_length() as i32,
            name: allocated_name,
            signature: allocated_signature,
            generic_signature: null_mut(),//todo impl
            slot: local_variable_view.local_var_slot() as i32,
        };
        res_table.offset(i as isize).write(entry);
    }
    table_ptr.write(res_table);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


pub unsafe extern "C" fn get_line_number_table(env: *mut jvmtiEnv, method: jmethodID, entry_count_ptr: *mut jint, table_ptr: *mut *mut jvmtiLineNumberEntry) -> jvmtiError {
    let jvm = get_state(env);
    let method_id: MethodId = transmute(method);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetLineNumberTable");
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let method_view = class.view().method_view_i(method_i as usize);
    let table = &method_view.line_number_table().unwrap().line_number_table;
    entry_count_ptr.write(table.len() as i32);
    let res_table = jvm.native_interface_allocations.allocate_malloc(size_of::<_jvmtiLineNumberEntry>() * table.len()) as *mut _jvmtiLineNumberEntry;
    for (i, entry) in table.iter().enumerate() {
        let start = entry.start_pc;
        let line_number = entry.line_number;
        res_table.offset(i as isize).write(_jvmtiLineNumberEntry {
            start_location: start as i64,
            line_number: line_number as i32,
        })
    }
    table_ptr.write(res_table);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}