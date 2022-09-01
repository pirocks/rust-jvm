use std::mem::transmute;

use jvmti_jni_bindings::{JavaVM, jint, JNIInvokeInterface_, JVMTI_VERSION_1_0, JVMTI_VERSION_1_2, jvmtiEnv};
use jvmti_jni_bindings::{JNI_OK, JNINativeInterface_};

use crate::{InterpreterStateGuard, JVMState};
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::jvmti::get_jvmti_interface;
use crate::rust_jni::interface::get_interface;

pub fn get_invoke_interface<'gc, 'l>(jvm: &JVMState, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> *const JNIInvokeInterface_ {
    let mut guard = jvm.native.invoke_interface.write().unwrap();
    match guard.as_ref() {
        None => {
            guard.replace(unsafe {
                Box::leak(box JNIInvokeInterface_ {
                    reserved0: transmute(jvm),
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
    *jvm.native.invoke_interface.read().unwrap().as_ref().unwrap()
}

pub fn get_invoke_interface_new<'gc, 'l>(jvm: &JVMState, opaque_frame: &mut OpaqueFrame<'gc,'l>) -> *const JNIInvokeInterface_ {
    let mut guard = jvm.native.invoke_interface.write().unwrap();
    match guard.as_ref() {
        None => {
            guard.replace(unsafe {
                Box::leak(box JNIInvokeInterface_ {
                    reserved0: transmute(jvm),
                    reserved1: std::ptr::null_mut(),
                    reserved2: transmute(opaque_frame),
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
    *jvm.native.invoke_interface.read().unwrap().as_ref().unwrap()
}

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l JVMState<'l> {
    &*((**vm).reserved0 as *const JVMState)
}

pub unsafe fn get_interpreter_state_invoke_interface<'l, 'interpreter_guard>(vm: *mut JavaVM) -> &'l mut InterpreterStateGuard<'l,'interpreter_guard> {
    let jvm = get_state_invoke_interface(vm);
    jvm.get_int_state()
}

pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut ::std::os::raw::c_void, version: jint) -> jint {
    let state = get_state_invoke_interface(vm);
    let int_state = todo!()/*get_interpreter_state_invoke_interface(vm)*/;
    // assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    if version == JVMTI_VERSION_1_0 as i32 || version == JVMTI_VERSION_1_2 as i32 {
        //todo do a proper jvmti check
        (penv as *mut *mut jvmtiEnv).write(get_jvmti_interface(state, int_state));
    } else {
        let res_ptr = get_interface(state, /*int_state*/todo!());
        (penv as *mut *mut *const JNINativeInterface_).write(res_ptr);
    }

    JNI_OK as i32
}