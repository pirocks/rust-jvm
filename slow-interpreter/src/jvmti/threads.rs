use std::ffi::{c_void, CString};
use std::intrinsics::transmute;
use std::mem::size_of;
use std::sync::Arc;

use jvmti_jni_bindings::*;

use crate::{JavaThread, SuspendedStatus};
use crate::java_values::JavaValue;
use crate::jvmti::{get_interpreter_state, get_state};
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::interface::local_frame::{new_local_ref, new_local_ref_internal};
use crate::rust_jni::native_util::{from_object, to_object};

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
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetTopThreadGroups");
    null_check!(group_count_ptr);
    null_check!(groups_ptr);
    assert!(jvm.vm_live());
    //There is only one top level thread group in this JVM.
    group_count_ptr.write(1);
    let system_j_thread_group = jvm.thread_state.system_thread_group.read().unwrap().clone().unwrap();

    dbg!(system_j_thread_group.threads_non_null().iter().map(|thread| thread.name().to_rust_string()).collect::<Vec<_>>());// todo should include Main thread
    let thread_group_object = system_j_thread_group.object();
    let res = new_local_ref_internal(to_object(thread_group_object.into()), int_state);

    jvm.native_interface_allocations.allocate_and_write_vec(vec![res], group_count_ptr, groups_ptr);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

///Get All Threads
///
///     jvmtiError
///     GetAllThreads(jvmtiEnv* env,
///                 jint* threads_count_ptr,
///                 jthread** threads_ptr)
///
/// Get all live threads. The threads are Java programming language threads; that is, threads that are attached to the VM.
/// A thread is live if java.lang.Thread.isAlive() would return true, that is, the thread has been started and has not yet died.
/// The universe of threads is determined by the context of the JVM TI environment, which typically is all threads attached to the VM.
/// Note that this includes JVM TI agent threads (see RunAgentThread).
///
/// Phase	Callback Safe	Position	Since
/// may only be called during the live phase 	No 	4	1.0
///
/// Capabilities
/// Required Functionality
///
/// Parameters
/// Name 	Type 	Description
/// threads_count_ptr	jint*	On return, points to the number of running threads.
///
/// Agent passes a pointer to a jint. On return, the jint has been set.
/// threads_ptr	jthread**	On return, points to an array of references, one for each running thread.
///
/// Agent passes a pointer to a jthread*. On return, the jthread* points to a newly allocated array of size *threads_count_ptr.
/// The array should be freed with Deallocate. The objects returned by threads_ptr are JNI local references and must be managed.
///
/// Errors
/// This function returns either a universal error or one of the following errors
/// Error 	Description
/// JVMTI_ERROR_NULL_POINTER	threads_count_ptr is NULL.
/// JVMTI_ERROR_NULL_POINTER	threads_ptr is NULL.
pub unsafe extern "C" fn get_all_threads(env: *mut jvmtiEnv, threads_count_ptr: *mut jint, threads_ptr: *mut *mut jthread) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetAllThreads");
    null_check!(threads_count_ptr);
    null_check!(threads_ptr);
    assert!(jvm.vm_live());
    let mut res_ptrs = jvm.thread_state.get_all_threads().values().filter(|thread| {
        thread.thread_object().is_alive(jvm, int_state) != 0
    }).map(|thread| {
        let thread_ptr = to_object(thread.thread_object().object().into());
        new_local_ref_internal(thread_ptr, int_state)
    }).collect::<Vec<jobject>>();
    jvm.native_interface_allocations.allocate_and_write_vec(res_ptrs, threads_count_ptr, threads_ptr);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_thread_info(env: *mut jvmtiEnv, thread: jthread, info_ptr: *mut jvmtiThreadInfo) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadInfo");
    let thread_object = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    (*info_ptr).thread_group = transmute(to_object(jvm.thread_state.system_thread_group.read().unwrap().clone().unwrap().object().into()));//todo get thread groups working at some point
    (*info_ptr).context_class_loader = transmute(to_object(
        jvm
            .classes
            .class_object_pool
            .read().unwrap()
            .get(&RuntimeClass::Int).unwrap()//todo technically this needs a check inited class
            .lookup_field("classLoader")
            .unwrap_object()));//todo deal with this whole loader situation
    (*info_ptr).name = jvm.native_interface_allocations.allocate_cstring(CString::new(thread_object.name().to_rust_string()).unwrap());
    (*info_ptr).is_daemon = thread_object.daemon() as u8;//todo this issue again
    (*info_ptr).priority = thread_object.priority();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn get_thread_state(env: *mut jvmtiEnv, thread: jthread, thread_state_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadState");
    let jthread = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let thread = jthread.get_java_thread(jvm);
    let suspended = *thread.suspended.suspended.lock().unwrap();
    let state = JVMTI_THREAD_STATE_ALIVE | if suspended {
        JVMTI_THREAD_STATE_SUSPENDED
    } else {
        JVMTI_THREAD_STATE_ALIVE//todo this is not always correct
    };
    thread_state_ptr.write(state as i32);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

pub unsafe extern "C" fn suspend_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThreadList");
    // dbg!(jvm.thread_state.alive_threads.read().unwrap().keys());
    // dbg!(jvm.thread_state.main_thread.read().unwrap().as_ref().unwrap().java_tid);
    for i in 0..request_count {
        let thread_object_raw = from_object(transmute(request_list.offset(i as isize).read()));//todo this transmute bs will sone have gone too far
        let jthread = JavaValue::Object(thread_object_raw).cast_thread();
        let java_thread = jthread.get_java_thread(jvm);
        results.offset(i as isize).write(suspend_thread_impl(java_thread));
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

fn suspend_thread_impl(java_thread: Arc<JavaThread>) -> jvmtiError {
    let SuspendedStatus { suspended, suspend_condvar } = &java_thread.suspended;
    let mut suspended_guard = suspended.lock().unwrap();
    if *suspended_guard {
        jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED
    } else {
        *suspended_guard = true;
        jvmtiError_JVMTI_ERROR_NONE
    }
}

pub unsafe extern "C" fn interrupt_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, suspend_thread(env, thread))//todo this is an ugly hack.
}

