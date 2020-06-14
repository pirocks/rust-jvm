use std::cell::RefCell;
use std::ffi::CString;
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::ptr::null_mut;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::*;
use rust_jvm_common::classnames::ClassName;

use crate::class_objects::get_or_create_class_object;
use crate::java_values::JavaValue;
use crate::JVMState;
use crate::jvmti::agent::*;
use crate::jvmti::allocate::{allocate, deallocate};
use crate::jvmti::breakpoint::*;
use crate::jvmti::capabilities::{add_capabilities, get_capabilities, get_potential_capabilities};
use crate::jvmti::classes::*;
use crate::jvmti::event_callbacks::set_event_callbacks;
use crate::jvmti::events::set_event_notification_mode;
use crate::jvmti::frame::{get_frame_count, get_frame_location};
use crate::jvmti::is::{is_array_class, is_interface, is_method_obsolete, is_method_native};
use crate::jvmti::monitor::{create_raw_monitor, destroy_raw_monitor, raw_monitor_enter, raw_monitor_exit, raw_monitor_notify, raw_monitor_notify_all, raw_monitor_wait};
use crate::jvmti::properties::get_system_property;
use crate::jvmti::tags::*;
use crate::jvmti::thread_local_storage::*;
use crate::jvmti::threads::{get_all_threads, get_thread_group_info, get_thread_info, get_thread_state, get_top_thread_groups, interrupt_thread, resume_thread_list, suspend_thread, suspend_thread_list};
use crate::jvmti::version::get_version_number;
use crate::method_table::MethodId;
use crate::rust_jni::interface::get_field::new_field_id;
use crate::rust_jni::native_util::{from_jclass, from_object, to_object};

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


pub unsafe extern "C" fn get_method_declaring_class(env: *mut jvmtiEnv, method: jmethodID, declaring_class_ptr: *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodDeclaringClass");
    let method_id: MethodId = transmute(method);
    let runtime_class = jvm.method_table.read().unwrap().lookup(method_id).0;
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
    let method_id: MethodId = transmute(method);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    match class.view().method_view_i(method_i as usize).code_attribute() {
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
    is_synthetic_ptr: *mut jboolean,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "IsMethodSynthetic");
    let method_id: MethodId = transmute(method);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let synthetic = class.view().method_view_i(method_i as usize).is_synthetic();
    is_synthetic_ptr.write(synthetic as u8);
    jvm.tracing.trace_jdwp_function_exit(jvm, "IsMethodSynthetic");
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn get_method_modifiers(
    env: *mut jvmtiEnv,
    method: jmethodID,
    modifiers_ptr: *mut jint,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodModifiers");
    let method_id: MethodId = transmute(method);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let modifiers = class.view().method_view_i(method_i as usize).access_flags();
    modifiers_ptr.write(modifiers as jint);
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetMethodModifiers");
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn get_method_name(env: *mut jvmtiEnv, method: jmethodID,
                                     name_ptr: *mut *mut ::std::os::raw::c_char,
                                     signature_ptr: *mut *mut ::std::os::raw::c_char,
                                     generic_ptr: *mut *mut ::std::os::raw::c_char,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetMethodName");
    let method_id: MethodId = transmute(method);
    let (class, method_i) = jvm.method_table.read().unwrap().lookup(method_id);
    let mv = class.view().method_view_i(method_i as usize);
    let name = mv.name();
    let desc_str = mv.desc_str();
    if generic_ptr != null_mut() {
        // unimplemented!()//todo figure out what this is
    }
    if signature_ptr != null_mut() {
        signature_ptr.write(CString::new(desc_str).unwrap().into_raw())
    }
    if name_ptr != null_mut() {
        name_ptr.write(CString::new(name).unwrap().into_raw())
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetMethodName");
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn get_source_file_name(
    env: *mut jvmtiEnv,
    klass: jclass,
    source_name_ptr: *mut *mut ::std::os::raw::c_char,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetSourceFileName");
    let class_obj = from_jclass(klass);
    let runtime_class = class_obj.as_runtime_class();
    let class_view = runtime_class.view();
    source_name_ptr.write(CString::new(class_view.sourcefile_attr().file()).unwrap().into_raw());
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetSourceFileName");
    jvmtiError_JVMTI_ERROR_NONE
}


unsafe extern "C" fn get_class_fields(
    env: *mut jvmtiEnv,
    klass: jclass,
    field_count_ptr: *mut jint,
    fields_ptr: *mut *mut jfieldID,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetClassFields");
    let class_obj = from_jclass(klass);
    let runtime_class = class_obj.as_runtime_class();
    let class_view = runtime_class.view();
    let num_fields = class_view.num_fields();
    field_count_ptr.write(num_fields as jint);
    fields_ptr.write(libc::calloc(num_fields, size_of::<*mut jfieldID>()) as *mut *mut jvmti_jni_bindings::_jfieldID);
    for i in 0..num_fields {
        fields_ptr.read().offset(i as isize).write(new_field_id(jvm, runtime_class.clone(), i))
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetClassFields");
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn get_implemented_interfaces(
    env: *mut jvmtiEnv,
    klass: jclass,
    interface_count_ptr: *mut jint,
    interfaces_ptr: *mut *mut jclass,
) -> jvmtiError {
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "GetImplementedInterfaces");
    let class_obj = from_jclass(klass);
    let runtime_class = class_obj.as_runtime_class();
    let class_view = runtime_class.view();
    let num_interfaces = class_view.num_interfaces();
    interface_count_ptr.write(num_interfaces as i32);
    interfaces_ptr.write(libc::calloc(num_interfaces, size_of::<*mut jclass>()) as *mut jclass);
    for (i, interface) in class_view.interfaces().enumerate() {
        let interface_obj = get_or_create_class_object(
            jvm,
            &ClassName::Str(interface.interface_name()).into(),
            jvm.get_current_frame().deref(),
            runtime_class.loader(jvm).clone(),
        );
        let interface_class = to_object(interface_obj.into());
        interfaces_ptr.read().offset(i as isize).write(interface_class)
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "GetImplementedInterfaces");
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