use libloading::Library;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use jvmti_jni_bindings::*;
use crate::{JVMState, JavaThread, ThreadId};
use crate::rust_jni::interface::get_interface;
use crate::jvmti::{get_jvmti_interface, get_state};
use std::mem::{transmute, size_of};
use crate::rust_jni::native_util::to_object;
use crate::invoke_interface::get_invoke_interface;
use std::ffi::{CString, c_void};
use std::os::raw::c_char;
use rust_jvm_common::classnames::ClassName;
use crate::class_objects::get_or_create_class_object;
use crate::java::lang::thread::JThread;
use std::ops::Deref;
use crate::tracing::TracingSettings;
use std::collections::HashMap;
use std::collections::hash_map::RandomState;
use crate::method_table::MethodId;


// does not support per thread notification
// VMInit
// VMStart
// VMDeath
// ThreadStart
// CompiledMethodLoad
// CompiledMethodUnload
// DynamicCodeGenerated
// DataDumpRequest
pub struct SharedLibJVMTI {
    lib: Arc<Library>,
    vm_init_callback: RwLock<jvmtiEventVMInit>,
    vm_init_enabled: RwLock<bool>,
    vm_death_callback: RwLock<jvmtiEventVMDeath>,
    vm_death_enabled: RwLock<bool>,
    thread_start_callback: RwLock<jvmtiEventThreadStart>,
    thread_start_enabled: RwLock<bool>,

    exception_callback: RwLock<jvmtiEventException>,
    exception_enabled: RwLock<HashMap<ThreadId, bool>>,
    thread_end_callback: RwLock<jvmtiEventThreadEnd>,
    thread_end_enabled: RwLock<HashMap<ThreadId, bool>>,
    class_prepare_callback: RwLock<jvmtiEventClassPrepare>,
    class_prepare_enabled: RwLock<HashMap<ThreadId, bool>>,
    garbage_collection_finish_callback: RwLock<jvmtiEventGarbageCollectionFinish>,
    garbage_collection_finish_enabled: RwLock<HashMap<ThreadId, bool>>,
    breakpoint_callback: RwLock<jvmtiEventBreakpoint>,
    breakpoint_enabled: RwLock<HashMap<ThreadId, bool>>,

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
    fn trigger_event_all_threads(jvm: &JVMState, jvmti_event: &JVMTIEvent) {
        jvm.thread_state.alive_threads.read().unwrap().values().for_each(|t| {
            jvm.trigger_jvmti_event(t, jvmti_event.clone())
        })
    }

    fn trigger_event_threads(jvm: &JVMState, threads: &HashMap<ThreadId, bool>, jvmti_event: &dyn Fn() -> JVMTIEvent) {
        threads.iter().for_each(|(tid, enabled)| {
            if *enabled {
                let read_guard = jvm.thread_state.alive_threads.read().unwrap();
                let t = read_guard.get(tid).unwrap();
                jvm.trigger_jvmti_event(t, jvmti_event())
            }
        });
    }

    pub fn vm_inited(&self, jvm: &JVMState) {
        if *self.vm_init_enabled.read().unwrap() {
            let main_thread_guard = jvm.thread_state.main_thread.read().unwrap();
            let main_thread_nonnull = main_thread_guard.as_ref().unwrap();
            let thread_object_borrow = main_thread_nonnull.thread_object.borrow();
            let main_thread_object = thread_object_borrow.as_ref().unwrap().clone().object();
            let jvmti_event = JVMTIEvent::VMInit(
                VMInitEvent {
                    thread: unsafe { transmute(to_object(main_thread_object.into())) }
                });
            SharedLibJVMTI::trigger_event_all_threads(jvm, &jvmti_event)
        }
    }

    pub fn thread_start(&self, jvm: &JVMState, jthread: JThread) {
        if *self.thread_start_enabled.read().unwrap() {
            unsafe {
                let thread = to_object(jthread.object().into());
                let event = JVMTIEvent::ThreadStart(ThreadStartEvent { thread });
                SharedLibJVMTI::trigger_event_all_threads(jvm, &event);
            }
        }
    }