pub unsafe extern "C" fn suspend_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    //todo dubplication
    //todo this part is not correct: If the calling thread is specified, this function will not return until some other thread calls ResumeThread. If the thread is currently suspended, this function does nothing and returns an error.
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    let thread_object_raw = from_object(thread);
    let jthread = JavaValue::Object(thread_object_raw).cast_thread();
    let java_thread = jthread.get_java_thread(jvm);
    let res = suspend_thread_impl(java_thread);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, res)
}

pub unsafe extern "C" fn resume_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThread");
    let thread_object_raw = from_object(thread);
    let jthread = JavaValue::Object(thread_object_raw).cast_thread();
    let java_thread = jthread.get_java_thread(jvm);
    let res = resume_thread_impl(java_thread);
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, res)
}


pub unsafe extern "C" fn resume_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThreadList");
    for i in 0..request_count {
        let jthreadp = request_list.offset(i as isize).read();
        let jthread = JavaValue::Object(from_object(jthreadp)).cast_thread();
        let java_thread = jthread.get_java_thread(jvm);
        results.offset(i as isize).write(resume_thread_impl(java_thread));
    }
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}


fn resume_thread_impl(java_thread: Arc<JavaThread>) -> jvmtiError {
    let SuspendedStatus { suspended, suspend_condvar } = &java_thread.suspended;
    let mut suspend_guard = suspended.lock().unwrap();
    if !*suspend_guard {
        unimplemented!()
    } else {
        *suspend_guard = false;
        suspend_condvar.notify_one();//notify one and notify all should be the same here
        jvmtiError_JVMTI_ERROR_NONE
    }
}

pub unsafe extern "C" fn get_thread_group_info(env: *mut jvmtiEnv, _group: jthreadGroup, info_ptr: *mut jvmtiThreadGroupInfo) -> jvmtiError {
    let jvm = get_state(env);
    let tracing_guard = jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadGroupInfo");
    //todo thread groups not implemented atm.
    let boxed_string = CString::new("main").unwrap().into_boxed_c_str();
    let info_pointer_writer = info_ptr.as_mut().unwrap();
    info_pointer_writer.name = Box::leak(boxed_string).as_ptr() as *mut i8;
    info_pointer_writer.is_daemon = false as jboolean;
    info_pointer_writer.max_priority = 0;
    info_pointer_writer.parent = std::ptr::null_mut();
    jvm.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}