use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::null_mut;

use classfile_view::view::HasAccessFlags;
use jvmti_jni_bindings::{_jvmtiLineNumberEntry, _jvmtiLocalVariableEntry, jlocation, jmethodID, jthread, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION, jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT, jvmtiError_JVMTI_ERROR_INVALID_METHODID, jvmtiError_JVMTI_ERROR_NATIVE_METHOD, jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES, jvmtiError_JVMTI_ERROR_NONE, jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE, jvmtiLineNumberEntry, jvmtiLocalVariableEntry};
use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_state::InterpreterState;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::jvmti::from_object;
use crate::method_table::from_jmethod_id;
use crate::stack_entry::StackEntry;

/// Get Frame Count
///
///     jvmtiError
///     GetFrameCount(jvmtiEnv* env,
///                 jthread thread,
///                 jint* count_ptr)
///
/// Get the number of frames currently in the specified thread's call stack.
///
/// If this function is called for a thread actively executing bytecodes (for example, not the current thread and not suspended), the information returned is transient.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	16	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread to query. If thread is NULL, the current thread is used.
/// count_ptr	jint*	On return, points to the number of frames in the call stack.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	DescriptionJS
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
/// JVMTI_ERROR_NULL_POINTER	count_ptr is NULL.
///
pub unsafe extern "C" fn get_frame_count(env: *mut jvmtiEnv, thread: jthread, count_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetFrameCount");
    assert!(jvm.vm_live());
    null_check!(count_ptr);

    let jthread = get_thread_or_error!(jvm,thread);
    let java_thread = jthread.get_java_thread(jvm);
    if !java_thread.is_alive() {
        return jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE;
    }
    // assert!(*java_thread.suspended.suspended.lock().unwrap());//todo technically need to support non-suspended threads as well

    let frame_count: i32 = match java_thread.interpreter_state.read().unwrap().deref() {
        /*InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.len(),*/
        InterpreterState::Jit { .. } => todo!()
    };
    count_ptr.write(frame_count as i32);

    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Frame Location
///
///     jvmtiError
///     GetFrameLocation(jvmtiEnv* env,
///                 jthread thread,
///                 jint depth,
///                 jmethodID* method_ptr,
///                 jlocation* location_ptr)
///
/// For a Java programming language frame, return the location of the instruction currently executing.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	19	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// thread	jthread	The thread of the frame to query. If thread is NULL, the current thread is used.
/// depth	jint	The depth of the frame to query.
/// method_ptr	jmethodID*	On return, points to the method for the current location.
///
/// Agent passes a pointer to a jmethodID. On return, the jmethodID has been set.
/// location_ptr	jlocation*	On return, points to the index of the currently executing instruction.
/// Is set to -1 if the frame is executing a native method.
///
/// Agent passes a pointer to a jlocation. On return, the jlocation has been set.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
/// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
/// JVMTI_ERROR_ILLEGAL_ARGUMENT	depth is less than zero.
/// JVMTI_ERROR_NO_MORE_FRAMES	There are no stack frames at the specified depth.
/// JVMTI_ERROR_NULL_POINTER	method_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	location_ptr is NULL.
pub unsafe extern "C" fn get_frame_location(env: *mut jvmtiEnv, thread: jthread, depth: jint, method_ptr: *mut jmethodID, location_ptr: *mut jlocation) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetFrameLocation");
    assert!(jvm.vm_live());
    let jthread = get_thread_or_error!(jvm,thread);
    null_check!(method_ptr);
    null_check!(location_ptr);
    if depth < 0 {
        return jvmtiError_JVMTI_ERROR_ILLEGAL_ARGUMENT;
    }
    let thread = jthread.get_java_thread(jvm);
    if !thread.is_alive() {
        return jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE;
    }
    let read_guard = thread.interpreter_state.read().unwrap();
    let call_stack_guard: Vec<StackEntry> = match read_guard.deref() {
        /*InterpreterState::LegacyInterpreter { call_stack, .. } => { call_stack }*/
        InterpreterState::Jit { .. } => todo!()
    };
    let stack_entry = match call_stack_guard.get(call_stack_guard.len() - 1 - depth as usize) {
        None => return jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES,
        Some(stack_entry) => stack_entry,
    };
    let meth_id = match stack_entry.try_method_i() {
        None => {
            // so in the event of a completely opaque frame, just say it is Thread.start.
            // this is not perfect, ideally we would return an error:
            // return jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES
            let int_state = get_interpreter_state(env);
            let thread_class = assert_inited_or_initing_class(jvm, CClassName::thread().into());
            let thread_class_view = thread_class.view();
            let possible_starts = thread_class_view.lookup_method_name(MethodName::method_start());
            let thread_start_view = possible_starts.get(0).unwrap();
            jvm.method_table.write().unwrap().get_method_id(thread_class.clone(), thread_start_view.method_i() as u16)
        }
        Some(method_i) => {
            jvm.method_table.write().unwrap().get_method_id(stack_entry.class_pointer().clone(), method_i)
        }
    };

    method_ptr.write(transmute(meth_id));
    location_ptr.write(stack_entry.try_pc().map(|x| x as i64).unwrap_or(-1));
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Local Variable Table
///
///     typedef struct {
///         jlocation start_location;
///         jint length;
///         char* name;
///         char* signature;
///         char* generic_signature;
///         jint slot;
///     } jvmtiLocalVariableEntry;
///
///     jvmtiError
///     GetLocalVariableTable(jvmtiEnv* env,
///                 jmethodID method,
///                 jint* entry_count_ptr,
///                 jvmtiLocalVariableEntry** table_ptr)
///
/// Return local variable information.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	72	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines.
/// The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_access_local_variables	Can set and get local variables
///
/// jvmtiLocalVariableEntry - Local variable table entry
/// Field 	Type 	Description
/// start_location	jlocation	The code array index where the local variable is first valid (that is, where it must have a value).
/// length	jint	The length of the valid section for this local variable. The last code array index where the local variable is valid is start_location + length.
/// name	char*	The local variable name, encoded as a modified UTF-8 string.
/// signature	char*	The local variable's type signature, encoded as a modified UTF-8 string. The signature format is the same as that defined in The Java™ Virtual Machine Specification, Chapter 4.3.2.
/// generic_signature	char*	The local variable's generic signature, encoded as a modified UTF-8 string. The value of this field will be NULL for any local variable which does not have a generic type.
/// slot	jint	The local variable's slot. See Local Variables.
///
/// Parameters
/// Name 	Type 	Description
/// method	jmethodID	The method to query.
/// entry_count_ptr	jint*	On return, points to the number of entries in the table
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// table_ptr	jvmtiLocalVariableEntry**	On return, points to an array of local variable table entries.
///
/// Agent passes a pointer to a jvmtiLocalVariableEntry*. On return, the jvmtiLocalVariableEntry* points to a newly allocated array of size *entry_count_ptr. The array should be freed with Deallocate. The pointers returned in the field name of jvmtiLocalVariableEntry are newly allocated arrays. The arrays should be freed with Deallocate. The pointers returned in the field signature of jvmtiLocalVariableEntry are newly allocated arrays. The arrays should be freed with Deallocate. The pointers returned in the field generic_signature of jvmtiLocalVariableEntry are newly allocated arrays. The arrays should be freed with Deallocate.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_access_local_variables. Use AddCapabilities.
/// JVMTI_ERROR_ABSENT_INFORMATION	Class information does not include local variable information.
/// JVMTI_ERROR_INVALID_METHODID	method is not a jmethodID.
/// JVMTI_ERROR_NATIVE_METHOD	method is a native method.
/// JVMTI_ERROR_NULL_POINTER	entry_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	table_ptr is NULL.
pub unsafe extern "C" fn get_local_variable_table(
    env: *mut jvmtiEnv,
    method: jmethodID,
    entry_count_ptr: *mut jint,
    table_ptr: *mut *mut jvmtiLocalVariableEntry,
) -> jvmtiError {
    //todo check capabilities
    let jvm = get_state(env);
    assert!(jvm.vm_live());
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetLocalVariableTable");
    null_check!(table_ptr);
    null_check!(entry_count_ptr);
    let method_id = from_jmethod_id(method);
    let option = jvm.method_table.read().unwrap().try_lookup(method_id);
    assert!(option.is_some());
    let (class, method_i) = match option {
        None => {
            assert!(false);
            return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_METHODID);
        }
        Some(pair) => pair
    };
    let class_view = class.view();
    let method_view = class_view.method_view_i(method_i);
    let num_locals = method_view.code_attribute().unwrap().max_locals as usize;
    if method_view.is_native() {
        return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NATIVE_METHOD);
    }
    let local_vars = match method_view.local_variable_attribute() {
        None => {
            //todo make up a local var table
            // let code_attr = method_view.code_attribute().unwrap();
            // let max_locals = code_attr.max_locals;
            // (0..max_locals).map(|i|{
            //     let name = format!("var{}",i);
            //     let allocated_name = jvm.native_interface_allocations.allocate_string(name);
            //     let signature = local_variable_view.desc_str();
            //     let allocated_signature = jvm.native_interface_allocations.allocate_string(signature);
            //     let slot = i as i32;
            //     _jvmtiLocalVariableEntry{
            //         start_location: 0,
            //         length: code_attr.code.len() as i32,
            //         name: allocated_name,
            //         signature: null_mut(),
            //         generic_signature: null_mut(),
            //         slot
            //     }
            // })
            return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION);
        }
        Some(lva) => lva,
    };
    entry_count_ptr.write(num_locals as i32);
    let res = local_vars.iter().map(|local_variable_view| {
        let name = local_variable_view.name();
        let allocated_name = jvm.native_interface_allocations.allocate_string(name);
        let signature = local_variable_view.desc_str();
        let allocated_signature = jvm.native_interface_allocations.allocate_string(signature);
        let slot = local_variable_view.local_var_slot() as i32;
        _jvmtiLocalVariableEntry {
            start_location: local_variable_view.variable_start_pc() as i64,
            length: local_variable_view.variable_length() as i32,
            name: allocated_name,
            signature: allocated_signature,
            generic_signature: null_mut(),//todo impl
            slot,
        }
    }).collect::<Vec<_>>();
    jvm.native_interface_allocations.allocate_and_write_vec(res, entry_count_ptr, table_ptr);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

