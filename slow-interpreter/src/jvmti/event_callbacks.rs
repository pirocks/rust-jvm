use std::ffi::{c_void, CString};
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, RwLock};

use libloading::Library;

use jvmti_jni_bindings::*;
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JavaThread, JVMState};
use crate::class_objects::get_or_create_class_object;
use crate::invoke_interface::get_invoke_interface;
use crate::java::lang::thread::JThread;
use crate::jvmti::{get_jvmti_interface, get_state};
use crate::method_table::MethodId;
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::native_util::to_object;
use crate::tracing::TracingSettings;

// does not support per thread notification
// VMInit
// VMStart
// VMDeath
// ThreadStart
// CompiledMethodLoad
// CompiledMethodUnload
// DynamicCodeGenerated
// DataDumpRequest
//todo technically speaking the RwLock needs to be less fine grain b/c setting callbacks is meant to be atomic
pub struct SharedLibJVMTI {
    lib: Arc<Library>,
    vm_init_callback: RwLock<jvmtiEventVMInit>,
    vm_init_enabled: RwLock<bool>,
    vm_death_callback: RwLock<jvmtiEventVMDeath>,
    vm_death_enabled: RwLock<bool>,
    thread_start_callback: RwLock<jvmtiEventThreadStart>,
    thread_start_enabled: RwLock<bool>,

    exception_callback: RwLock<jvmtiEventException>,
    thread_end_callback: RwLock<jvmtiEventThreadEnd>,
    class_prepare_callback: RwLock<jvmtiEventClassPrepare>,
    garbage_collection_finish_callback: RwLock<jvmtiEventGarbageCollectionFinish>,
    breakpoint_callback: RwLock<jvmtiEventBreakpoint>,

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

#[derive(Debug)]
pub struct ThreadJVMTIEnabledStatus {
    exception_enabled: bool,
    thread_end_enabled: bool,
    class_prepare_enabled: bool,
    garbage_collection_finish_enabled: bool,
    breakpoint_enabled: bool,
}

impl Default for ThreadJVMTIEnabledStatus {
    fn default() -> Self {
        Self {
            exception_enabled: false,
            thread_end_enabled: false,
            class_prepare_enabled: false,
            garbage_collection_finish_enabled: false,
            breakpoint_enabled: false,
        }
    }
}

#[derive(Clone)]
pub enum JVMTIEvent {
    VMInit(VMInitEvent),
    ThreadStart(ThreadStartEvent),
    Breakpoint(BreakpointEvent),
    ClassPrepare(ClassPrepareEvent),
}

#[derive(Clone)]
pub struct VMInitEvent {
    thread: jthread
}

#[derive(Clone)]
pub struct ThreadStartEvent {
    thread: jthread
}

#[derive(Clone)]
pub struct BreakpointEvent {
    thread: *mut _jobject,
    method: *mut _jmethodID,
    location: i64,
}

#[derive(Clone)]
pub struct ClassPrepareEvent {
    thread: jthread,
    klass: jclass,
}

#[derive(Clone)]
pub struct ExceptionEvent {
    thread: *mut _jobject,
    method: *mut _jmethodID,
    location: i64,
    exception: *mut _jobject,
    catch_method: *mut _jmethodID,
    catch_location: i64,
}

impl SharedLibJVMTI {
    /*fn trigger_event_all_threads(jvm: &'static JVMState, jvmti_event: &JVMTIEvent) {
        jvm.thread_state.alive_threads.read().unwrap().values().for_each(|t| {
            jvm.trigger_jvmti_event(t, jvmti_event.clone())
        })
    }*/

    /* fn trigger_event_threads(jvm: &'static JVMState, threads: &HashMap<JavaThreadId, bool>, jvmti_event: &dyn Fn() -> JVMTIEvent) {
         threads.iter().for_each(|(tid, enabled)| {
             if *enabled {
                 let read_guard = jvm.thread_state.alive_threads.read().unwrap();
                 let t = read_guard.get(tid).unwrap();
                 jvm.trigger_jvmti_event(t, jvmti_event())
             }
         });
     }*/

    pub fn vm_inited(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, main_thread: Arc<JavaThread>) {
        if *self.vm_init_enabled.read().unwrap() {
            unsafe {
                let main_thread_object = main_thread.thread_object();
                let event = VMInitEvent {
                    thread: transmute(to_object(main_thread_object.object().into()))
                };
                self.VMInit(jvm, int_state, event);
            }
        }
    }

    pub fn thread_start(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, jthread: JThread) {
        if *self.thread_start_enabled.read().unwrap() {
            unsafe {
                let thread = to_object(jthread.object().into());
                let event = ThreadStartEvent { thread };
                self.ThreadStart(jvm, int_state, event);
            }
        }
    }


