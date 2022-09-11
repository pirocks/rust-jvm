use std::ptr::null_mut;
use jvmti_jni_bindings::{jvmtiEnv, jvmtiInterface_1_};
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::{InterpreterStateGuard, JVMState};
use crate::rust_jni::jvmti_interface::agent::run_agent_thread;
use crate::rust_jni::jvmti_interface::allocate::{allocate, deallocate, dispose_environment};
use crate::rust_jni::jvmti_interface::breakpoint::{clear_breakpoint, set_breakpoint};
use crate::rust_jni::jvmti_interface::capabilities::{add_capabilities, get_capabilities, get_potential_capabilities};
use crate::rust_jni::jvmti_interface::classes::{get_class_loader, get_class_methods, get_class_signature, get_class_status, get_implemented_interfaces, get_loaded_classes, get_source_file_name};
use crate::rust_jni::jvmti_interface::event_callbacks::set_event_callbacks;
use crate::rust_jni::jvmti_interface::events::set_event_notification_mode;
use crate::rust_jni::jvmti_interface::field::{get_class_fields, get_field_modifiers, get_field_name, is_field_synthetic};
use crate::rust_jni::jvmti_interface::frame::{get_frame_count, get_frame_location, get_line_number_table, get_local_variable_table};
use crate::rust_jni::jvmti_interface::{get_class_modifiers, get_current_thread, get_field_declaring_class, get_max_locals, notify_frame_pop, set_local_double, set_local_float, set_local_int, set_local_long, set_local_object};
use crate::rust_jni::jvmti_interface::is::{is_array_class, is_interface};
use crate::rust_jni::jvmti_interface::locals::{get_local_double, get_local_float, get_local_int, get_local_long, get_local_object};
use crate::rust_jni::jvmti_interface::methods::{get_arguments_size, get_method_declaring_class, get_method_location, get_method_modifiers, get_method_name, is_method_native, is_method_obsolete, is_method_synthetic};
use crate::rust_jni::jvmti_interface::monitor::{create_raw_monitor, destroy_raw_monitor, raw_monitor_enter, raw_monitor_exit, raw_monitor_notify, raw_monitor_notify_all, raw_monitor_wait};
use crate::rust_jni::jvmti_interface::object::get_object_hash_code;
use crate::rust_jni::jvmti_interface::properties::get_system_property;
use crate::rust_jni::jvmti_interface::tags::{get_tag, set_tag};
use crate::rust_jni::jvmti_interface::thread_local_storage::{get_thread_local_storage, set_thread_local_storage};
use crate::rust_jni::jvmti_interface::threads::{get_all_threads, get_thread_info, get_thread_state};
use crate::rust_jni::jvmti_interface::threads::suspend_resume::{resume_thread, resume_thread_list, suspend_thread, suspend_thread_list};
use crate::rust_jni::jvmti_interface::threads::thread_groups::{get_thread_group_info, get_top_thread_groups};
use crate::rust_jni::jvmti_interface::version::get_version_number;

