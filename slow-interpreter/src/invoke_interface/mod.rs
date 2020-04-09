use jvmti_bindings::{JNIInvokeInterface_, JavaVM, jint, JVMTI_VERSION_1_0, jvmtiEnv};
use crate::{JVMState, StackEntry};
use std::rc::Rc;
use jni_bindings::JNI_OK;
use std::intrinsics::transmute;
use crate::jvmti::get_jvmti_interface;

pub fn get_invoke_interface(state: &JVMState) -> JNIInvokeInterface_ {
    JNIInvokeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: std::ptr::null_mut(),
        reserved2: std::ptr::null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: Some(get_env),
        AttachCurrentThreadAsDaemon: None,
    }
}

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l JVMState/*<'l>*/ {
    transmute((**vm).reserved0)
}

pub unsafe fn get_frame_invoke_interface(vm: *mut JavaVM) -> Rc<StackEntry> {
    get_state_invoke_interface(vm).get_current_thread().call_stack.clone()
}

pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut ::std::os::raw::c_void, version: jint) -> jint {
    let state = get_state_invoke_interface(vm);
    let frame = get_frame_invoke_interface(vm);
    assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    *(penv as *mut *mut jvmtiEnv) = Box::leak((get_jvmti_interface(state)).into()) as *mut jvmtiEnv;
    JNI_OK as i32
}