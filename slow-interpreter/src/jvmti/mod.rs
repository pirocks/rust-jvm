use std::mem::transmute;
use std::ptr::null_mut;

use jvmti_jni_bindings::*;

use crate::{InterpreterStateGuard, JVMState};
use crate::jvmti::agent::*;
use crate::jvmti::allocate::*;
use crate::jvmti::breakpoint::*;
use crate::jvmti::capabilities::*;
use crate::jvmti::classes::*;
use crate::jvmti::event_callbacks::set_event_callbacks;
use crate::jvmti::events::set_event_notification_mode;
use crate::jvmti::field::*;
use crate::jvmti::frame::*;
use crate::jvmti::is::*;
use crate::jvmti::locals::*;
use crate::jvmti::methods::*;
use crate::jvmti::monitor::*;
use crate::jvmti::object::get_object_hash_code;
use crate::jvmti::properties::get_system_property;
use crate::jvmti::tags::*;
use crate::jvmti::thread_local_storage::*;
use crate::jvmti::threads::*;
use crate::jvmti::threads::suspend_resume::*;
use crate::jvmti::threads::thread_groups::*;
use crate::jvmti::version::get_version_number;

pub mod event_callbacks;

//todo handle early return message here?
#[macro_export]
macro_rules! null_check {
    ($ptr: expr) => {
        if $ptr == std::ptr::null_mut() {
            return crate::jvmti::jvmtiError_JVMTI_ERROR_NULL_POINTER
        }
    };
}


pub unsafe fn get_state(env: *mut jvmtiEnv) -> &'static JVMState {
    transmute((**env).reserved1)
}


pub unsafe fn get_interpreter_state<'l>(env: *mut jvmtiEnv) -> &'l mut InterpreterStateGuard<'l> {
    let jvm = get_state(env);
    jvm.get_int_state()
}


pub fn get_jvmti_interface(jvm: &JVMState, _int_state: &mut InterpreterStateGuard) -> *mut jvmtiEnv {
    let new = get_jvmti_interface_impl(jvm);
    let jni_data_structure_ptr = Box::leak(box (Box::leak(box new) as *const jvmtiInterface_1_)) as *mut jvmtiEnv;
    jni_data_structure_ptr
}