/// Get Line Number Table
///
/// typedef struct {
///     jlocation start_location;
///     jint line_number;
/// } jvmtiLineNumberEntry;
///
/// jvmtiError
/// GetLineNumberTable(jvmtiEnv* env,
/// jmethodID method,
/// jint* entry_count_ptr,
/// jvmtiLineNumberEntry** table_ptr)
///
/// For the method indicated by method, return a table of source line number entries. The size of the table is returned via entry_count_ptr and the table itself is returned via table_ptr.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the start or the live phase 	No 	70	1.0
///
/// Capabilities
/// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
/// Capability 	Effect
/// can_get_line_numbers	Can get the line number table of a method
///
/// jvmtiLineNumberEntry - Line number table entry
/// Field 	Type 	Description
/// start_location	jlocation	the jlocation where the line begins
/// line_number	jint	the line number
///
/// Parameters
/// Name 	Type 	Description
/// method	jmethodID	The method to query.
/// entry_count_ptr	jint*	On return, points to the number of entries in the table
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// table_ptr	jvmtiLineNumberEntry**	On return, points to the line number table pointer.
///
/// Agent passes a pointer to a jvmtiLineNumberEntry*. On return, the jvmtiLineNumberEntry* points to a newly allocated array of size *entry_count_ptr. The array should be freed with Deallocate.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_get_line_numbers. Use AddCapabilities.
/// JVMTI_ERROR_ABSENT_INFORMATION	Class information does not include line numbers.
/// JVMTI_ERROR_INVALID_METHODID	method is not a jmethodID.
/// JVMTI_ERROR_NATIVE_METHOD	method is a native method.
/// JVMTI_ERROR_NULL_POINTER	entry_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	table_ptr is NULL.
pub unsafe extern "C" fn get_line_number_table(env: *mut jvmtiEnv, method: jmethodID, entry_count_ptr: *mut jint, table_ptr: *mut *mut jvmtiLineNumberEntry) -> jvmtiError {
    let jvm = get_state(env);
    let method_id = from_jmethod_id(method);
    //todo capabilities
    assert!(jvm.vm_live());
    null_check!(table_ptr);
    null_check!(entry_count_ptr);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetLineNumberTable");
    let (class, method_i) = match jvm.method_table.read().unwrap().try_lookup(method_id) {
        None => {
            return jvmtiError_JVMTI_ERROR_INVALID_METHODID;
        }
        Some(method) => method,
    };//todo
    let class_view = class.view();
    let method_view = class_view.method_view_i(method_i);
    if method_view.is_native() {
        return jvmtiError_JVMTI_ERROR_NATIVE_METHOD;
    }
    let table = &match method_view.line_number_table() {
        None => {
            return jvmtiError_JVMTI_ERROR_ABSENT_INFORMATION;
        }
        Some(table) => table,
    }.line_number_table;
    entry_count_ptr.write(table.len() as i32);
    let res_table = jvm.native_interface_allocations.allocate_malloc(size_of::<_jvmtiLineNumberEntry>() * table.len()) as *mut _jvmtiLineNumberEntry;
    for (i, entry) in table.iter().enumerate() {
        let start = entry.start_pc;
        let line_number = entry.line_number;
        res_table.add(i).write(_jvmtiLineNumberEntry {
            start_location: start as i64,
            line_number: line_number as i32,
        })
    }
    table_ptr.write(res_table);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
