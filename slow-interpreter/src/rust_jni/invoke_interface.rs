use std::mem::transmute;
use jvmti_jni_bindings::JNIInvokeInterface_;
use crate::{JVMState, OpaqueFrame};

pub fn get_invoke_interface_new<'gc, 'l>(jvm: &JVMState, opaque_frame: &mut OpaqueFrame<'gc, 'l>) -> *const JNIInvokeInterface_ {
    let mut guard = jvm.native.invoke_interface.write().unwrap();
    match guard.as_ref() {
        None => unsafe {
            let mut invoke_interface = jvm.default_per_stack_initial_interfaces.invoke_interface.clone();
            invoke_interface.reserved0 = transmute(jvm);
            guard.replace(Box::leak(box invoke_interface) as *const JNIInvokeInterface_/*unsafe {
                Box::leak(box JNIInvokeInterface_ {
                    reserved0: transmute(jvm),
                    reserved1: null_mut(),
                    reserved2: null_mut()/*transmute(opaque_frame.stack_guard())*/,
                    DestroyJavaVM: None,
                    AttachCurrentThread: None,
                    DetachCurrentThread: None,
                    GetEnv: Some(get_env),
                    AttachCurrentThreadAsDaemon: None,
                })
            }*/);
        }
        Some(_) => {}
    }
    drop(guard);
    *jvm.native.invoke_interface.read().unwrap().as_ref().unwrap()
}

