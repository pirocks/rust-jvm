use std::mem::transmute;
use jvmti_jni_bindings::invoke_interface::JNIInvokeInterfaceNamedReservedPointers;
use crate::{JVMState};

pub fn get_invoke_interface_new<'gc, 'l>(jvm: &JVMState) -> *const JNIInvokeInterfaceNamedReservedPointers {
    jvm.native.invoke_interface.with(|this_thread_invoke_interface|{
        let mut invoke_interface = *jvm.default_per_stack_initial_interfaces.invoke_interface.clone();
        unsafe { invoke_interface.jvm_state = transmute(jvm); }
        this_thread_invoke_interface.replace(Some(Box::leak(Box::new(invoke_interface)) as *const JNIInvokeInterfaceNamedReservedPointers));
        *this_thread_invoke_interface.borrow().as_ref().unwrap()
    })
}

