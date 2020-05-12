use jvmti_jni_bindings::*;
use std::intrinsics::transmute;
use std::ops::Deref;
use crate::{JVMState};
use crate::jvmti::version::get_version_number;
use crate::jvmti::properties::get_system_property;
use crate::jvmti::allocate::{allocate, deallocate};
use crate::jvmti::capabilities::{add_capabilities, get_potential_capabilities, get_capabilities};
use crate::jvmti::events::set_event_notification_mode;
use std::cell::RefCell;
use crate::jvmti::monitor::{create_raw_monitor, raw_monitor_enter, raw_monitor_exit, raw_monitor_wait, raw_monitor_notify_all, raw_monitor_notify};
use crate::jvmti::threads::{get_top_thread_groups, get_all_threads, get_thread_info, suspend_thread_list, suspend_thread, resume_thread_list, get_thread_state, get_thread_group_info};
use crate::rust_jni::MethodId;
use crate::rust_jni::native_util::{to_object, from_object};
use crate::jvmti::thread_local_storage::*;
use crate::jvmti::tags::*;
use crate::jvmti::agent::*;
use crate::jvmti::classes::*;
use crate::class_objects::get_or_create_class_object;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java_values::JavaValue;
use crate::jvmti::is::{is_interface, is_array_class, is_method_obsolete};
use crate::jvmti::frame::{get_frame_count, get_frame_location};
use crate::jvmti::breakpoint::set_breakpoint;
use crate::jvmti::event_callbacks::set_event_callbacks;
use classfile_view::view::HasAccessFlags;

pub mod event_callbacks;


pub unsafe fn get_state<'l>(env: *mut jvmtiEnv) -> &'l JVMState {
    transmute((**env).reserved1)
}

thread_local! {
    static JVMTI_INTERFACE: RefCell<Option<jvmtiInterface_1_>> = RefCell::new(None);
}

pub fn get_jvmti_interface(jvm: &JVMState) -> jvmtiEnv {
    JVMTI_INTERFACE.with(|refcell| {
        {
            let first_borrow = refcell.borrow();
            match first_borrow.as_ref() {
                None => {}
                Some(interface) => {
                    return interface as jvmtiEnv;
                }
            }
        }
        let new = get_jvmti_interface_impl(jvm);
        refcell.replace(new.into());
        let new_borrow = refcell.borrow();
        new_borrow.as_ref().unwrap() as jvmtiEnv
    })
}

