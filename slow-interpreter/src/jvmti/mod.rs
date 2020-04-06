use jvmti_bindings::{jvmtiInterface_1_, JavaVM, jint, JNIInvokeInterface_, jvmtiError, jvmtiEnv};
use std::rc::Rc;
use std::intrinsics::transmute;
use std::os::raw::{c_void, c_char};
use libloading::Library;
use std::ops::Deref;
use crate::{InterpreterState, StackEntry};
use crate::rust_jni::interface::get_interface;
use std::ffi::CString;
use crate::invoke_interface::get_invoke_interface;
use crate::jvmti::version::get_version_number;
use crate::jvmti::properties::get_system_property;
use crate::jvmti::allocate::allocate;

pub struct LibJDWP{
    lib : Library
}


impl LibJDWP{
    pub fn agent_load(&self, state: &mut InterpreterState, frame: Rc<StackEntry>) -> jvmtiError {
        unsafe {
            let agent_load_symbol = self.lib.get::<fn(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) -> jint>("Agent_OnLoad".as_bytes()).unwrap();
            let agent_load_fn_ptr = agent_load_symbol.deref();
            let args = CString::new("transport=dt_socket,server=y,suspend=n,address=5005").unwrap().into_raw();//todo parse these at jvm startup
            let interface: JNIInvokeInterface_ = get_invoke_interface(state,frame);
            agent_load_fn_ptr(&mut (&interface as *const JNIInvokeInterface_) as *mut *const JNIInvokeInterface_, args, std::ptr::null_mut()) as jvmtiError
        }
    }
}

pub fn load_libjdwp(jdwp_path : &str) -> LibJDWP{
    LibJDWP{
        lib: Library::new(jdwp_path).unwrap()
    }
}

pub fn get_jvmti_interface(state : &mut InterpreterState, frame : Rc<StackEntry>) -> jvmtiEnv{
    Box::leak(jvmtiInterface_1_ {
        reserved1: unsafe {transmute(state)},
        SetEventNotificationMode: None,
        reserved3: unsafe {
            let boxed = Box::new(frame);
            Box::into_raw(boxed) as *mut c_void//todo leak?
        },
        GetAllThreads: None,
        SuspendThread: None,
        ResumeThread: None,
        StopThread: None,
        InterruptThread: None,
        GetThreadInfo: None,
        GetOwnedMonitorInfo: None,
        GetCurrentContendedMonitor: None,
        RunAgentThread: None,
        GetTopThreadGroups: None,
        GetThreadGroupInfo: None,
        GetThreadGroupChildren: None,
        GetFrameCount: None,
        GetThreadState: None,
        GetCurrentThread: None,
        GetFrameLocation: None,
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
        CreateRawMonitor: None,
        DestroyRawMonitor: None,
        RawMonitorEnter: None,
        RawMonitorExit: None,
        RawMonitorWait: None,
        RawMonitorNotify: None,
        RawMonitorNotifyAll: None,
        SetBreakpoint: None,
        ClearBreakpoint: None,
        reserved40: std::ptr::null_mut(),
        SetFieldAccessWatch: None,
        ClearFieldAccessWatch: None,
        SetFieldModificationWatch: None,
        ClearFieldModificationWatch: None,
        IsModifiableClass: None,
        Allocate: Some(allocate),
        Deallocate: None,
        GetClassSignature: None,
        GetClassStatus: None,
        GetSourceFileName: None,
        GetClassModifiers: None,
        GetClassMethods: None,
        GetClassFields: None,
        GetImplementedInterfaces: None,
        IsInterface: None,
        IsArrayClass: None,
        GetClassLoader: None,
        GetObjectHashCode: None,
        GetObjectMonitorUsage: None,
        GetFieldName: None,
        GetFieldDeclaringClass: None,
        GetFieldModifiers: None,
        IsFieldSynthetic: None,
        GetMethodName: None,
        GetMethodDeclaringClass: None,
        GetMethodModifiers: None,
        reserved67: std::ptr::null_mut(),
        GetMaxLocals: None,
        GetArgumentsSize: None,
        GetLineNumberTable: None,
        GetMethodLocation: None,
        GetLocalVariableTable: None,
        SetNativeMethodPrefix: None,
        SetNativeMethodPrefixes: None,
        GetBytecodes: None,
        IsMethodNative: None,
        IsMethodSynthetic: None,
        GetLoadedClasses: None,
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
        GetCapabilities: None,
        GetSourceDebugExtension: None,
        IsMethodObsolete: None,
        SuspendThreadList: None,
        ResumeThreadList: None,
        reserved94: std::ptr::null_mut(),
        reserved95: std::ptr::null_mut(),
        reserved96: std::ptr::null_mut(),
        reserved97: std::ptr::null_mut(),
        reserved98: std::ptr::null_mut(),
        reserved99: std::ptr::null_mut(),
        GetAllStackTraces: None,
        GetThreadListStackTraces: None,
        GetThreadLocalStorage: None,
        SetThreadLocalStorage: None,
        GetStackTrace: None,
        reserved105: std::ptr::null_mut(),
        GetTag: None,
        SetTag: None,
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
        SetEventCallbacks: None,
        GenerateEvents: None,
        GetExtensionFunctions: None,
        GetExtensionEvents: None,
        SetExtensionEventCallback: None,
        DisposeEnvironment: None,
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
        GetPotentialCapabilities: Some(capabilities::get_potential_capabilities),
        reserved141: std::ptr::null_mut(),
        AddCapabilities: Some(capabilities::add_capabilities),
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
        GetLocalInstance: None
    }.into()) as jvmtiEnv
}

pub mod capabilities;
pub mod version;
pub mod properties;
pub mod allocate;