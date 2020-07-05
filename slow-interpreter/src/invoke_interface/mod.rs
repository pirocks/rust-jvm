use jvmti_jni_bindings::{JNIInvokeInterface_, JavaVM, jint, JVMTI_VERSION_1_0, jvmtiEnv,  JVMTI_VERSION_1_2};
use crate::{JVMState, InterpreterStateGuard};

use jvmti_jni_bindings::{JNI_OK, JNINativeInterface_};
use std::intrinsics::transmute;
use crate::jvmti::get_jvmti_interface;
use crate::rust_jni::interface::get_interface;


pub fn get_invoke_interface(state: &'static JVMState,int_state: &mut InterpreterStateGuard) -> *const JNIInvokeInterface_ {
    let read_guard = state.invoke_interface.read().unwrap();
    match read_guard.as_ref() {
        None => {
            std::mem::drop(read_guard);
            state.invoke_interface.write().unwrap().replace(unsafe {transmute::<_,jvmti_jni_bindings::JNIInvokeInterface_>(JNIInvokeInterface_ {
                reserved0:  transmute(state) ,
                reserved1: transmute(int_state),
                reserved2: std::ptr::null_mut(),
                DestroyJavaVM: None,
                AttachCurrentThread: None,
                DetachCurrentThread: None,
                GetEnv: Some(get_env),
                AttachCurrentThreadAsDaemon: None,
            })}.into());
        },
        Some(_) => {},
    }
    state.invoke_interface.read().unwrap().as_ref().unwrap() as *const jvmti_jni_bindings::JNIInvokeInterface_ as *const JNIInvokeInterface_
}

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l JVMState/*<'l>*/ {
    transmute((**vm).reserved0)
}

pub unsafe fn get_interpreter_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l mut InterpreterStateGuard<'l> {
    transmute((**vm).reserved1)
}


pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut ::std::os::raw::c_void, version: jint) -> jint {
    let state = get_state_invoke_interface(vm);
    let int_state = get_interpreter_state_invoke_interface(vm);
    // assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    if version == JVMTI_VERSION_1_0 as i32 || version == JVMTI_VERSION_1_2 as i32 {
        //todo do a proper jvmti check
        *(penv as *mut *mut jvmtiEnv) = Box::leak((get_jvmti_interface(state)).into()) as *mut jvmtiEnv;
    }else {
        let res_ptr = get_interface(state,int_state) ;
        (penv as *mut *mut *const JNINativeInterface_).write(Box::into_raw(Box::new(res_ptr)));
    }

    JNI_OK as i32
}