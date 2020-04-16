use jvmti_bindings::*;
use std::intrinsics::transmute;
use std::os::raw::{c_void, c_char};
use libloading::Library;
use std::ops::Deref;
use crate::{JVMState, JavaThread};
use std::ffi::CString;
use crate::invoke_interface::get_invoke_interface;
use crate::jvmti::version::get_version_number;
use crate::jvmti::properties::get_system_property;
use crate::jvmti::allocate::{allocate, deallocate};
use crate::jvmti::capabilities::{add_capabilities, get_potential_capabilities};
use crate::jvmti::events::{set_event_notification_mode, set_event_callbacks};
use std::sync::{Arc, RwLock};
use std::cell::RefCell;
use crate::rust_jni::interface::get_interface;
use crate::jvmti::monitor::{create_raw_monitor, raw_monitor_enter, raw_monitor_exit, raw_monitor_wait, raw_monitor_notify_all, raw_monitor_notify};
use crate::jvmti::threads::{get_top_thread_groups, get_all_threads};
use crate::rust_jni::MethodId;
use crate::rust_jni::native_util::to_object;
use crate::jvmti::thread_local_storage::*;
use crate::jvmti::tags::*;
use crate::jvmti::agent::*;
use crate::jvmti::classes::*;
use std::collections::HashSet;
use std::iter::FromIterator;

pub struct SharedLibJVMTI {
    lib: Arc<Library>,
    vm_init_callback: RwLock<jvmtiEventVMInit>,
    vm_init_enabled: RwLock<bool>,
    vm_death_callback: RwLock<jvmtiEventVMDeath>,
    vm_death_enabled: RwLock<bool>,
    exception_callback: RwLock<jvmtiEventException>,
    exception_enabled: RwLock<bool>,
    thread_start_callback: RwLock<jvmtiEventThreadStart>,
    thread_start_enabled: RwLock<bool>,
    thread_end_callback: RwLock<jvmtiEventThreadEnd>,
    thread_end_enabled: RwLock<bool>,
    class_prepare_callback: RwLock<jvmtiEventClassPrepare>,
    class_prepare_enabled: RwLock<bool>,
    garbage_collection_finish_callback: RwLock<jvmtiEventGarbageCollectionFinish>,
    garbage_collection_finish_enabled: RwLock<bool>,
    breakpoint_callback: RwLock<jvmtiEventBreakpoint>,
    breakpoint_enabled: RwLock<bool>,

    class_load_callback: RwLock<jvmtiEventClassLoad>,

    exception_catch_callback: RwLock<jvmtiEventExceptionCatch>,
    single_step_callback: RwLock<jvmtiEventSingleStep>,
    frame_pop_callback: RwLock<jvmtiEventFramePop>,
    field_access_callback: RwLock<jvmtiEventFieldAccess>,
    field_modification_callback: RwLock<jvmtiEventFieldModification>,
    method_entry_callback: RwLock<jvmtiEventMethodEntry>,
    method_exit_callback: RwLock<jvmtiEventMethodExit>,
    monitor_wait_callback: RwLock<jvmtiEventMonitorWait>,
    monitor_waited_callback: RwLock<jvmtiEventMonitorWaited>,
    monitor_conteded_enter_callback: RwLock<jvmtiEventMonitorContendedEnter>,
    monitor_conteded_entered_callback: RwLock<jvmtiEventMonitorContendedEntered>,
}

impl SharedLibJVMTI {
    pub fn vm_inited(&self, state: &JVMState) {
        unsafe {
            let interface = get_interface(state);
            let mut jvmti_interface = get_jvmti_interface(state);
            let mut casted_jni = interface as *const jni_bindings::JNINativeInterface_ as *const libc::c_void as *const JNINativeInterface_;
            let main_thread_guard = state.thread_state.main_thread.read().unwrap();
            let main_thread_nonnull = main_thread_guard.as_ref().unwrap();
            let thread_object_borrow = main_thread_nonnull.thread_object.borrow();
            let main_thread_object = thread_object_borrow.as_ref().unwrap().clone().object();
            self.VMInit(&mut jvmti_interface, &mut casted_jni, transmute(to_object(main_thread_object.into())))
        }
    }
}

