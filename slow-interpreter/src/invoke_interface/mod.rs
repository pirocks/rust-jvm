use std::mem::transmute;
use std::ptr::null_mut;

use jvmti_jni_bindings::{JavaVM, jint, JNIInvokeInterface_, JVMTI_VERSION_1_0, JVMTI_VERSION_1_2, jvmtiEnv};
use jvmti_jni_bindings::{JNI_OK, JNINativeInterface_};

use crate::{JVMState, WasException};
use crate::better_java_stack::native_frame::NativeFrame;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::rust_jni::interface::jvmti::get_jvmti_interface;

pub fn get_invoke_interface<'gc, 'l>(jvm: &JVMState, int_state: &mut NativeFrame<'gc, 'l>) -> *const JNIInvokeInterface_ {
    let mut guard = jvm.native.invoke_interface.write().unwrap();
    match guard.as_ref() {
        None => {
            guard.replace(unsafe {
                Box::leak(box JNIInvokeInterface_ {
                    reserved0: transmute(jvm),
                    reserved1: transmute(int_state),
                    reserved2: null_mut(),
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

pub fn get_invoke_interface_new<'gc, 'l>(jvm: &JVMState, opaque_frame: &mut OpaqueFrame<'gc, 'l>) -> *const JNIInvokeInterface_ {
    let mut guard = jvm.native.invoke_interface.write().unwrap();
    match guard.as_ref() {
        None => {
            guard.replace(unsafe {
                Box::leak(box JNIInvokeInterface_ {
                    reserved0: transmute(jvm),
                    reserved1: null_mut(),
                    reserved2: null_mut()/*transmute(opaque_frame.stack_guard())*/,
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

//    interface.reserved0 = jvm_ptr;
//     interface.reserved1 = int_state_ptr;
//     interface.reserved2 = exception_pointer;

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
        //todo do a proper jvmti check
        (penv as *mut *mut jvmtiEnv).write(get_jvmti_interface(jvm, todo!()/*int_state*/));
    } else {
        //todo fix this.
        let jni_native_interface = int_state.stack_jni_interface().jni_inner_mut();
        (penv as *mut *mut *const JNINativeInterface_).write(Box::into_raw(box (jni_native_interface as *const JNINativeInterface_)));
    }

    JNI_OK as i32
}