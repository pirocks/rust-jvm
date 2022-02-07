use std::mem::transmute;
use std::ptr::null_mut;

use jvmti_jni_bindings::*;
use rust_jvm_common::FieldId;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::get_thread_or_error;
use crate::interpreter_state::AddFrameNotifyError;
use crate::java_values::JavaValue;
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
use crate::rust_jni::interface::local_frame::new_local_ref_public;
use crate::rust_jni::native_util::from_jclass;
use crate::rust_jni::native_util::from_object;

pub mod event_callbacks;

//todo handle early return message here?
#[macro_export]
macro_rules! null_check {
    ($ptr: expr) => {
        if $ptr.is_null() {
            return crate::jvmti::jvmtiError_JVMTI_ERROR_NULL_POINTER;
        }
    };
}

pub unsafe fn get_state<'gc_life, 'l>(env: *mut jvmtiEnv) -> &'l JVMState<'gc_life> {
    &*((**env).reserved1 as *const JVMState)
}

pub unsafe fn get_interpreter_state<'gc_life,'l, 'k>(env: *mut jvmtiEnv) -> &'k mut InterpreterStateGuard<'gc_life,'k> {
    let jvm = get_state(env);
    jvm.get_int_state()
}

pub fn get_jvmti_interface(jvm: &'gc_life JVMState<'gc_life>, _int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) -> *mut jvmtiEnv {
    let new = get_jvmti_interface_impl(jvm);
    Box::leak(box (Box::leak(box new) as *const jvmtiInterface_1_)) as *mut jvmtiEnv
}

fn get_jvmti_interface_impl(jvm: &'gc_life JVMState<'gc_life>) -> jvmtiInterface_1_ {
    jvmtiInterface_1_ {
        reserved1: unsafe { transmute(jvm) },
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
        reserved40: std::ptr::null_mut(),
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
        reserved67: std::ptr::null_mut(),
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
        reserved94: std::ptr::null_mut(),
        reserved95: std::ptr::null_mut(),
        reserved96: std::ptr::null_mut(),
        reserved97: std::ptr::null_mut(),
        reserved98: std::ptr::null_mut(),
        reserved99: std::ptr::null_mut(),
        GetAllStackTraces: None,        //todo impl this needs to be atomic, so blocking on better thread story
        GetThreadListStackTraces: None, //todo impl
        GetThreadLocalStorage: Some(get_thread_local_storage),
        SetThreadLocalStorage: Some(set_thread_local_storage),
        GetStackTrace: None, //todo impl
        reserved105: std::ptr::null_mut(),
        GetTag: Some(get_tag),
        SetTag: Some(set_tag),
        ForceGarbageCollection: None,                //todo impl blocking on gc
        IterateOverObjectsReachableFromObject: None, //todo impl blocking on gc
        IterateOverReachableObjects: None,           //todo impl blocking on gc
        IterateOverHeap: None,                       //todo impl blocking on gc
        IterateOverInstancesOfClass: None,           //todo impl blocking on gc
        reserved113: std::ptr::null_mut(),
        GetObjectsWithTags: None, //todo impl
        FollowReferences: None,   //todo impl blocking on gc
        IterateThroughHeap: None, //todo impl blocking on gc
        reserved117: std::ptr::null_mut(),
        reserved118: std::ptr::null_mut(),
        reserved119: std::ptr::null_mut(),
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
        reserved141: std::ptr::null_mut(),
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

///Get Max Locals
//
//     jvmtiError
//     GetMaxLocals(jvmtiEnv* env,
//                 jmethodID method,
//                 jint* max_ptr)
//
// For the method indicated by method, return the number of local variable slots used by the method, including the local variables used to pass parameters to the method on its invocation.
//
// See max_locals in The Java™ Virtual Machine Specification, Chapter 4.7.3.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	68	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// method	jmethodID	The method to query.
// max_ptr	jint*	On return, points to the maximum number of local slots
//
// Agent passes a pointer to a jint. On return, the jint has been set.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_INVALID_METHODID	method is not a jmethodID.
// JVMTI_ERROR_NATIVE_METHOD	method is a native method.
// JVMTI_ERROR_NULL_POINTER	max_ptr is NULL.
unsafe extern "C" fn get_max_locals(env: *mut jvmtiEnv, method: jmethodID, max_ptr: *mut jint) -> jvmtiError {
    null_check!(max_ptr);
    let jvm = get_state(env);
    let (runtime_class, index) = match jvm.method_table.read().unwrap().try_lookup(method as usize) {
        None => return jvmtiError_JVMTI_ERROR_INVALID_METHODID,
        Some(method_id) => method_id,
    };
    let max_locals = match runtime_class.view().method_view_i(index).code_attribute() {
        None => return jvmtiError_JVMTI_ERROR_NATIVE_METHOD,
        Some(res) => res,
    }
        .max_locals;
    max_ptr.write(max_locals as i32);
    jvmtiError_JVMTI_ERROR_NONE
}

//Get Field Declaring Class
//
//     jvmtiError
//     GetFieldDeclaringClass(jvmtiEnv* env,
//                 jclass klass,
//                 jfieldID field,
//                 jclass* declaring_class_ptr)
//
// For the field indicated by klass and field return the class that defined it via declaring_class_ptr. The declaring class will either be klass, a superclass, or an implemented interface.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	61	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// klass	jclass	The class to query.
// field	jfieldID	The field to query.
// declaring_class_ptr	jclass*	On return, points to the declaring class
//
// Agent passes a pointer to a jclass. On return, the jclass has been set. The object returned by declaring_class_ptr is a JNI local reference and must be managed.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
// JVMTI_ERROR_INVALID_FIELDID	field is not a jfieldID.
// JVMTI_ERROR_NULL_POINTER	declaring_class_ptr is NULL.

unsafe extern "C" fn get_field_declaring_class(env: *mut jvmtiEnv, _klass: jclass, field: jfieldID, declaring_class_ptr: *mut jclass) -> jvmtiError {
    let jvm = get_state(env);
    null_check!(declaring_class_ptr);
    let field_id: FieldId = field as usize;
    let (runtime_class, index) = jvm.field_table.read().unwrap().lookup(field_id);
    let type_ = runtime_class.view().field(index as usize).field_type();
    let int_state = get_interpreter_state(env);
    let res_object = new_local_ref_public(
        match get_or_create_class_object(jvm, type_, int_state) {
            Ok(res) => res,
            Err(_) => return jvmtiError_JVMTI_ERROR_INTERNAL,
        }
            .into(),
        int_state,
    );
    declaring_class_ptr.write(res_object);
    return jvmtiError_JVMTI_ERROR_NONE;
}

///Get Class Modifiers
//
//     jvmtiError
//     GetClassModifiers(jvmtiEnv* env,
//                 jclass klass,
//                 jint* modifiers_ptr)
//
// For the class indicated by klass, return the access flags via modifiers_ptr. Access flags are defined in The Java™ Virtual Machine Specification, Chapter 4.
//
// If the class is an array class, then its public, private, and protected modifiers are the same as those of its component type. For arrays of primitives, this component type is represented by one of the primitive classes (for example, java.lang.Integer.TYPE).
//
// If the class is a primitive class, its public modifier is always true, and its protected and private modifiers are always false.
//
// If the class is an array class or a primitive class then its final modifier is always true and its interface modifier is always false. The values of its other modifiers are not determined by this specification.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	51	1.0
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// klass	jclass	The class to query.
// modifiers_ptr	jint*	On return, points to the current access flags of this class.
//
// Agent passes a pointer to a jint. On return, the jint has been set.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_INVALID_CLASS	klass is not a class object or the class has been unloaded.
// JVMTI_ERROR_NULL_POINTER	modifiers_ptr is NULL.
unsafe extern "C" fn get_class_modifiers(env: *mut jvmtiEnv, klass: jclass, modifiers_ptr: *mut jint) -> jvmtiError {
    let jvm = get_state(env);
    null_check!(modifiers_ptr);
    //handle klass invalid
    let runtime_class = from_jclass(jvm, klass).as_runtime_class(jvm);
    modifiers_ptr.write(runtime_class.view().access_flags() as u32 as i32);
    jvmtiError_JVMTI_ERROR_NONE
}

unsafe extern "C" fn set_local_object(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jobject) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Object(todo!() /*from_jclass(jvm,value)*/))
}