    pub fn class_prepare<'l>(&self, jvm: &'static JVMState, class: &ClassName, int_state: &mut InterpreterStateGuard) {
        if jvm.thread_state.get_current_thread().jvmti_event_status().class_prepare_enabled {
            unsafe {
                let thread = to_object(jvm.thread_state.try_get_current_thread().and_then(|t| t.try_thread_object()).and_then(|jt| jt.object().into()));
                let klass_obj = get_or_create_class_object(jvm,
                                                           &class.clone().into(),
                                                           int_state,
                                                           jvm.bootstrap_loader.clone());
                let klass = to_object(klass_obj.into());
                let event = ClassPrepareEvent { thread, klass };
                self.ClassPrepare(jvm, int_state, event)
            }
        }
    }

    pub fn breakpoint(&self, jvm: &'static JVMState, method: MethodId, location: i64) {
        unsafe {
            let event_getter = &|| {
                let thread = to_object(jvm.thread_state.get_current_thread().thread_object().object().into());
                let native_method_id = jvm.native_interface_allocations.allocate_box(method.clone());//todo use a vtable based methodId
                let method = transmute(native_method_id);
                let jvmti_event = JVMTIEvent::Breakpoint(BreakpointEvent {
                    thread,
                    method,
                    location,
                });
                jvmti_event
            };
            unimplemented!()
            // SharedLibJVMTI::trigger_event_threads(jvm, &self.breakpoint_enabled.read().unwrap(), event_getter);
        }
    }
}


#[allow(non_snake_case)]
pub trait DebuggerEventConsumer {
    unsafe fn VMInit(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, vminit: VMInitEvent);

    fn VMInit_enable(&self, trace: &TracingSettings);
    fn VMInit_disable(&self, trace: &TracingSettings);

    unsafe fn VMDeath(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv);

    fn VMDeath_enable(&self, trace: &TracingSettings);
    fn VMDeath_disable(&self, trace: &TracingSettings);

    unsafe fn ThreadStart(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: ThreadStartEvent);

    fn ThreadStart_enable(&self, trace: &TracingSettings);
    fn ThreadStart_disable(&self, trace: &TracingSettings);

    unsafe fn Exception(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: ExceptionEvent);

    fn Exception_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);
    fn Exception_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);

    unsafe fn ThreadEnd(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadEnd_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);
    fn ThreadEnd_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);

    unsafe fn ClassPrepare(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: ClassPrepareEvent);

    fn ClassPrepare_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);
    fn ClassPrepare_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);


    unsafe fn GarbageCollectionFinish(jvmti_env: *mut jvmtiEnv);
    fn GarbageCollectionFinish_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);
    fn GarbageCollectionFinish_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);


    unsafe fn Breakpoint(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: BreakpointEvent);
    fn Breakpoint_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);
    fn Breakpoint_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>);
}

#[allow(non_snake_case)]
impl DebuggerEventConsumer for SharedLibJVMTI {
    unsafe fn VMInit(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, vminit: VMInitEvent) {
        jvm.tracing.trace_event_trigger("VMInit");
        let VMInitEvent { thread } = vminit;
        let jvmti = Box::leak(box get_jvmti_interface(jvm));
        let jni = get_interface(jvm, int_state);//todo deal with leak
        let guard = self.vm_init_callback.read().unwrap();
        let f_pointer = *guard.as_ref().unwrap();
        std::mem::drop(guard);
        f_pointer(jvmti, jni, thread);
    }

