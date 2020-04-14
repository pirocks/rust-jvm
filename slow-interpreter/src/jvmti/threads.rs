use jvmti_bindings::{jvmtiEnv, jint, jthreadGroup, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jthread};
use crate::jvmti::get_state;
use crate::rust_jni::native_util::to_object;
use std::intrinsics::transmute;

pub unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    let state = get_state(env);
    group_count_ptr.write(1);
    // dbg!(groups_ptr);
    let mut res = vec![to_object(state.system_thread_group.read().unwrap().clone())];
    groups_ptr.write(transmute(res.as_mut_ptr()) );//todo fix this bs that requires a transmute
    Vec::leak(res);
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_all_threads(env: *mut jvmtiEnv, threads_count_ptr: *mut jint, threads_ptr: *mut *mut jthread) -> jvmtiError{
    let jvm = get_state(env);
    let mut res_ptr = vec![];
    jvm.alive_threads.read().unwrap().values().for_each(|v|{
        let thread_object_arc = v.thread_object.borrow().as_ref().unwrap().clone();
        res_ptr.push(to_object(thread_object_arc.object().into()));
    });
    threads_count_ptr.write(res_ptr.len() as i32);
    threads_ptr.write(transmute(res_ptr.as_mut_ptr()));//todo fix these transmutes
    Vec::leak(res_ptr);//todo memory leak
    jvmtiError_JVMTI_ERROR_NONE
}