unsafe extern "C" fn set_local_int(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jint) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Int(value))
}

unsafe extern "C" fn set_local_long(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jlong) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Long(value))
}

unsafe extern "C" fn set_local_double(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jdouble) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Double(value))
}

unsafe extern "C" fn set_local_float(env: *mut jvmtiEnv, thread: jthread, depth: jint, slot: jint, value: jfloat) -> jvmtiError {
    set_local(env, thread, depth, slot, JavaValue::Float(value))
}

///Notify Frame Pop
//
//     jvmtiError
//     NotifyFramePop(jvmtiEnv* env,
//                 jthread thread,
//                 jint depth)
//
// When the frame that is currently at depth is popped from the stack, generate a FramePop event. See the FramePop event for details. Only frames corresponding to non-native Java programming language methods can receive notification.
//
// The specified thread must either be the current thread or the thread must be suspended.
//
// Phase	Callback Safe	Position	Since
// may only be called during the live phase 	No 	20	1.0
//
// Capabilities
// Optional Functionality: might not be implemented for all virtual machines. The following capability (as returned by GetCapabilities) must be true to use this function.
// Capability 	Effect
// can_generate_frame_pop_events	Can set and thus get FramePop events
//
// Parameters
// Name 	Type 	Description
// thread	jthread	The thread of the frame for which the frame pop event will be generated. If thread is NULL, the current thread is used.
// depth	jint	The depth of the frame for which the frame pop event will be generated.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_MUST_POSSESS_CAPABILITY 	The environment does not possess the capability can_generate_frame_pop_events. Use AddCapabilities.
// JVMTI_ERROR_OPAQUE_FRAME	The frame at depth is executing a native method.
// JVMTI_ERROR_THREAD_NOT_SUSPENDED	Thread was not suspended and was not the current thread.
// JVMTI_ERROR_INVALID_THREAD	thread is not a thread object.
// JVMTI_ERROR_THREAD_NOT_ALIVE	thread is not live (has not been started or is now dead).
// JVMTI_ERROR_ILLEGAL_ARGUMENT	depth is less than zero.
// JVMTI_ERROR_NO_MORE_FRAMES	There are no stack frames at the specified depth.