    pub fn class_prepare(&self, jvm: &JVMState, class: &ClassName) {
        let event_getter= &||{
            let thread = unsafe { to_object(jvm.get_current_thread().thread_object.borrow().clone().unwrap().object().into()) };
            let klass_obj = get_or_create_class_object(jvm,
                                                       &class.clone().into(),
                                                       jvm.get_current_frame().deref(),
                                                       jvm.bootstrap_loader.clone());
            let klass = unsafe { to_object(klass_obj.into()) };
            let event = JVMTIEvent::ClassPrepare(ClassPrepareEvent { thread, klass });
            // jvm.tracing.trace_event_trigger("")
            event
        };
        SharedLibJVMTI::trigger_event_threads(jvm, &self.class_prepare_enabled.read().unwrap(), event_getter);
    }

    pub fn breakpoint(&self, jvm: &JVMState, method: MethodId, location: i64) {
        unsafe {
            let event_getter = &|| {
                let thread = to_object(jvm.get_current_thread().thread_object.borrow().as_ref().unwrap().clone().object().into());
                let native_method_id = box method.clone();//todo leaks here , use a vtable based methodId
                let method = transmute(Box::leak(native_method_id));
                let jvmti_event = JVMTIEvent::Breakpoint(BreakpointEvent {
                    thread,
                    method,
                    location,
                });
                jvmti_event
            };
            SharedLibJVMTI::trigger_event_threads(jvm, &self.breakpoint_enabled.read().unwrap(), event_getter);
        }
    }
}


#[allow(non_snake_case)]
pub trait DebuggerEventConsumer {
    unsafe fn VMInit(&self, jvm: &JVMState, vminit: VMInitEvent);

    fn VMInit_enable(&self, trace: &TracingSettings);
    fn VMInit_disable(&self, trace: &TracingSettings);

    unsafe fn VMDeath(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv);

    fn VMDeath_enable(&self, trace: &TracingSettings);
    fn VMDeath_disable(&self, trace: &TracingSettings);

    unsafe fn ThreadStart(&self, jvm: &JVMState, event: ThreadStartEvent);

    fn ThreadStart_enable(&self, trace: &TracingSettings);
    fn ThreadStart_disable(&self, trace: &TracingSettings);

    unsafe fn Exception(&self, jvm: &JVMState, event: ExceptionEvent);

    fn Exception_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>);
    fn Exception_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>);

    unsafe fn ThreadEnd(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadEnd_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>);
    fn ThreadEnd_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>);

    unsafe fn ClassPrepare(&self, jvm: &JVMState, event: ClassPrepareEvent);

    fn ClassPrepare_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>);
    fn ClassPrepare_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>);


    unsafe fn GarbageCollectionFinish(jvmti_env: *mut jvmtiEnv);
    fn GarbageCollectionFinish_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>);
    fn GarbageCollectionFinish_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>);


    unsafe fn Breakpoint(&self, jvm: &JVMState, event: BreakpointEvent);
    fn Breakpoint_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>);
    fn Breakpoint_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>);
}