    fn VMInit_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("VMInit");
        *self.vm_init_enabled.write().unwrap() = true;
    }
    fn VMInit_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("VMInit");
        *self.vm_init_enabled.write().unwrap() = false;
    }

    unsafe fn VMDeath(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_) {
        if *self.vm_death_enabled.read().unwrap() {
            (self.vm_death_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env);
        }
    }

    fn VMDeath_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("VMDeath");
        *self.vm_death_enabled.write().unwrap() = true;
    }
    fn VMDeath_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("VMDeath");
        *self.vm_death_enabled.write().unwrap() = false;
    }

    unsafe fn ThreadStart(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: ThreadStartEvent) {
        jvm.tracing.trace_event_trigger("ThreadStart");
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let jni_env = get_interface(jvm, int_state);//fix these leaks
        let ThreadStartEvent { thread } = event;
        (self.thread_start_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread);
    }

    fn ThreadStart_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("ThreadStart");
        *self.thread_start_enabled.write().unwrap() = true;
    }
    fn ThreadStart_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("ThreadStart");
        *self.thread_start_enabled.write().unwrap() = false;
    }

    unsafe fn Exception(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: ExceptionEvent) {
        let jni_env = get_interface(jvm, int_state);
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let ExceptionEvent { thread, method, location, exception, catch_method, catch_location } = event;
        (self.exception_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location, exception, catch_method, catch_location);
    }

    fn Exception_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.exception_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "Exception")
    }
    fn Exception_disable(&self, jvm: &'static JVMState, java_thread: Option<Arc<JavaThread>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.exception_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, java_thread, &disabler, "Exception")
    }

    unsafe fn ThreadEnd(_jvmti_env: *mut *const jvmtiInterface_1_, _jni_env: *mut *const JNINativeInterface_, _thread: jthread) {
        unimplemented!()
    }
    fn ThreadEnd_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.thread_end_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "ThreadEnd")
    }
    fn ThreadEnd_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.thread_end_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "ThreadEnd")
    }

    unsafe fn ClassPrepare(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: ClassPrepareEvent) {
        jvm.tracing.trace_event_trigger("ClassPrepare");
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));//todo deal with these leaks
        let jni_env = get_interface(jvm, int_state);
        let ClassPrepareEvent { thread, klass } = event;
        (self.class_prepare_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, klass);
    }

    fn ClassPrepare_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.class_prepare_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "ClassPrepare")
    }

    fn ClassPrepare_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.class_prepare_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "ClassPrepare")
    }

    unsafe fn GarbageCollectionFinish(_jvmti_env: *mut *const jvmtiInterface_1_) {
//todo blocking on having a garbage collector
        unimplemented!()
    }

    fn GarbageCollectionFinish_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.garbage_collection_finish_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "GarbageCollectionFinish")
    }


    fn GarbageCollectionFinish_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.garbage_collection_finish_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "GarbageCollectionFinish")
    }

    unsafe fn Breakpoint(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, event: BreakpointEvent) {
        jvm.tracing.trace_event_trigger("Breakpoint");
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let jni_env = get_interface(jvm, int_state);
        let BreakpointEvent { thread, method, location } = event;
        (self.breakpoint_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location);
    }

    fn Breakpoint_enable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.breakpoint_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "Breakpoint")
    }

    fn Breakpoint_disable(&self, jvm: &'static JVMState, tid: Option<Arc<JavaThread>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.breakpoint_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "Breakpoint")
    }
}


impl SharedLibJVMTI {
    pub fn agent_load(&self, jvm: &'static JVMState) -> jvmtiError {
        unsafe {
            let agent_load_symbol = self.lib.get::<fn(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) -> jint>("Agent_OnLoad".as_bytes()).unwrap();
            let agent_load_fn_ptr = agent_load_symbol.deref();
            let args = CString::new("transport=dt_socket,server=y,suspend=y,address=5005").unwrap().into_raw();//todo parse these at jvm startup
            let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm, None);
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
            thread_start_callback: RwLock::new(None),
            thread_start_enabled: RwLock::new(false),
            thread_end_callback: Default::default(),
            class_prepare_callback: Default::default(),
            garbage_collection_finish_callback: Default::default(),
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
        }
    }
}


