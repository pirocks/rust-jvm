use jvmti_bindings::{jvmtiEnv, jint, jthreadGroup, jvmtiError, jvmtiError_JVMTI_ERROR_NONE, jthread, jvmtiThreadInfo};
use crate::jvmti::get_state;
use crate::rust_jni::native_util::{to_object, from_object};
use std::intrinsics::transmute;
use crate::java_values::JavaValue;
use classfile_view::view::ptype_view::PTypeView;
use std::ffi::CString;

pub unsafe extern "C" fn get_top_thread_groups(env: *mut jvmtiEnv, group_count_ptr: *mut jint, groups_ptr: *mut *mut jthreadGroup) -> jvmtiError {
    let jvm = get_state(env);
    group_count_ptr.write(1);
    // dbg!(groups_ptr);
    let mut res = vec![to_object(jvm.thread_state.system_thread_group.read().unwrap().clone())];
    groups_ptr.write(transmute(res.as_mut_ptr()) );//todo fix this bs that requires a transmute
    Vec::leak(res);
    jvmtiError_JVMTI_ERROR_NONE
}


pub unsafe extern "C" fn get_all_threads(env: *mut jvmtiEnv, threads_count_ptr: *mut jint, threads_ptr: *mut *mut jthread) -> jvmtiError{
    let jvm = get_state(env);
    let mut res_ptr = vec![];
    jvm.thread_state.alive_threads.read().unwrap().values().for_each(|v|{
        let thread_object_arc = v.thread_object.borrow().as_ref().unwrap().clone();
        res_ptr.push(to_object(thread_object_arc.object().into()));
    });
    threads_count_ptr.write(res_ptr.len() as i32);
    threads_ptr.write(transmute(res_ptr.as_mut_ptr()));//todo fix these transmutes
    Vec::leak(res_ptr);//todo memory leak
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_thread_info(env: *mut jvmtiEnv, thread: jthread, info_ptr: *mut jvmtiThreadInfo) -> jvmtiError{
    let jvm = get_state(env);
    let thread_object = JavaValue::Object(from_object(transmute(thread))).cast_thread();
    (*info_ptr).thread_group = transmute(to_object(jvm.thread_state.system_thread_group.read().unwrap().clone()));//todo get thread groups working at some point
    (*info_ptr).context_class_loader = transmute(to_object(jvm.class_object_pool.read().unwrap().get(&PTypeView::IntType).unwrap().lookup_field("classLoader").unwrap_object()));//todo deal with this whole loader situation
    (*info_ptr).name = CString::new(thread_object.name().to_rust_string()).unwrap().into_raw();//todo leak
    (*info_ptr).is_daemon = thread_object.daemon() as u8;//todo this issue again
    (*info_ptr).priority = thread_object.priority();
    jvmtiError_JVMTI_ERROR_NONE
}