fn get_jvmti_interface_impl(jvm: &JVMState) -> jvmtiInterface_1_ {
    jvmtiInterface_1_ {
        reserved1: unsafe { transmute(jvm) },
        SetEventNotificationMode: Some(set_event_notification_mode),
        reserved3: null_mut(),
        GetAllThreads: Some(get_all_threads),
        SuspendThread: Some(suspend_thread),
        ResumeThread: Some(resume_thread),
        StopThread: None,
        InterruptThread: Some(interrupt_thread),//todo technically these are different.For now should be fine though
        GetThreadInfo: Some(get_thread_info),
        GetOwnedMonitorInfo: None,
        GetCurrentContendedMonitor: None,
        RunAgentThread: Some(run_agent_thread),
        GetTopThreadGroups: Some(get_top_thread_groups),
        GetThreadGroupInfo: Some(get_thread_group_info),
        GetThreadGroupChildren: None,
        GetFrameCount: Some(get_frame_count),
        GetThreadState: Some(get_thread_state),
        GetCurrentThread: None,
        GetFrameLocation: Some(get_frame_location),
        NotifyFramePop: None,
        GetLocalObject: Some(get_local_object),
        GetLocalInt: Some(get_local_int),
        GetLocalLong: Some(get_local_long),
        GetLocalFloat: Some(get_local_float),
        GetLocalDouble: Some(get_local_double),
        SetLocalObject: None,
        SetLocalInt: None,
        SetLocalLong: None,
        SetLocalFloat: None,
        SetLocalDouble: None,
        CreateRawMonitor: Some(create_raw_monitor),
        DestroyRawMonitor: Some(destroy_raw_monitor),
        RawMonitorEnter: Some(raw_monitor_enter),
        RawMonitorExit: Some(raw_monitor_exit),
        RawMonitorWait: Some(raw_monitor_wait),
        RawMonitorNotify: Some(raw_monitor_notify),
        RawMonitorNotifyAll: Some(raw_monitor_notify_all),
        SetBreakpoint: Some(set_breakpoint),
        ClearBreakpoint: Some(clear_breakpoint),
        reserved40: std::ptr::null_mut(),
        SetFieldAccessWatch: None,
        ClearFieldAccessWatch: None,
        SetFieldModificationWatch: None,
        ClearFieldModificationWatch: None,
        IsModifiableClass: None,
        Allocate: Some(allocate),
        Deallocate: Some(deallocate),
        GetClassSignature: Some(get_class_signature),
        GetClassStatus: Some(get_class_status),
        GetSourceFileName: Some(get_source_file_name),
        GetClassModifiers: None,
        GetClassMethods: Some(get_class_methods),
        GetClassFields: Some(get_class_fields),
        GetImplementedInterfaces: Some(get_implemented_interfaces),
        IsInterface: Some(is_interface),
        IsArrayClass: Some(is_array_class),
        GetClassLoader: Some(get_class_loader),
        GetObjectHashCode: Some(get_object_hash_code),
        GetObjectMonitorUsage: None,
        GetFieldName: Some(get_field_name),
        GetFieldDeclaringClass: None,
        GetFieldModifiers: Some(get_field_modifiers),
        IsFieldSynthetic: Some(is_field_synthetic),
        GetMethodName: Some(get_method_name),
        GetMethodDeclaringClass: Some(get_method_declaring_class),
        GetMethodModifiers: Some(get_method_modifiers),
        reserved67: std::ptr::null_mut(),
        GetMaxLocals: None,
        GetArgumentsSize: Some(get_arguments_size),
        GetLineNumberTable: Some(get_line_number_table),
        GetMethodLocation: Some(get_method_location),
        GetLocalVariableTable: Some(get_local_variable_table),
        SetNativeMethodPrefix: None,
        SetNativeMethodPrefixes: None,
        GetBytecodes: None,
        IsMethodNative: Some(is_method_native),
        IsMethodSynthetic: Some(is_method_synthetic),
        GetLoadedClasses: Some(get_loaded_classes),
        GetClassLoaderClasses: None,
        PopFrame: None,
        ForceEarlyReturnObject: None,
        ForceEarlyReturnInt: None,
        ForceEarlyReturnLong: None,
        ForceEarlyReturnFloat: None,
        ForceEarlyReturnDouble: None,
        ForceEarlyReturnVoid: None,
        RedefineClasses: None,
        GetVersionNumber: Some(get_version_number),
        GetCapabilities: Some(get_capabilities),
        GetSourceDebugExtension: None,
        IsMethodObsolete: Some(is_method_obsolete),
        SuspendThreadList: Some(suspend_thread_list),
        ResumeThreadList: Some(resume_thread_list),
        reserved94: std::ptr::null_mut(),
        reserved95: std::ptr::null_mut(),
        reserved96: std::ptr::null_mut(),
        reserved97: std::ptr::null_mut(),
        reserved98: std::ptr::null_mut(),
        reserved99: std::ptr::null_mut(),
        GetAllStackTraces: None,
        GetThreadListStackTraces: None,
        GetThreadLocalStorage: Some(get_thread_local_storage),
        SetThreadLocalStorage: Some(set_thread_local_storage),
        GetStackTrace: None,
        reserved105: std::ptr::null_mut(),
        GetTag: Some(get_tag),
        SetTag: Some(set_tag),
        ForceGarbageCollection: None,
        IterateOverObjectsReachableFromObject: None,
        IterateOverReachableObjects: None,
        IterateOverHeap: None,
        IterateOverInstancesOfClass: None,
        reserved113: std::ptr::null_mut(),
        GetObjectsWithTags: None,
        FollowReferences: None,
        IterateThroughHeap: None,
        reserved117: std::ptr::null_mut(),
        reserved118: std::ptr::null_mut(),
        reserved119: std::ptr::null_mut(),
        SetJNIFunctionTable: None,
        GetJNIFunctionTable: None,
        SetEventCallbacks: Some(set_event_callbacks),
        GenerateEvents: None,
        GetExtensionFunctions: None,
        GetExtensionEvents: None,
        SetExtensionEventCallback: None,
        DisposeEnvironment: Some(dispose_environment),
        GetErrorName: None,
        GetJLocationFormat: None,
        GetSystemProperties: None,
        GetSystemProperty: Some(get_system_property),
        SetSystemProperty: None,
        GetPhase: None,
        GetCurrentThreadCpuTimerInfo: None,
        GetCurrentThreadCpuTime: None,
        GetThreadCpuTimerInfo: None,
        GetThreadCpuTime: None,
        GetTimerInfo: None,
        GetTime: None,
        GetPotentialCapabilities: Some(get_potential_capabilities),
        reserved141: std::ptr::null_mut(),
        AddCapabilities: Some(add_capabilities),
        RelinquishCapabilities: None,
        GetAvailableProcessors: None,
        GetClassVersionNumbers: None,
        GetConstantPool: None,
        GetEnvironmentLocalStorage: None,
        SetEnvironmentLocalStorage: None,
        AddToBootstrapClassLoaderSearch: None,
        SetVerboseFlag: None,
        AddToSystemClassLoaderSearch: None,
        RetransformClasses: None,
        GetOwnedMonitorStackDepthInfo: None,
        GetObjectSize: None,
        GetLocalInstance: None,
    }
}


pub mod object;
pub mod methods;
pub mod is;
pub mod breakpoint;
#[macro_use]
pub mod threads;
#[macro_use]
pub mod frame;
#[macro_use]
pub mod thread_local_storage;
pub mod agent;
pub mod classes;
pub mod tags;
pub mod monitor;
pub mod capabilities;
pub mod version;
pub mod properties;
pub mod allocate;
pub mod events;
pub mod field;
pub mod locals;