#[allow(non_snake_case)]
impl DebuggerEventConsumer for SharedLibJVMTI {
    unsafe fn VMInit(&self, jvm: &JVMState, vminit: VMInitEvent) {
        jvm.tracing.trace_event_trigger("VMInit");
        let VMInitEvent { thread } = vminit;
        let jvmti = Box::leak(box get_jvmti_interface(jvm));
        let jni = Box::leak(box get_interface(jvm));//todo deal with leak
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

    unsafe fn ThreadStart(&self, jvm: &JVMState, event: ThreadStartEvent) {
        jvm.tracing.trace_event_trigger("ThreadStart");
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let jni_env = Box::leak(box get_interface(jvm));//fix these leaks
        let ThreadStartEvent { thread } = event;
        (self.thread_start_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread);
    }

    fn ThreadStart_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("ThreadStart");
// assert!(self.thread_start_callback.read().unwrap().is_some());
        *self.thread_start_enabled.write().unwrap() = true;
    }
    fn ThreadStart_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("ThreadStart");
        *self.thread_start_enabled.write().unwrap() = false;
    }

    unsafe fn Exception(&self, jvm: &JVMState, event: ExceptionEvent) {
        let jni_env = Box::leak(box get_interface(jvm));
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let ExceptionEvent { thread, method, location, exception, catch_method, catch_location } = event;
        (self.exception_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location, exception, catch_method, catch_location);
    }

    fn Exception_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_enable_global("Exception");
        let guard = self.exception_enabled.write().unwrap();
        SharedLibJVMTI::enable_impl(tid, guard)
    }
    fn Exception_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_disable_global("Exception");
        let guard = self.exception_enabled.write().unwrap();
        SharedLibJVMTI::disable_impl(tid, guard)
    }

    unsafe fn ThreadEnd(_jvmti_env: *mut *const jvmtiInterface_1_, _jni_env: *mut *const JNINativeInterface_, _thread: jthread) {
        unimplemented!()
    }

    fn ThreadEnd_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_enable_global("ThreadEnd");
        let guard = self.thread_end_enabled.write().unwrap();
        SharedLibJVMTI::enable_impl(tid, guard)
    }
    fn ThreadEnd_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_disable_global("ThreadEnd");
        let guard = self.thread_end_enabled.write().unwrap();
        SharedLibJVMTI::disable_impl(tid, guard)
    }

    unsafe fn ClassPrepare(&self, jvm: &JVMState, event: ClassPrepareEvent) {
        jvm.tracing.trace_event_trigger("ClassPrepare");
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let jni_env = Box::leak(box get_interface(jvm));
        let ClassPrepareEvent { thread, klass } = event;
        (self.class_prepare_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, klass);
    }

    fn ClassPrepare_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_enable_global("ClassPrepare");
        let guard = self.class_prepare_enabled.write().unwrap();
        SharedLibJVMTI::enable_impl(tid, guard)
    }
    fn ClassPrepare_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_disable_global("ClassPrepare");
        let guard = self.class_prepare_enabled.write().unwrap();
        SharedLibJVMTI::disable_impl(tid, guard)
    }

    unsafe fn GarbageCollectionFinish(_jvmti_env: *mut *const jvmtiInterface_1_) {
//todo blocking on having a garbage collector
        unimplemented!()
    }

    fn GarbageCollectionFinish_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_enable_global("GarbageCollectionFinish");
        let guard = self.garbage_collection_finish_enabled.write().unwrap();
        SharedLibJVMTI::enable_impl(tid, guard)
    }
    fn GarbageCollectionFinish_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_disable_global("GarbageCollectionFinish");
        let guard = self.garbage_collection_finish_enabled.write().unwrap();
        SharedLibJVMTI::disable_impl(tid, guard)
    }

    unsafe fn Breakpoint(&self, jvm: &JVMState, event: BreakpointEvent) {
        jvm.tracing.trace_event_trigger("Breakpoint");
        let jvmti_env = Box::leak(box get_jvmti_interface(jvm));
        let jni_env = Box::leak(box get_interface(jvm));
        let BreakpointEvent { thread, method, location } = event;
        (self.breakpoint_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location);
    }

    fn Breakpoint_enable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_enable_global("Breakpoint");
        let guard = self.breakpoint_enabled.write().unwrap();
        SharedLibJVMTI::enable_impl(tid, guard)
    }
    fn Breakpoint_disable(&self, trace: &TracingSettings, tid: Option<ThreadId>) {
        trace.trace_event_disable_global("Breakpoint");
        let guard = self.breakpoint_enabled.write().unwrap();
        SharedLibJVMTI::disable_impl(tid, guard)
    }
}


