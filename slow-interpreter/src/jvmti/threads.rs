use jvmti_bindings::{jvmtiEnv, jint, jthreadGroup, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jthread, jvmtiThreadInfo, jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE, jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED};
use crate::jvmti::get_state;
use crate::rust_jni::native_util::{to_object, from_object};
use std::intrinsics::transmute;
use crate::java_values::JavaValue;
use classfile_view::view::ptype_view::PTypeView;
use std::ffi::CString;
use std::sync::Arc;
use crate::JavaThread;

pub unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetTopThreadGroups");
    group_count_ptr.write(1);
    let mut res = vec![to_object(jvm.thread_state.system_thread_group.read().unwrap().clone())];
    groups_ptr.write(transmute(res.as_mut_ptr()));//todo fix this bs that requires a transmute
    Vec::leak(res);
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetTopThreadGroups");
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_all_threads(env: *mut jvmtiEnv, threads_count_ptr: *mut jint, threads_ptr: *mut *mut jthread) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetAllThreads");
    let mut res_ptr = vec![];
    //todo why is main not an alive thread
    std::mem::drop(jvm.thread_state.alive_threads.read().unwrap().values()
        /*.chain(vec![/*chain(vec![jvm.thread_state.main_thread.read().unwrap().clone().unwrap()].iter())*/jvm.thread_state.main_thread.read().unwrap().clone().unwrap()].iter())*/
        .map(|v| {
            let thread_object_arc = v.thread_object.borrow().as_ref().unwrap().clone();
            // dbg!(thread_object_arc.tid());
            // dbg!(thread_object_arc.name().to_rust_string());
            res_ptr.push(to_object(thread_object_arc.object().into()));
        }).collect::<Vec<()>>());
    threads_count_ptr.write(res_ptr.len() as i32);
    threads_ptr.write(transmute(res_ptr.as_mut_ptr()));//todo fix these transmutes
    Vec::leak(res_ptr);//todo memory leak
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetAllThreads");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_thread_info(env: *mut jvmtiEnv, thread: jthread, info_ptr: *mut jvmtiThreadInfo) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"GetThreadInfo");
    let thread_object = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    (*info_ptr).thread_group = transmute(to_object(jvm.thread_state.system_thread_group.read().unwrap().clone()));//todo get thread groups working at some point
    (*info_ptr).context_class_loader = transmute(to_object(jvm.class_object_pool.read().unwrap().get(&PTypeView::IntType).unwrap().lookup_field("classLoader").unwrap_object()));//todo deal with this whole loader situation
    (*info_ptr).name = CString::new(thread_object.name().to_rust_string()).unwrap().into_raw();//todo leak
    (*info_ptr).is_daemon = thread_object.daemon() as u8;//todo this issue again
    (*info_ptr).priority = thread_object.priority();
    jvm.tracing.trace_jdwp_function_exit(jvm,"GetThreadInfo");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_thread_state(_env: *mut jvmtiEnv, _thread: jthread, _thread_state_ptr: *mut jint) -> jvmtiError {
    unimplemented!();
    // jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn suspend_thread_list(env: *mut jvmtiEnv, request_count: jint, request_list: *const jthread, results: *mut jvmtiError) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"SuspendThreadList");
    // dbg!(jvm.thread_state.alive_threads.read().unwrap().keys());
    // dbg!(jvm.thread_state.main_thread.read().unwrap().as_ref().unwrap().java_tid);
    for i in 0..request_count {
        let thread_object_raw = from_object(transmute(request_list.offset(i as isize).read()));//todo this transmute bs will sone have gone too far
        let thread_object = JavaValue::Object(thread_object_raw).cast_thread();
        dbg!(thread_object.tid());
        dbg!(thread_object.name().to_rust_string());
        let thread_id = thread_object.tid();
        let java_thread = jvm.thread_state.alive_threads.read().unwrap().get(&thread_id).map(|x| x.clone());
        results.offset(i as isize).write(suspend_thread_impl(java_thread));
    }
    jvm.tracing.trace_jdwp_function_exit(jvm,"SuspendThreadList");
    jvmtiError_JVMTI_ERROR_NONE
}

fn suspend_thread_impl(java_thread: Option<Arc<JavaThread>>) -> jvmtiError {
    match java_thread {
        None => {
            jvmtiError_JVMTI_ERROR_THREAD_NOT_ALIVE
        }
        Some(java_thread) => {
            let mut suspend_info = java_thread.interpreter_state.suspended.write().unwrap();
            if suspend_info.suspended {
                jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED
            } else {
                suspend_info.suspended = true;
                std::mem::forget(suspend_info.suspended_lock.lock());
                jvmtiError_JVMTI_ERROR_NONE
            }
        }
    }
}

pub unsafe extern "C" fn suspend_thread(env: *mut jvmtiEnv, thread: jthread) -> jvmtiError {
    //todo dubplication
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"SuspendThread");
    let thread_object_raw = from_object(transmute(thread));//todo this transmute bs will sone have gone too far
    let thread_id = JavaValue::Object(thread_object_raw).cast_thread().tid();
    let java_thread = jvm.thread_state.alive_threads.read().unwrap().get(&thread_id).map(|x| x.clone());
    let res = suspend_thread_impl(java_thread);
    jvm.tracing.trace_jdwp_function_exit(jvm,"SuspendThread");
    res

}