#[allow(non_snake_case)]
pub trait DebuggerEventConsumer {
    unsafe fn VMInit(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn VMInit_enable(&self);
    fn VMInit_disable(&self);

    unsafe fn VMDeath(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv);

    fn VMDeath_enable(&self);
    fn VMDeath_disable(&self);

    //unsafe extern "C" fn(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation, exception: jobject, catch_method: jmethodID, catch_location: jlocation)
    unsafe fn Exception(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation, exception: jobject, catch_method: jmethodID, catch_location: jlocation);

    fn Exception_enable(&self);
    fn Exception_disable(&self);

    unsafe fn ThreadStart(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadStart_enable(&self);
    fn ThreadStart_disable(&self);


    unsafe fn ThreadEnd(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadEnd_enable(&self);
    fn ThreadEnd_disable(&self);

    unsafe fn ClassPrepare(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, klass: jclass);

    fn ClassPrepare_enable(&self);
    fn ClassPrepare_disable(&self);


    unsafe fn GarbageCollectionFinish(jvmti_env: *mut jvmtiEnv);
    fn GarbageCollectionFinish_enable(&self);
    fn GarbageCollectionFinish_disable(&self);


    unsafe fn Breakpoint(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation);
    fn Breakpoint_enable(&self);
    fn Breakpoint_disable(&self);
}

#[allow(non_snake_case)]
impl DebuggerEventConsumer for SharedLibJVMTI {
    unsafe fn VMInit(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject) {
        if *self.vm_init_enabled.read().unwrap() {
            let guard = self.vm_init_callback.read().unwrap();
            let f_pointer = *guard.as_ref().unwrap();
            std::mem::drop(guard);
            f_pointer(jvmti_env, jni_env, thread);
        }
    }

    fn VMInit_enable(&self) {
        *self.vm_init_enabled.write().unwrap() = true;
    }

    fn VMInit_disable(&self) {
        *self.vm_init_enabled.write().unwrap() = false;
    }

    unsafe fn VMDeath(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_) {
        if *self.vm_death_enabled.read().unwrap() {
            (self.vm_death_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env);
        }
    }

    fn VMDeath_enable(&self) {
        *self.vm_death_enabled.write().unwrap() = true;
    }

    fn VMDeath_disable(&self) {
        *self.vm_death_enabled.write().unwrap() = false;
    }

    unsafe fn Exception(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject, method: *mut _jmethodID, location: i64, exception: *mut _jobject, catch_method: *mut _jmethodID, catch_location: i64) {
        if *self.exception_enabled.read().unwrap() {
            (self.exception_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location, exception, catch_method, catch_location);
        }
    }

    fn Exception_enable(&self) {
        *self.exception_enabled.write().unwrap() = true;
    }

    fn Exception_disable(&self) {
        *self.exception_enabled.write().unwrap() = false;
    }

    unsafe fn ThreadStart(jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject) {
        unimplemented!()
    }

    fn ThreadStart_enable(&self) {
        *self.thread_start_enabled.write().unwrap() = true;
    }

    fn ThreadStart_disable(&self) {
        *self.thread_start_enabled.write().unwrap() = false;
    }

    unsafe fn ThreadEnd(jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: jthread) {
        unimplemented!()
    }

    fn ThreadEnd_enable(&self) {
        *self.thread_start_enabled.write().unwrap() = true;
    }

    fn ThreadEnd_disable(&self) {
        *self.thread_start_enabled.write().unwrap() = false;
    }

    unsafe fn ClassPrepare(jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject, klass: *mut _jobject) {
        unimplemented!()
    }

    fn ClassPrepare_enable(&self) {
        *self.class_prepare_enabled.write().unwrap() = true;
    }

    fn ClassPrepare_disable(&self) {
        *self.class_prepare_enabled.write().unwrap() = false;
    }

    unsafe fn GarbageCollectionFinish(jvmti_env: *mut *const jvmtiInterface_1_) {
        unimplemented!()
    }

    fn GarbageCollectionFinish_enable(&self) {
        *self.garbage_collection_finish_enabled.write().unwrap() = true;
    }

    fn GarbageCollectionFinish_disable(&self) {
        *self.garbage_collection_finish_enabled.write().unwrap() = false;
    }

    unsafe fn Breakpoint(jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject, method: *mut _jmethodID, location: i64) {
        unimplemented!()
    }

    fn Breakpoint_enable(&self) {
        *self.breakpoint_enabled.write().unwrap() = true;
    }

    fn Breakpoint_disable(&self) {
        *self.breakpoint_enabled.write().unwrap() = false;
    }
}

impl SharedLibJVMTI {
    pub fn agent_load(&self, state: &JVMState, thread: &JavaThread) -> jvmtiError {
        unsafe {
            let agent_load_symbol = self.lib.get::<fn(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) -> jint>("Agent_OnLoad".as_bytes()).unwrap();
            let agent_load_fn_ptr = agent_load_symbol.deref();
            let args = CString::new("transport=dt_socket,server=y,suspend=n,address=5005").unwrap().into_raw();//todo parse these at jvm startup
            let interface: *const JNIInvokeInterface_ = get_invoke_interface(state);
            agent_load_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterface_, args, std::ptr::null_mut()) as jvmtiError//todo leak
        }
    }
}

impl SharedLibJVMTI {
    pub fn load_libjdwp(jdwp_path: &str) -> SharedLibJVMTI {
        SharedLibJVMTI {
            lib: Arc::new(Library::new(jdwp_path).unwrap()),
            vm_init_callback: RwLock::new(None),
            vm_init_enabled: RwLock::new(false),
            vm_death_callback: RwLock::new(None),
            vm_death_enabled: RwLock::new(false),
            exception_callback: RwLock::new(None),
            exception_enabled: RwLock::new(false),
            thread_start_callback: RwLock::new(None),
            thread_start_enabled: RwLock::new(false),
            thread_end_callback: Default::default(),
            thread_end_enabled: Default::default(),
            class_prepare_callback: Default::default(),
            class_prepare_enabled: Default::default(),
            garbage_collection_finish_callback: Default::default(),
            garbage_collection_finish_enabled: Default::default(),
            class_load_callback: Default::default(),
            exception_catch_callback: Default::default(),
            single_step_callback: Default::default(),
            frame_pop_callback: Default::default(),
            breakpoint_callback: Default::default(),
            field_access_callback: Default::default(),
            field_modification_callback: Default::default(),
            method_entry_callback: Default::default(),
            method_exit_callback: Default::default(),
            monitor_wait_callback: Default::default(),
            monitor_waited_callback: Default::default(),
            monitor_conteded_enter_callback: Default::default(),
            monitor_conteded_entered_callback: Default::default(),
            breakpoint_enabled: Default::default()
        }
    }
}

pub unsafe fn get_state<'l>(env: *mut jvmtiEnv) -> &'l JVMState/*<'l>*/ {
    transmute((**env).reserved1)
}

thread_local! {
    static JVMTI_INTERFACE: RefCell<Option<jvmtiInterface_1_>> = RefCell::new(None);
}


pub fn get_jvmti_interface(state: &JVMState) -> jvmtiEnv {
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
        let new = get_jvmti_interface_impl(state);
        refcell.replace(new.into());
        let new_borrow = refcell.borrow();
        new_borrow.as_ref().unwrap() as jvmtiEnv
    })
}

