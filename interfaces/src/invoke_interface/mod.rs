use std::os::raw::c_void;
use std::ptr::null_mut;

use jvmti_jni_bindings::{JavaVM, jint, JNINativeInterface_, JVMTI_VERSION_1_0, JVMTI_VERSION_1_2, jvmtiEnv};
use jvmti_jni_bindings::{JNI_OK};
use jvmti_jni_bindings::invoke_interface::{JavaVMNamedReservedPointers, JNIInvokeInterfaceNamedReservedPointers};
use jvmti_jni_bindings::jni_interface::JNINativeInterfaceNamedReservedPointers;

use slow_interpreter::exceptions::WasException;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::jvmti::get_jvmti_interface;

//    jni_interface.reserved0 = jvm_ptr;
//     jni_interface.reserved1 = int_state_ptr;
//     jni_interface.reserved2 = exception_pointer;

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l JVMState<'l> {
    let vm = vm as *mut JavaVMNamedReservedPointers;
    &*((**vm).jvm_state as *const JVMState<'l>)
}

pub unsafe fn get_jni_interface_invoke_interface<'gc, 'l, 'any>(vm: *mut JavaVM) -> *mut JNINativeInterfaceNamedReservedPointers {
    let vm = vm as *mut JavaVMNamedReservedPointers;
    ((**vm).other_native_interfaces_this_thread).as_ref().unwrap().0
}

pub unsafe fn get_throw_invoke_interface<'gc, 'l>(vm: *mut JavaVM) -> &'l mut Option<WasException<'gc>> {
    todo!()
    /*(**vm).reserved2.cast::<Option<WasException<'gc>>>().as_mut().unwrap()*/
}

pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut c_void, version: jint) -> jint {
    let jvm = get_state_invoke_interface(vm);
    // let int_state = get_interpreter_state_invoke_interface(vm);
    // assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    if version == JVMTI_VERSION_1_0 as i32 || version == JVMTI_VERSION_1_2 as i32 {
        //todo do a proper jvmti_interface check
        (penv as *mut *mut jvmtiEnv).write(get_jvmti_interface(jvm, todo!()/*int_state*/));
    } else {
        //todo this is really not correct in the case of multiple thread interactions
        let jni_native_interface = get_jni_interface_invoke_interface(vm);
        let res = Box::into_raw(box (jni_native_interface as *const JNINativeInterface_));
        (penv as *mut *mut *const JNINativeInterface_).write(res);
    }

    JNI_OK as i32
}

pub fn initial_invoke_interface() -> JNIInvokeInterfaceNamedReservedPointers {
    JNIInvokeInterfaceNamedReservedPointers {
        jvm_state: null_mut(),
        other_native_interfaces_this_thread: null_mut(),
        _unused: null_mut(),
        DestroyJavaVM: None,
        AttachCurrentThread: None,
        DetachCurrentThread: None,
        GetEnv: Some(get_env),
        AttachCurrentThreadAsDaemon: None,
    }
}