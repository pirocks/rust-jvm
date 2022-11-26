use std::ffi::c_void;
use crate::{JavaVM, jint, jmmInterface_1_, JNIInvokeInterface_, JNINativeInterface_, jvmtiInterface_1_};
use crate::jmm_interface::JMMInterfaceNamedReservedPointers;
use crate::jni_interface::JNINativeInterfaceNamedReservedPointers;
use crate::jvmti_interface::JVMTIInterfaceNamedReservedPointers;

// pointers I need from here:
// jvm pointer
// jni interface pointer
// jvmti interface pointer?
//jmm interface

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct JNIInvokeInterfaceNamedReservedPointers {
    pub jvm_state: *mut c_void,
    //todo if using getenv on different thread I kinda have to do thread local shenanigans?
    pub other_native_interfaces_this_thread: *mut (*mut JNINativeInterfaceNamedReservedPointers, *mut JVMTIInterfaceNamedReservedPointers, *mut JMMInterfaceNamedReservedPointers),
    pub _unused: *mut c_void,
    pub DestroyJavaVM: Option<unsafe extern "C" fn(vm: *mut JavaVM) -> jint>,
    pub AttachCurrentThread: Option<unsafe extern "C" fn(vm: *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint>,
    pub DetachCurrentThread: Option<unsafe extern "C" fn(vm: *mut JavaVM) -> jint>,
    pub GetEnv: Option<unsafe extern "C" fn(vm: *mut JavaVM, penv: *mut *mut c_void, version: jint) -> jint>,
    pub AttachCurrentThreadAsDaemon: Option<unsafe extern "C" fn(vm: *mut JavaVM, penv: *mut *mut c_void, args: *mut c_void) -> jint>,
}