fn get_jvmti_interface_impl(state: &JVMState) -> jvmtiInterface_1_ {
    jvmtiInterface_1_ {
        reserved1: unsafe { transmute(state) },
        SetEventNotificationMode: Some(set_event_notification_mode),
        reserved3: std::ptr::null_mut(),
        GetAllThreads: Some(get_all_threads),
        SuspendThread: None,
        ResumeThread: None,
        StopThread: None,
        InterruptThread: None,
        GetThreadInfo: None,
        GetOwnedMonitorInfo: None,
        GetCurrentContendedMonitor: None,
        RunAgentThread: Some(run_agent_thread),
        GetTopThreadGroups: Some(get_top_thread_groups),
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
        GetClassSignature: None,
        GetClassStatus: Some(get_class_status),
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
        GetMethodLocation: Some(get_method_location),
        GetLocalVariableTable: None,
        SetNativeMethodPrefix: None,
        SetNativeMethodPrefixes: None,
        GetBytecodes: None,
        IsMethodNative: None,
        IsMethodSynthetic: None,
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

pub unsafe extern "C" fn get_method_location(env: *mut jvmtiEnv, method: jmethodID, start_location_ptr: *mut jlocation, end_location_ptr: *mut jlocation) -> jvmtiError {
    let method_id = (method as *mut MethodId).as_ref().unwrap();
    match method_id.class.class_view.method_view_i(method_id.method_i).code_attribute() {
        None => {
            start_location_ptr.write(-1);
            end_location_ptr.write(-1);
        }
        Some(code) => {
            start_location_ptr.write(0);
            end_location_ptr.write((code.code.len() - 1) as i64);
        }
    };
    jvmtiError_JVMTI_ERROR_NONE
}

pub unsafe extern "C" fn dispose_environment(_env: *mut jvmtiEnv) -> jvmtiError {
    jvmtiError_JVMTI_ERROR_MUST_POSSESS_CAPABILITY
}

pub unsafe extern "C" fn set_breakpoint(env: *mut jvmtiEnv, method: jmethodID, location: jlocation) -> jvmtiError{
    let jvm = get_state(env);
    let method_id = (method as *mut MethodId).as_ref().unwrap();
    let mut breakpoint_guard = jvm.jvmti_state.break_points.write().unwrap();
    match breakpoint_guard.get(method_id){
        None => {
            breakpoint_guard.insert(method_id.clone(),RwLock::new(HashSet::from_iter(vec![location as isize].iter().cloned())));
        },
        Some(breakpoints) => {
            breakpoints.write().unwrap().insert(location as isize);//todo should I cast here?
        },
    }
    jvmtiError_JVMTI_ERROR_NONE
}

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