impl SharedLibJVMTI {
    pub fn agent_load(&self, jvm: &JVMState, _thread: &JavaThread) -> jvmtiError {
//todo why is thread relevant/unused here
        jvm.init_signal_handler();
        unsafe {
            let agent_load_symbol = self.lib.get::<fn(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) -> jint>("Agent_OnLoad".as_bytes()).unwrap();
            let agent_load_fn_ptr = agent_load_symbol.deref();
            let args = CString::new("transport=dt_socket,server=y,suspend=y,address=5005").unwrap().into_raw();//todo parse these at jvm startup
            let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm);
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
            exception_enabled: RwLock::new(HashMap::new()),
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
            breakpoint_enabled: Default::default(),
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
        *jvm.jvmti_state.built_in_jdwp.vm_init_callback.write().unwrap() = VMInit;
    }
    if VMDeath.is_some() {
        *jvm.jvmti_state.built_in_jdwp.vm_death_callback.write().unwrap() = VMDeath;
    }
    if ThreadStart.is_some() {
        *jvm.jvmti_state.built_in_jdwp.thread_start_callback.write().unwrap() = ThreadStart;
    }
    if ThreadEnd.is_some() {
        *jvm.jvmti_state.built_in_jdwp.thread_end_callback.write().unwrap() = ThreadEnd;
    }
    if ClassFileLoadHook.is_some() {
        unimplemented!()
    }
    if ClassLoad.is_some() {
        *jvm.jvmti_state.built_in_jdwp.class_load_callback.write().unwrap() = ClassLoad;
    }
    if ClassPrepare.is_some() {
        *jvm.jvmti_state.built_in_jdwp.class_prepare_callback.write().unwrap() = ClassPrepare;
    }
    if VMStart.is_some() {
        unimplemented!()
    }
    if Exception.is_some() {
        *jvm.jvmti_state.built_in_jdwp.exception_callback.write().unwrap() = Exception;
    }
    if ExceptionCatch.is_some() {
        *jvm.jvmti_state.built_in_jdwp.exception_catch_callback.write().unwrap() = ExceptionCatch;
    }
    if SingleStep.is_some() {
        *jvm.jvmti_state.built_in_jdwp.single_step_callback.write().unwrap() = SingleStep;
    }
    if FramePop.is_some() {
        *jvm.jvmti_state.built_in_jdwp.frame_pop_callback.write().unwrap() = FramePop;
    }
    if Breakpoint.is_some() {
        *jvm.jvmti_state.built_in_jdwp.breakpoint_callback.write().unwrap() = Breakpoint;
    }
    if FieldAccess.is_some() {
        *jvm.jvmti_state.built_in_jdwp.field_access_callback.write().unwrap() = FieldAccess;
    }
    if FieldModification.is_some() {
        *jvm.jvmti_state.built_in_jdwp.field_modification_callback.write().unwrap() = FieldModification;
    }
    if MethodEntry.is_some() {
        *jvm.jvmti_state.built_in_jdwp.method_entry_callback.write().unwrap() = MethodEntry;
    }
    if MethodExit.is_some() {
        *jvm.jvmti_state.built_in_jdwp.method_exit_callback.write().unwrap() = MethodExit;
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
        *jvm.jvmti_state.built_in_jdwp.monitor_wait_callback.write().unwrap() = MonitorWait;
    }
    if MonitorWaited.is_some() {
        *jvm.jvmti_state.built_in_jdwp.monitor_waited_callback.write().unwrap() = MonitorWaited;
    }
    if MonitorContendedEnter.is_some() {
        *jvm.jvmti_state.built_in_jdwp.monitor_conteded_enter_callback.write().unwrap() = MonitorContendedEnter;
    }
    if MonitorContendedEntered.is_some() {
        *jvm.jvmti_state.built_in_jdwp.monitor_conteded_entered_callback.write().unwrap() = MonitorContendedEntered;
    }
    if ResourceExhausted.is_some() {
        unimplemented!()
    }
    if GarbageCollectionStart.is_some() {
        unimplemented!()
    }
    if GarbageCollectionFinish.is_some() {
        *jvm.jvmti_state.built_in_jdwp.garbage_collection_finish_callback.write().unwrap() = GarbageCollectionFinish;
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
    fn disable_impl(tid: Option<i64>, mut guard: RwLockWriteGuard<HashMap<i64, bool, RandomState>>) {
        match tid {
            None => {
                guard.iter_mut().for_each(|(_,enabled)| {
                    *enabled = false
                })
            }
            Some(key) => {
                guard.insert(key, false);
            }
        }
    }

    fn enable_impl(tid: Option<i64>, mut guard: RwLockWriteGuard<HashMap<i64, bool, RandomState>>) {
        match tid {
            None => {
                guard.iter_mut().for_each(|(_,enabled)| {
                    *enabled = true
                })
            }
            Some(key) => {
                guard.insert(key, true);
            }
        }
    }
}
