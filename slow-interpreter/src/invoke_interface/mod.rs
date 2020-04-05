use jvmti_bindings::{JNIInvokeInterface_, JavaVM, jint, JVMTI_VERSION_1_0, jvmtiInterface_1_, jvmtiEnv};
use crate::{InterpreterState, StackEntry};
use std::rc::Rc;
use crate::rust_jni::interface::get_interface;
use jni_bindings::{JNINativeInterface_, JNI_OK};
use std::intrinsics::transmute;
use std::ffi::c_void;
use crate::jvmti::get_jvmti_interface;

pub fn get_invoke_interface(state: &mut InterpreterState, frame: Rc<StackEntry>) -> JNIInvokeInterface_ {
    JNIInvokeInterface_ {
        reserved0: unsafe { transmute(state) },
        reserved1: Box::into_raw(Box::new(frame)) as *mut c_void,
        reserved2: std::ptr::null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: Some(get_env),
        AttachCurrentThreadAsDaemon: None,
    }
}

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l mut InterpreterState {
    transmute((**vm).reserved0)
}

pub unsafe fn get_frame_invoke_interface(vm: *mut JavaVM) -> Rc<StackEntry> {
    let res = ((**vm).reserved1 as *mut Rc<StackEntry>).as_ref().unwrap();// ptr::as_ref
    res.clone()
}

pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut ::std::os::raw::c_void, version: jint) -> jint {
    let state = get_state_invoke_interface(vm);
    let frame = get_frame_invoke_interface(vm);
    assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    *(penv as *mut *mut jvmtiEnv)  = Box::leak((get_jvmti_interface(state, frame)).into()) as *mut jvmtiEnv ;
    JNI_OK as i32
}