use std::ffi::{c_void, CString};
use std::intrinsics::transmute;
use std::sync::Arc;

use jvmti_jni_bindings::*;

use crate::{JavaThread, SuspendedStatus};
use crate::java_values::JavaValue;
use crate::jvmti::get_state;
use crate::runtime_class::RuntimeClass;
use crate::rust_jni::native_util::{from_object, to_object};

pub unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetTopThreadGroups");
    group_count_ptr.write(1);
    let j_thread_group = jvm.thread_state.system_thread_group.read().unwrap().clone().unwrap();

    dbg!(j_thread_group.threads_non_null().iter().map(|thread| thread.name().to_rust_string()).collect::<Vec<_>>());
    let thread_group_object = j_thread_group.object();
    let mut res = vec![to_object(thread_group_object.into())];
    groups_ptr.write(transmute(res.as_mut_ptr()));//todo fix this bs that requires a transmute
    Vec::leak(res);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetTopThreadGroups");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_all_threads(env: *mut jvmtiEnv, threads_count_ptr: *mut jint, threads_ptr: *mut *mut jthread) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetAllThreads");
    if !jvm.vm_live() {
        threads_count_ptr.write(0);
        threads_ptr.write(jvm.native_interface_allocations.allocate_box(()) as *mut () as *mut c_void as *mut jthread);
        jvm.tracing.trace_jdwp_function_exit(jvm, "GetAllThreads");
        return jvmtiError_JVMTI_ERROR_NONE
    }
    let mut res_ptr = jvm.thread_state.get_all_threads().values().map(|thread| {
        dbg!(thread.thread_object().name().to_rust_string());
        to_object(thread.thread_object().object().into())
    }).collect::<Vec<_>>();
    threads_count_ptr.write(res_ptr.len() as i32);
    threads_ptr.write(transmute(res_ptr.as_mut_ptr()));//todo fix these transmutes
    Vec::leak(res_ptr);//todo memory leak
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetAllThreads");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_thread_info(env: *mut jvmtiEnv, thread: jthread, info_ptr: *mut jvmtiThreadInfo) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadInfo");
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
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetThreadInfo");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_thread_state(env: *mut jvmtiEnv, thread: jthread, thread_state_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadState");
    let jthread = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    let thread = jthread.get_java_thread(jvm);
    let suspended = *thread.suspended.suspended.lock().unwrap();
    let state = JVMTI_THREAD_STATE_ALIVE | if suspended {
        JVMTI_THREAD_STATE_SUSPENDED
    } else {
        JVMTI_THREAD_STATE_ALIVE//todo this is not always correct
    };
    thread_state_ptr.write(state as i32);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetThreadState");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn suspend_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThreadList");
    // dbg!(jvm.thread_state.alive_threads.read().unwrap().keys());
    // dbg!(jvm.thread_state.main_thread.read().unwrap().as_ref().unwrap().java_tid);
    for i in 0..request_count {
        let thread_object_raw = from_object(transmute(request_list.offset(i as isize).read()));//todo this transmute bs will sone have gone too far
        let jthread = JavaValue::Object(thread_object_raw).cast_thread();
        let java_thread = jthread.get_java_thread(jvm);
        results.offset(i as isize).write(suspend_thread_impl(java_thread));
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "SuspendThreadList");
    jvmtiError_JVMTI_ERROR_NONE
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
    let _jvm = get_state(env);
    suspend_thread(env, thread);//todo this is an ugly hack.
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn suspend_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    //todo dubplication
    //todo this part is not correct: If the calling thread is specified, this function will not return until some other thread calls ResumeThread. If the thread is currently suspended, this function does nothing and returns an error.
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SuspendThread");
    let thread_object_raw = from_object(thread);
    let jthread = JavaValue::Object(thread_object_raw).cast_thread();
    let java_thread = jthread.get_java_thread(jvm);
    let res = suspend_thread_impl(java_thread);
    jvm.tracing.trace_jdwp_function_exit(jvm, "SuspendThread");
    res
}

pub unsafe extern "C" fn resume_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThread");
    let thread_object_raw = from_object(thread);
    let jthread = JavaValue::Object(thread_object_raw).cast_thread();
    let java_thread = jthread.get_java_thread(jvm);
    let res = resume_thread_impl(java_thread);
    jvm.tracing.trace_jdwp_function_exit(jvm, "ResumeThread");
    res
}


pub unsafe extern "C" fn resume_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "ResumeThreadList");
    for i in 0..request_count {
        let jthreadp = request_list.offset(i as isize).read();
        let jthread = JavaValue::Object(from_object(jthreadp)).cast_thread();
        let java_thread = jthread.get_java_thread(jvm);
        results.offset(i as isize).write(resume_thread_impl(java_thread));
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "ResumeThreadList");
    jvmtiError_JVMTI_ERROR_NONE
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
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetThreadGroupInfo");
    //todo thread groups not implemented atm.
    let boxed_string = CString::new("main").unwrap().into_boxed_c_str();
    let info_pointer_writer = info_ptr.as_mut().unwrap();
    info_pointer_writer.name = Box::leak(boxed_string).as_ptr() as *mut i8;
    info_pointer_writer.is_daemon = false as jboolean;
    info_pointer_writer.max_priority = 0;
    info_pointer_writer.parent = std::ptr::null_mut();
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetThreadGroupInfo");
    jvmtiError_JVMTI_ERROR_NONE
}