pub fn initial_jvmti() -> jvmtiInterface_1_ {
    jvmtiInterface_1_ {
        reserved1: null_mut(),
        SetEventNotificationMode: Some(set_event_notification_mode),
        reserved3: null_mut(),
        GetAllThreads: Some(get_all_threads),
        SuspendThread: Some(suspend_thread),
        ResumeThread: Some(resume_thread),
        StopThread: None,      //doesn't need impl not in currently supported capabilities
        InterruptThread: None, //doesn't need impl not in currently supported capabilities
        GetThreadInfo: Some(get_thread_info),
        GetOwnedMonitorInfo: None,        //doesn't need impl not in currently supported capabilities
        GetCurrentContendedMonitor: None, //doesn't need impl not in currently supported capabilities
        RunAgentThread: Some(run_agent_thread),
        GetTopThreadGroups: Some(get_top_thread_groups),
        GetThreadGroupInfo: Some(get_thread_group_info),
        GetThreadGroupChildren: None, //doesn't need impl not in currently supported capabilities
        GetFrameCount: Some(get_frame_count),
        GetThreadState: Some(get_thread_state),
        GetCurrentThread: Some(get_current_thread),
        GetFrameLocation: Some(get_frame_location),
        NotifyFramePop: Some(notify_frame_pop),
        GetLocalObject: Some(get_local_object),
        GetLocalInt: Some(get_local_int),
        GetLocalLong: Some(get_local_long),
        GetLocalFloat: Some(get_local_float),
        GetLocalDouble: Some(get_local_double),
        SetLocalObject: Some(set_local_object),
        SetLocalInt: Some(set_local_int),
        SetLocalLong: Some(set_local_long),
        SetLocalFloat: Some(set_local_float),
        SetLocalDouble: Some(set_local_double),
        CreateRawMonitor: Some(create_raw_monitor),
        DestroyRawMonitor: Some(destroy_raw_monitor),
        RawMonitorEnter: Some(raw_monitor_enter),
        RawMonitorExit: Some(raw_monitor_exit),
        RawMonitorWait: Some(raw_monitor_wait),
        RawMonitorNotify: Some(raw_monitor_notify),
        RawMonitorNotifyAll: Some(raw_monitor_notify_all),
        SetBreakpoint: Some(set_breakpoint),
        ClearBreakpoint: Some(clear_breakpoint),
        reserved40: null_mut(),
        SetFieldAccessWatch: None,         //doesn't need impl not in currently supported capabilities
        ClearFieldAccessWatch: None,       //doesn't need impl not in currently supported capabilities
        SetFieldModificationWatch: None,   //doesn't need impl not in currently supported capabilities
        ClearFieldModificationWatch: None, //doesn't need impl not in currently supported capabilities
        IsModifiableClass: None,           //doesn't need impl not in currently supported capabilities
        Allocate: Some(allocate),
        Deallocate: Some(deallocate),
        GetClassSignature: Some(get_class_signature),
        GetClassStatus: Some(get_class_status),
        GetSourceFileName: Some(get_source_file_name),
        GetClassModifiers: Some(get_class_modifiers),
        GetClassMethods: Some(get_class_methods),
        GetClassFields: Some(get_class_fields),
        GetImplementedInterfaces: Some(get_implemented_interfaces),
        IsInterface: Some(is_interface),
        IsArrayClass: Some(is_array_class),
        GetClassLoader: Some(get_class_loader),
        GetObjectHashCode: Some(get_object_hash_code),
        GetObjectMonitorUsage: None, //doesn't need impl not in currently supported capabilities
        GetFieldName: Some(get_field_name),
        GetFieldDeclaringClass: Some(get_field_declaring_class),
        GetFieldModifiers: Some(get_field_modifiers),
        IsFieldSynthetic: Some(is_field_synthetic),
        GetMethodName: Some(get_method_name),
        GetMethodDeclaringClass: Some(get_method_declaring_class),
        GetMethodModifiers: Some(get_method_modifiers),
        reserved67: null_mut(),
        GetMaxLocals: Some(get_max_locals),
        GetArgumentsSize: Some(get_arguments_size),
        GetLineNumberTable: Some(get_line_number_table),
        GetMethodLocation: Some(get_method_location),
        GetLocalVariableTable: Some(get_local_variable_table),
        SetNativeMethodPrefix: None,   //doesn't need impl not in currently supported capabilities
        SetNativeMethodPrefixes: None, //doesn't need impl not in currently supported capabilities
        GetBytecodes: None,            //doesn't need impl not in currently supported capabilities
        IsMethodNative: Some(is_method_native),
        IsMethodSynthetic: Some(is_method_synthetic),
        GetLoadedClasses: Some(get_loaded_classes),
        GetClassLoaderClasses: None,  //doesn't need impl not in currently supported capabilities
        PopFrame: None,               //todo impl. this is really blocking on a bunch of native stuff/jit
        ForceEarlyReturnObject: None, //doesn't need impl not in currently supported capabilities
        ForceEarlyReturnInt: None,    //doesn't need impl not in currently supported capabilities
        ForceEarlyReturnLong: None,   //doesn't need impl not in currently supported capabilities
        ForceEarlyReturnFloat: None,  //doesn't need impl not in currently supported capabilities
        ForceEarlyReturnDouble: None, //doesn't need impl not in currently supported capabilities
        ForceEarlyReturnVoid: None,   //doesn't need impl not in currently supported capabilities
        RedefineClasses: None,        //doesn't need impl not in currently supported capabilities
        GetVersionNumber: Some(get_version_number),
        GetCapabilities: Some(get_capabilities),
        GetSourceDebugExtension: None, //doesn't need impl not in currently supported capabilities
        IsMethodObsolete: Some(is_method_obsolete),
        SuspendThreadList: Some(suspend_thread_list),
        ResumeThreadList: Some(resume_thread_list),
        reserved94: null_mut(),
        reserved95: null_mut(),
        reserved96: null_mut(),
        reserved97: null_mut(),
        reserved98: null_mut(),
        reserved99: null_mut(),
        GetAllStackTraces: None,        //todo impl this needs to be atomic, so blocking on better thread story
        GetThreadListStackTraces: None, //todo impl
        GetThreadLocalStorage: Some(get_thread_local_storage),
        SetThreadLocalStorage: Some(set_thread_local_storage),
        GetStackTrace: None, //todo impl
        reserved105: null_mut(),
        GetTag: Some(get_tag),
        SetTag: Some(set_tag),
        ForceGarbageCollection: None,                //todo impl blocking on gc
        IterateOverObjectsReachableFromObject: None, //todo impl blocking on gc
        IterateOverReachableObjects: None,           //todo impl blocking on gc
        IterateOverHeap: None,                       //todo impl blocking on gc
        IterateOverInstancesOfClass: None,           //todo impl blocking on gc
        reserved113: null_mut(),
        GetObjectsWithTags: None, //todo impl
        FollowReferences: None,   //todo impl blocking on gc
        IterateThroughHeap: None, //todo impl blocking on gc
        reserved117: null_mut(),
        reserved118: null_mut(),
        reserved119: null_mut(),
        SetJNIFunctionTable: None, //todo impl
        GetJNIFunctionTable: None, //todo impl
        SetEventCallbacks: Some(set_event_callbacks),
        GenerateEvents: None,            //doesn't need impl not in currently supported capabilities
        GetExtensionFunctions: None,     //todo impl
        GetExtensionEvents: None,        //todo impl
        SetExtensionEventCallback: None, //todo impl
        DisposeEnvironment: Some(dispose_environment),
        GetErrorName: None,        //todo impl
        GetJLocationFormat: None,  //todo impl
        GetSystemProperties: None, //todo impl
        GetSystemProperty: Some(get_system_property),
        SetSystemProperty: None,            //todo impl
        GetPhase: None,                     //todo impl
        GetCurrentThreadCpuTimerInfo: None, //doesn't need impl not in currently supported capabilities
        GetCurrentThreadCpuTime: None,      //doesn't need impl not in currently supported capabilities
        GetThreadCpuTimerInfo: None,        //doesn't need impl not in currently supported capabilities
        GetThreadCpuTime: None,             //doesn't need impl not in currently supported capabilities
        GetTimerInfo: None,                 //todo impl
        GetTime: None,                      //todo impl
        GetPotentialCapabilities: Some(get_potential_capabilities),
        reserved141: null_mut(),
        AddCapabilities: Some(add_capabilities),
        RelinquishCapabilities: None,          //todo impl
        GetAvailableProcessors: None,          //todo impl
        GetClassVersionNumbers: None,          //todo impl
        GetConstantPool: None,                 //doesn't need impl not in currently supported capabilities
        GetEnvironmentLocalStorage: None,      //todo impl
        SetEnvironmentLocalStorage: None,      //todo impl
        AddToBootstrapClassLoaderSearch: None, //todo impl
        SetVerboseFlag: None,                  //todo impl
        AddToSystemClassLoaderSearch: None,    //todo impl
        RetransformClasses: None,              //doesn't need impl not in currently supported capabilities
        GetOwnedMonitorStackDepthInfo: None,   //doesn't need impl not in currently supported capabilities
        GetObjectSize: None,                   //todo impl
        GetLocalInstance: None,                //todo impl
    }
}

pub unsafe fn get_state<'gc, 'l>(env: *mut jvmtiEnv) -> &'l JVMState<'gc> {
    &*((**env).reserved1 as *const JVMState)
}

pub unsafe fn get_interpreter_state<'gc,'l, 'k>(env: *mut jvmtiEnv) -> &'k mut InterpreterStateGuard<'gc,'k> {
    let jvm = get_state(env);
    jvm.get_int_state()
}

pub fn get_jvmti_interface<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut JavaStackGuard<'gc>) -> *mut jvmtiEnv {
    let jvmti_interface = int_state.stack_jni_interface().jvmti_inner_mut();
    todo!()
}
