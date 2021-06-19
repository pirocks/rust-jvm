use std::mem::transmute;

use jvmti_jni_bindings::{JavaVM, jint, JNIInvokeInterface_, JVMTI_VERSION_1_0, JVMTI_VERSION_1_2, jvmtiEnv};
use jvmti_jni_bindings::{JNI_OK, JNINativeInterface_};

use crate::{InterpreterStateGuard, JVMState};
use crate::jvmti::get_jvmti_interface;
use crate::rust_jni::interface::get_interface;

pub fn get_invoke_interface(state: &JVMState, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> *const JNIInvokeInterface_ {
    let mut guard = state.invoke_interface.write().unwrap();
    match guard.as_ref() {
        None => {
            guard.replace(unsafe {
                Box::leak(box JNIInvokeInterface_ {
                    reserved0: transmute(state),
                    reserved1: transmute(int_state),
                    reserved2: std::ptr::null_mut(),
                    DestroyJavaVM: None,
                    AttachCurrentThread: None,
                    DetachCurrentThread: None,
                    GetEnv: Some(get_env),
                    AttachCurrentThreadAsDaemon: None,
                }) as *const JNIInvokeInterface_
            });
        }
        Some(_) => {}
    }
    drop(guard);
    *state.invoke_interface.read().unwrap().as_ref().unwrap()
}

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l JVMState<'l> {
    &*((**vm).reserved0 as *const JVMState)
}

pub unsafe fn get_interpreter_state_invoke_interface<'l, 'k>(vm: *mut JavaVM) -> &'l mut InterpreterStateGuard<'l, 'k> {
    let jvm = get_state_invoke_interface(vm);
    jvm.get_int_state()
}


pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut ::std::os::raw::c_void, version: jint) -> jint {
    let state = get_state_invoke_interface(vm);
    let int_state = get_interpreter_state_invoke_interface(vm);
    // assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    if version == JVMTI_VERSION_1_0 as i32 || version == JVMTI_VERSION_1_2 as i32 {
        //todo do a proper jvmti check
        (penv as *mut *mut jvmtiEnv).write(get_jvmti_interface(state, int_state));
    } else {
        let res_ptr = get_interface(state, int_state);
        (penv as *mut *mut *const JNINativeInterface_).write(res_ptr);
    }

    JNI_OK as i32
}