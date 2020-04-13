use jvmti_bindings::{jvmtiEnv, jint, jthreadGroup, jvmtiError};

unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    unimplemented!();
}
