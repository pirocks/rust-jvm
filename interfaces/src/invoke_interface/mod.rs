use std::ptr::null_mut;

use jvmti_jni_bindings::{JavaVM, jint, JNINativeInterface_, JVMTI_VERSION_1_0, JVMTI_VERSION_1_2, jvmtiEnv};
use jvmti_jni_bindings::{JNI_OK};
use jvmti_jni_bindings::invoke_interface::JNIInvokeInterfaceNamedReservedPointers;
use jvmti_jni_bindings::jni_interface::JNINativeInterfaceNamedReservedPointers;

use slow_interpreter::better_java_stack::native_frame::NativeFrame;
use slow_interpreter::exceptions::WasException;
use slow_interpreter::jvm_state::JVMState;
use slow_interpreter::rust_jni::jvmti::get_jvmti_interface;

//    jni_interface.reserved0 = jvm_ptr;
//     jni_interface.reserved1 = int_state_ptr;
//     jni_interface.reserved2 = exception_pointer;

pub unsafe fn get_state_invoke_interface<'l>(vm: *mut JavaVM) -> &'l JVMState<'l> {
    (**vm).reserved0.cast::<JVMState<'l>>().as_ref().unwrap()
}

pub unsafe fn get_interpreter_state_invoke_interface<'gc, 'l, 'any>(vm: *mut JavaVM) -> &'any mut NativeFrame<'gc, 'l> {
    (**vm).reserved1.cast::<NativeFrame<'gc, 'l>>().as_mut().unwrap()
}

pub unsafe fn get_throw_invoke_interface<'gc, 'l>(vm: *mut JavaVM) -> &'l mut Option<WasException<'gc>> {
    (**vm).reserved2.cast::<Option<WasException<'gc>>>().as_mut().unwrap()
}

pub unsafe extern "C" fn get_env(vm: *mut JavaVM, penv: *mut *mut ::std::os::raw::c_void, version: jint) -> jint {
    let jvm = get_state_invoke_interface(vm);
    let int_state = get_interpreter_state_invoke_interface(vm);
    // assert_eq!(version, JVMTI_VERSION_1_0 as i32);
    if version == JVMTI_VERSION_1_0 as i32 || version == JVMTI_VERSION_1_2 as i32 {
        //todo do a proper jvmti_interface check
        (penv as *mut *mut jvmtiEnv).write(get_jvmti_interface(jvm, todo!()/*int_state*/));
    } else {
        //todo fix this.
        let jni_native_interface = int_state.stack_jni_interface().jni_inner_mut();
        (penv as *mut *mut *const JNINativeInterface_).write(Box::into_raw(box (jni_native_interface as *mut JNINativeInterfaceNamedReservedPointers as *const JNINativeInterface_)));
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