#[allow(non_snake_case)]
pub unsafe extern "C" fn set_event_callbacks(env: *mut jvmtiEnv, callbacks: *const jvmtiEventCallbacks, _size_of_callbacks: jint) -> jvmtiError {
//todo use size_of_callbacks ?
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm, "SetEventCallbacks");
    let mut callback_copy = jvmtiEventCallbacks {
        VMInit: None,
        VMDeath: None,
        ThreadStart: None,
        ThreadEnd: None,
        ClassFileLoadHook: None,
        ClassLoad: None,
        ClassPrepare: None,
        VMStart: None,
        Exception: None,
        ExceptionCatch: None,
        SingleStep: None,
        FramePop: None,
        Breakpoint: None,
        FieldAccess: None,
        FieldModification: None,
        MethodEntry: None,
        MethodExit: None,
        NativeMethodBind: None,
        CompiledMethodLoad: None,
        CompiledMethodUnload: None,
        DynamicCodeGenerated: None,
        DataDumpRequest: None,
        reserved72: None,
        MonitorWait: None,
        MonitorWaited: None,
        MonitorContendedEnter: None,
        MonitorContendedEntered: None,
        reserved77: None,
        reserved78: None,
        reserved79: None,
        ResourceExhausted: None,
        GarbageCollectionStart: None,
        GarbageCollectionFinish: None,
        ObjectFree: None,
        VMObjectAlloc: None,
    };
    libc::memcpy(&mut callback_copy as *mut jvmtiEventCallbacks as *mut libc::c_void, callbacks as *const libc::c_void, size_of::<jvmtiEventCallbacks>());
    let jvmtiEventCallbacks {
        VMInit,
        VMDeath,
        ThreadStart,
        ThreadEnd,
        ClassFileLoadHook,
        ClassLoad,
        ClassPrepare,
        VMStart,
        Exception,
        ExceptionCatch,
        SingleStep,
        FramePop,
        Breakpoint,
        FieldAccess,
        FieldModification,
        MethodEntry,
        MethodExit,
        NativeMethodBind,
        CompiledMethodLoad,
        CompiledMethodUnload,
        DynamicCodeGenerated,
        DataDumpRequest,
        reserved72,
        MonitorWait,
        MonitorWaited,
        MonitorContendedEnter,
        MonitorContendedEntered,
        reserved77: _,
        reserved78: _,
        reserved79: _,
        ResourceExhausted,
        GarbageCollectionStart,
        GarbageCollectionFinish,
        ObjectFree,
        VMObjectAlloc
    } = callback_copy;

    if VMInit.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.vm_init_callback.write().unwrap() = VMInit;
    }
    if VMDeath.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.vm_death_callback.write().unwrap() = VMDeath;
    }
    if ThreadStart.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_start_callback.write().unwrap() = ThreadStart;
    }
    if ThreadEnd.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.thread_end_callback.write().unwrap() = ThreadEnd;
    }
    if ClassFileLoadHook.is_some() {
        unimplemented!()
    }
    if ClassLoad.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.class_load_callback.write().unwrap() = ClassLoad;
    }
    if ClassPrepare.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.class_prepare_callback.write().unwrap() = ClassPrepare;
    }
    if VMStart.is_some() {
        unimplemented!()
    }
    if Exception.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.exception_callback.write().unwrap() = Exception;
    }
    if ExceptionCatch.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.exception_catch_callback.write().unwrap() = ExceptionCatch;
    }
    if SingleStep.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.single_step_callback.write().unwrap() = SingleStep;
    }
    if FramePop.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.frame_pop_callback.write().unwrap() = FramePop;
    }
    if Breakpoint.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.breakpoint_callback.write().unwrap() = Breakpoint;
    }
    if FieldAccess.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.field_access_callback.write().unwrap() = FieldAccess;
    }
    if FieldModification.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.field_modification_callback.write().unwrap() = FieldModification;
    }
    if MethodEntry.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.method_entry_callback.write().unwrap() = MethodEntry;
    }
    if MethodExit.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.method_exit_callback.write().unwrap() = MethodExit;
    }
    if NativeMethodBind.is_some() {
        unimplemented!()
    }
    if CompiledMethodLoad.is_some() {
        unimplemented!()
    }
    if CompiledMethodUnload.is_some() {
        unimplemented!()
    }
    if DynamicCodeGenerated.is_some() {
        unimplemented!()
    }
    if DataDumpRequest.is_some() {
        unimplemented!()
    }
    if reserved72.is_some() {
        unimplemented!()
    }
    if MonitorWait.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.monitor_wait_callback.write().unwrap() = MonitorWait;
    }
    if MonitorWaited.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.monitor_waited_callback.write().unwrap() = MonitorWaited;
    }
    if MonitorContendedEnter.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.monitor_conteded_enter_callback.write().unwrap() = MonitorContendedEnter;
    }
    if MonitorContendedEntered.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.monitor_conteded_entered_callback.write().unwrap() = MonitorContendedEntered;
    }
    if ResourceExhausted.is_some() {
        unimplemented!()
    }
    if GarbageCollectionStart.is_some() {
        unimplemented!()
    }
    if GarbageCollectionFinish.is_some() {
        *jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.garbage_collection_finish_callback.write().unwrap() = GarbageCollectionFinish;
    }
    if ObjectFree.is_some() {
        unimplemented!()
    }
    if VMObjectAlloc.is_some() {
        unimplemented!()
    }
    jvm.tracing.trace_jdwp_function_exit(jvm, "SetEventCallbacks");
    jvmtiError_JVMTI_ERROR_NONE
}

impl SharedLibJVMTI {
    //todo these are essentially the same merge into one?
    fn disable_impl(jvm: &'static JVMState, java_thread: Option<Arc<JavaThread>>, disabler: &dyn Fn(&mut ThreadJVMTIEnabledStatus), event_name: &str) {
        jvm.tracing.trace_event_disable_global(event_name);
        match java_thread {
            None => {
                for java_thread in jvm.thread_state.all_java_threads.read().unwrap().values() {
                    disabler(&mut java_thread.jvmti_event_status_mut());
                }
            },
            Some(java_thread) => {
                disabler(&mut java_thread.jvmti_event_status_mut());
            },
        }
    }

    fn enable_impl(jvm: &'static JVMState, java_thread: Option<Arc<JavaThread>>, enabler: &dyn Fn(&mut ThreadJVMTIEnabledStatus), event_name: &str) {
        jvm.tracing.trace_event_enable_global(event_name);
        match java_thread {
            None => {
                for java_thread in jvm.thread_state.all_java_threads.read().unwrap().values() {
                    enabler(&mut java_thread.jvmti_event_status_mut());
                }
            },
            Some(java_thread) => {
                enabler(&mut java_thread.jvmti_event_status_mut());
            },
        }
    }
}
