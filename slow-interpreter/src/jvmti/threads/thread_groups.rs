use std::ffi::CString;

use jvmti_jni_bindings::{jint, jthreadGroup, jvmtiEnv, jvmtiError, jvmtiError_JVMTI_ERROR_INVALID_THREAD_GROUP, jvmtiError_JVMTI_ERROR_NONE, jvmtiThreadGroupInfo};

use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::rust_jni::interface::local_frame::new_local_ref_public;

///Get Thread Group Info
///
///     typedef struct {
///         jthreadGroup parent;
///         char* name;
///         jint max_priority;
///         jboolean is_daemon;
///     } jvmtiThreadGroupInfo;
///
///     jvmtiError
///     GetThreadGroupInfo(jvmtiEnv* env,
///                 jthreadGroup group,
///                 jvmtiThreadGroupInfo* info_ptr)
///
/// Get information about the thread group. The fields of the jvmtiThreadGroupInfo structure are filled in with details of the specified thread group.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	14	1.0
///
/// Capabilities
/// Required Functionality
///
/// jvmtiThreadGroupInfo - Thread group information structure
/// Field 	Type 	Description
/// parent	jthreadGroup	The parent thread group.
/// name	char*	The thread group's name, encoded as a modified UTF-8 string.
/// max_priority	jint	The maximum priority for this thread group.
/// is_daemon	jboolean	Is this a daemon thread group?
///
/// Parameters
/// Name 	Type 	Description
/// group	jthreadGroup	The thread group to query.
/// info_ptr	jvmtiThreadGroupInfo*	On return, filled with information describing the specified thread group.
///
/// Agent passes a pointer to a jvmtiThreadGroupInfo. On return, the jvmtiThreadGroupInfo has been set.
/// The object returned in the field parent of jvmtiThreadGroupInfo is a JNI local reference and must be managed.
/// The pointer returned in the field name of jvmtiThreadGroupInfo is a newly allocated array.
/// The array should be freed with Deallocate.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_INVALID_THREAD_GROUP	group is not a thread group object.
/// JVMTI_ERROR_NULL_POINTER	info_ptr is NULL.
pub unsafe extern "C" fn get_thread_group_info(env: *mut jvmtiEnv, group: jthreadGroup, info_ptr: *mut jvmtiThreadGroupInfo) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetThreadGroupInfo");
    assert!(jvm.vm_live());
    let thread_group = match JavaValue::Object(todo!() /*from_jclass(jvm,group)*/).try_cast_thread_group() {
        None => return jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_INVALID_THREAD_GROUP),
        Some(thread_group) => thread_group,
    };
    null_check!(info_ptr);

    let name = jvm.native.native_interface_allocations.allocate_cstring(CString::new(thread_group.name(jvm).to_rust_string(jvm)).unwrap());
    let info_pointer_writer = info_ptr.as_mut().unwrap();
    info_pointer_writer.name = name;
    info_pointer_writer.is_daemon = thread_group.daemon(jvm);
    info_pointer_writer.max_priority = thread_group.max_priority(jvm);
    info_pointer_writer.parent = new_local_ref_public(thread_group.parent(jvm).map(|x| x.object().to_gc_managed()), int_state);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get Top Thread Groups
///
///     jvmtiError
///     GetTopThreadGroups(jvmtiEnv* env,
///                 jint* group_count_ptr,
///                 jthreadGroup** groups_ptr)
///
/// Return all top-level (parentless) thread groups in the VM.
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	13	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// group_count_ptr	jint*	On return, points to the number of top-level thread groups.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// groups_ptr	jthreadGroup**	On return, refers to a pointer to the top-level thread group array.
///
/// Agent passes a pointer to a jthreadGroup*. On return, the jthreadGroup* points to a newly allocated array of size *group_count_ptr. The array should be freed with Deallocate. The objects returned by groups_ptr are JNI local references and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NULL_POINTER	group_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	groups_ptr is NULL.
pub unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "GetTopThreadGroups");
    null_check!(group_count_ptr);
    null_check!(groups_ptr);
    assert!(jvm.vm_live());
    //There is only one top level thread group in this JVM.
    group_count_ptr.write(1);
    let system_j_thread_group = jvm.thread_state.get_system_thread_group();
    let thread_group_object = system_j_thread_group.object().to_gc_managed();
    let res = new_local_ref_public(thread_group_object.into(), int_state);

    jvm.native.native_interface_allocations.allocate_and_write_vec(vec![res], group_count_ptr, groups_ptr);
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}