fn get_jvmti_interface_impl(jvm: &JVMState) -> jvmtiInterface_1_ {
    jvmtiInterface_1_ {
        reserved1: unsafe { transmute(jvm) },
        SetEventNotificationMode: Some(set_event_notification_mode),
        reserved3: std::ptr::null_mut(),
        GetAllThreads: Some(get_all_threads),
        SuspendThread: Some(suspend_thread),
        ResumeThread: None,
        StopThread: None,
        InterruptThread: None,
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
        GetLocalObject: None,
        GetLocalInt: None,
        GetLocalLong: None,
        GetLocalFloat: None,
        GetLocalDouble: None,
        SetLocalObject: None,
        SetLocalInt: None,
        SetLocalLong: None,
        SetLocalFloat: None,
        SetLocalDouble: None,
        CreateRawMonitor: Some(create_raw_monitor),
        DestroyRawMonitor: None,
        RawMonitorEnter: Some(raw_monitor_enter),
        RawMonitorExit: Some(raw_monitor_exit),
        RawMonitorWait: Some(raw_monitor_wait),
        RawMonitorNotify: Some(raw_monitor_notify),
        RawMonitorNotifyAll: Some(raw_monitor_notify_all),
        SetBreakpoint: Some(set_breakpoint),
        ClearBreakpoint: None,
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
        GetSourceFileName: None,
        GetClassModifiers: None,
        GetClassMethods: Some(get_class_methods),
        GetClassFields: None,
        GetImplementedInterfaces: None,
        IsInterface: Some(is_interface),
        IsArrayClass: Some(is_array_class),
        GetClassLoader: None,
        GetObjectHashCode: Some(get_object_hash_code),
        GetObjectMonitorUsage: None,
        GetFieldName: None,
        GetFieldDeclaringClass: None,
        GetFieldModifiers: None,
        IsFieldSynthetic: None,
        GetMethodName: Some(get_method_name),
        GetMethodDeclaringClass: Some(get_method_declaring_class),
        GetMethodModifiers: Some(get_method_modifiers),
        reserved67: std::ptr::null_mut(),
        GetMaxLocals: None,
        GetArgumentsSize: None,
        GetLineNumberTable: None,
        GetMethodLocation: Some(get_method_location),
        GetLocalVariableTable: None,
        SetNativeMethodPrefix: None,
        SetNativeMethodPrefixes: None,
        GetBytecodes: None,
        IsMethodNative: None,
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

pub unsafe extern "C" fn get_method_declaring_class(env: *mut jvmtiEnv, method: jmethodID, declaring_class_ptr: *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodDeclaringClass");
    let runtime_class = (&*(method as *const MethodId)).class.clone();
    let class_object = get_or_create_class_object(
        jvm,
        &PTypeView::Ref(ReferenceTypeView::Class(runtime_class.view().name())),
        jvm.get_current_frame().deref(),
        runtime_class.loader(jvm).clone(),
    );//todo fix this type verbosity thing
    declaring_class_ptr.write(transmute(to_object(class_object.into())));
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetMethodDeclaringClass");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_object_hash_code(env: *mut jvmtiEnv, object: jobject, hash_code_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetObjectHashCode");
    let object = JavaValue::Object(from_object(transmute(object))).cast_object();
    let res = object.hash_code(jvm, jvm.get_current_frame().deref());
    hash_code_ptr.write(res);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetObjectHashCode");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn get_method_location(env: *mut jvmtiEnv, method: jmethodID, start_location_ptr: *mut jlocation, end_location_ptr: *mut jlocation) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodLocation");
    let method_id = (method as *mut MethodId).as_ref().unwrap();
    match method_id.class.view().method_view_i(method_id.method_i).code_attribute() {
        None => {
            start_location_ptr.write(-1);
            end_location_ptr.write(-1);
        }
        Some(code) => {
            start_location_ptr.write(0);
            end_location_ptr.write((code.code.len() - 1) as i64);
        }
    };
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodLocation");
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn dispose_environment(env: *mut jvmtiEnv) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "DisposeEnvironment");
    jvm.tracing.trace_jdwp_function_exit(jvm, "DisposeEnvironment");
    jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY
}

pub unsafe extern "C" fn is_method_synthetic(
    env: *mut jvmtiEnv,
    method: jmethodID,
    is_synthetic_ptr: *mut jboolean
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "IsMethodSynthetic");
    let method_id = &*(method as *const MethodId);
    let synthetic = method_id.class.view().method_view_i(method_id.method_i).is_synthetic();
    is_synthetic_ptr.write(synthetic as u8);
    jvm.tracing.trace_jdwp_function_exit(jvm, "IsMethodSynthetic");
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn get_method_modifiers(
env: *mut jvmtiEnv,
method: jmethodID,
modifiers_ptr: *mut jint,
) -> jvmtiError{
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodModifiers");
    let method_id = &*(method as *const MethodId);
    let modifiers = method_id.class.view().method_view_i(method_id.method_i).access_flags();
    modifiers_ptr.write(modifiers as jint);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetMethodModifiers");
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn get_method_name(env: *mut jvmtiEnv, method: jmethodID,
name_ptr: *mut *mut ::std::os::raw::c_char,
signature_ptr: *mut *mut ::std::os::raw::c_char,
generic_ptr: *mut *mut ::std::os::raw::c_char,
) -> jvmtiError{
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodName");
    unimplemented!()
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetMethodName");
    jvmtiError_JVMTI_ERROR_NONE
}


pub mod is;
pub mod breakpoint;
pub mod frame;
pub mod thread_local_storage;
pub mod agent;
pub mod classes;
pub mod tags;
pub mod threads;
pub mod monitor;
pub mod capabilities;
pub mod version;
pub mod properties;
pub mod allocate;
pub mod events;