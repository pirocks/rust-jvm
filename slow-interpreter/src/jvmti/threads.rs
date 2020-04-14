use jvmti_bindings::{jvmtiEnv, jint, jthreadGroup, jvmtiError, jvmtiError_JVMTI_ERROR_NONE};
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