unsafe extern "C" fn notify_frame_pop(env: *mut jvmtiEnv, thread: jthread, depth: jint) -> jvmtiError {
    let jvm = get_state(env);
    //todo check capability
    let java_thread = get_thread_or_error!(jvm, thread).get_java_thread(jvm);
    let action = |int_state: &mut InterpreterStateGuard| {
        //todo check thread opaque
        match int_state.add_should_frame_pop_notify(depth as usize) {
            Ok(_) => jvmtiError_JVMTI_ERROR_NONE,
            Err(err) => match err {
                AddFrameNotifyError::Opaque => jvmtiError_JVMTI_ERROR_OPAQUE_FRAME,
                AddFrameNotifyError::NothingAtDepth => jvmtiError_JVMTI_ERROR_NO_MORE_FRAMES,
            },
        }
    };

    if java_thread.is_this_thread() {
        action(get_interpreter_state(env))
    } else {
        if todo!() {
            return jvmtiError_JVMTI_ERROR_THREAD_SUSPENDED;
        }
        //todo check thread suspended
        let mut int_state_not_ref = InterpreterStateGuard::RemoteInterpreterState {
            int_state: todo!(),
            thread: java_thread,
            registered: false,
            jvm
        };
        action(&mut int_state_not_ref)
    }
}

///Get Current Thread
//
//     jvmtiError
//     GetCurrentThread(jvmtiEnv* env,
//                 jthread* thread_ptr)
//
// Get the current thread. The current thread is the Java programming language thread which has called the function.
//
// Note that most JVM TI functions that take a thread as an argument will accept NULL to mean the current thread.
//
// Phase	Callback Safe	Position	Since
// may only be called during the start or the live phase 	No 	18	1.1
//
// Capabilities
// Required Functionality
//
// Parameters
// Name 	Type 	Description
// thread_ptr	jthread*	On return, points to the current thread.
//
// Agent passes a pointer to a jthread. On return, the jthread has been set. The object returned by thread_ptr is a JNI local reference and must be managed.
//
// Errors
// This function returns either a universal error or one of the following errors
// Error 	Description
// JVMTI_ERROR_NULL_POINTER	thread_ptr is NULL.
unsafe extern "C" fn get_current_thread(env: *mut jvmtiEnv, thread_ptr: *mut jthread) -> jvmtiError {
    let jvm = get_state(env);
    let int_state = get_interpreter_state(env);
    null_check!(thread_ptr);
    let current_thread = jvm.thread_state.get_current_thread();
    thread_ptr.write(new_local_ref_public(current_thread.thread_object().object().to_gc_managed().into(), int_state));
    jvmtiError_JVMTI_ERROR_NONE
}

///Universal Errors
// The following errors may be returned by any function
//
// JVMTI_ERROR_NONE (0)
//     No error has occurred. This is the error code that is returned on successful completion of the function.
//
// JVMTI_ERROR_NULL_POINTER (100)
//     Pointer is unexpectedly NULL.
//
// JVMTI_ERROR_OUT_OF_MEMORY (110)
//     The function attempted to allocate memory and no more memory was available for allocation.
//
// JVMTI_ERROR_ACCESS_DENIED (111)
//     The desired functionality has not been enabled in this virtual machine.
//
// JVMTI_ERROR_UNATTACHED_THREAD (115)
//     The thread being used to call this function is not attached to the virtual machine. Calls must be made from attached threads. See AttachCurrentThread in the JNI invocation API.
//
// JVMTI_ERROR_INVALID_ENVIRONMENT (116)
//     The JVM TI environment provided is no longer connected or is not an environment.
//
// JVMTI_ERROR_WRONG_PHASE (112)
//     The desired functionality is not available in the current phase. Always returned if the virtual machine has completed running.
//
// JVMTI_ERROR_INTERNAL (113)
//     An unexpected internal error has occurred.
pub fn universal_error() -> jvmtiError {
    jvmtiError_JVMTI_ERROR_INTERNAL
    //todo make this better
}

pub mod breakpoint;
pub mod is;
pub mod methods;
pub mod object;
#[macro_use]
pub mod threads;
#[macro_use]
pub mod frame;
#[macro_use]
pub mod thread_local_storage;
pub mod agent;
pub mod allocate;
pub mod capabilities;
pub mod classes;
pub mod events;
pub mod field;
pub mod locals;
pub mod monitor;
pub mod properties;
pub mod tags;
pub mod version;