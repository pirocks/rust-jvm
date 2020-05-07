use libloading::Library;
use std::sync::{Arc, RwLock};
use jvmti_jni_bindings::{jvmtiEventVMInit, jvmtiEventVMDeath, jvmtiEventException, jvmtiEventThreadStart, jvmtiEventThreadEnd, jvmtiEventClassPrepare, jvmtiEventGarbageCollectionFinish, jvmtiEventBreakpoint, jvmtiEventClassLoad, jvmtiEventExceptionCatch, jvmtiEventSingleStep, jvmtiEventFramePop, jvmtiEventFieldAccess, jvmtiEventFieldModification, jvmtiEventMethodEntry, jvmtiEventMethodExit, jvmtiEventMonitorWait, jvmtiEventMonitorWaited, jvmtiEventMonitorContendedEnter, jvmtiEventMonitorContendedEntered, jvmtiEnv, JNIEnv, jthread, jclass, jmethodID, jlocation, jvmtiInterface_1_, JNINativeInterface_, _jmethodID, _jobject, JNIInvokeInterface_, JavaVM, jint, jvmtiError, jobject, jvmtiError_JVMTI_ERROR_NONE, jvmtiEventCallbacks};
use crate::{JVMState, JavaThread};
use crate::rust_jni::interface::get_interface;
use crate::jvmti::{get_jvmti_interface, get_state};
use std::mem::{transmute, size_of};
use crate::rust_jni::native_util::to_object;
use crate::rust_jni::MethodId;
use crate::invoke_interface::get_invoke_interface;
use std::ffi::{CString, c_void};
use std::os::raw::c_char;
use rust_jvm_common::classnames::ClassName;
use crate::class_objects::get_or_create_class_object;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use crate::java::lang::thread::JThread;
use std::ops::Deref;
use crate::tracing::TracingSettings;

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
            let mut casted_jni = interface as *const jvmti_jni_bindings::JNINativeInterface_ as *const libc::c_void as *const JNINativeInterface_;
            let main_thread_guard = state.thread_state.main_thread.read().unwrap();
            let main_thread_nonnull = main_thread_guard.as_ref().unwrap();
            let thread_object_borrow = main_thread_nonnull.thread_object.borrow();
            let main_thread_object = thread_object_borrow.as_ref().unwrap().clone().object();
            self.VMInit(&mut jvmti_interface, &mut casted_jni, transmute(to_object(main_thread_object.into())))
        }
    }
    pub fn thread_start(&self, jvm: &JVMState, thread: JThread) {
        unsafe {
            let obj = to_object(thread.object().into());
            let jvmti = get_jvmti_interface(jvm);
            let jni = get_interface(jvm);
            self.ThreadStart(Box::leak(Box::new(jvmti)), Box::leak(Box::new(transmute(jni))), transmute(obj));
        }
    }

    pub fn class_prepare(&self, jvm: &JVMState, class: &ClassName) {
        unsafe {
            if *self.class_prepare_enabled.read().unwrap() {
                let thread_obj = to_object(jvm.get_current_thread().thread_object.borrow().clone().unwrap().object().into());
                let jvmti = get_jvmti_interface(jvm);
                let jni = get_interface(jvm);
                let class_obj = get_or_create_class_object(jvm, &PTypeView::Ref(ReferenceTypeView::Class(class.clone())), jvm.get_current_frame().deref(), jvm.bootstrap_loader.clone());
                self.ClassPrepare(
                    Box::leak(Box::new(jvmti)),
                    Box::leak(Box::new(transmute(jni))),
                    transmute(thread_obj),
                    transmute(to_object(class_obj.into())),
                )//todo are these leaks needed
            }
        }
    }

    pub fn breakpoint(&self, jvm: &JVMState, method: MethodId, location: isize) {
        unsafe {
            let thread = to_object(jvm.get_current_thread().thread_object.borrow().as_ref().unwrap().clone().object().into());
            let jvmti = box get_jvmti_interface(jvm);
            let jni = box transmute(get_interface(jvm));
            let native_method_id = box method;//todo leaks here , use a vtable based methodId
            self.Breakpoint(
                Box::leak(jvmti),
                Box::leak(jni),
                transmute(thread),
                transmute(Box::leak(native_method_id)),
                location as i64,
            )
        }
    }
}


#[allow(non_snake_case)]
pub trait DebuggerEventConsumer {
    unsafe fn VMInit(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn VMInit_enable(&self, trace: &TracingSettings);
    fn VMInit_disable(&self, trace: &TracingSettings);

    unsafe fn VMDeath(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv);

    fn VMDeath_enable(&self, trace: &TracingSettings);
    fn VMDeath_disable(&self, trace: &TracingSettings);

    unsafe fn Exception(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation, exception: jobject, catch_method: jmethodID, catch_location: jlocation);

    fn Exception_enable(&self, trace: &TracingSettings);
    fn Exception_disable(&self, trace: &TracingSettings);

    unsafe fn ThreadStart(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadStart_enable(&self, trace: &TracingSettings);
    fn ThreadStart_disable(&self, trace: &TracingSettings);


    unsafe fn ThreadEnd(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadEnd_enable(&self, trace: &TracingSettings);
    fn ThreadEnd_disable(&self, trace: &TracingSettings);

    unsafe fn ClassPrepare(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, klass: jclass);

    fn ClassPrepare_enable(&self, trace: &TracingSettings);
    fn ClassPrepare_disable(&self, trace: &TracingSettings);


    unsafe fn GarbageCollectionFinish(jvmti_env: *mut jvmtiEnv);
    fn GarbageCollectionFinish_enable(&self, trace: &TracingSettings);
    fn GarbageCollectionFinish_disable(&self, trace: &TracingSettings);


    unsafe fn Breakpoint(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread, method: jmethodID, location: jlocation);
    fn Breakpoint_enable(&self, trace: &TracingSettings);
    fn Breakpoint_disable(&self, trace: &TracingSettings);
}

#[allow(non_snake_case)]
impl DebuggerEventConsumer for SharedLibJVMTI {
    unsafe fn VMInit(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject) {
        if *self.vm_init_enabled.read().unwrap() {
            let jvm = get_state(jvmti_env);
            jvm.tracing.trace_event_trigger("VMInit");
            let guard = self.vm_init_callback.read().unwrap();
            let f_pointer = *guard.as_ref().unwrap();
            std::mem::drop(guard);
            f_pointer(jvmti_env, jni_env, thread);
        }
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

    unsafe fn Exception(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject, method: *mut _jmethodID, location: i64, exception: *mut _jobject, catch_method: *mut _jmethodID, catch_location: i64) {
        if *self.exception_enabled.read().unwrap() {
            (self.exception_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location, exception, catch_method, catch_location);
        }
    }

    fn Exception_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("Exception");
        *self.exception_enabled.write().unwrap() = true;
    }

    fn Exception_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("Exception");
        *self.exception_enabled.write().unwrap() = false;
    }

    unsafe fn ThreadStart(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject) {
        if *self.thread_start_enabled.read().unwrap() {//todo kinda sorta maybe race condition
            get_state(jvmti_env).tracing.trace_event_trigger("ThreadStart");
            (self.thread_start_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread);
        }
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

    unsafe fn ThreadEnd(_jvmti_env: *mut *const jvmtiInterface_1_, _jni_env: *mut *const JNINativeInterface_, _thread: jthread) {
        unimplemented!()
    }

    fn ThreadEnd_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("ThreadEnd");
        *self.thread_start_enabled.write().unwrap() = true;
    }

    fn ThreadEnd_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("ThreadEnd");
        *self.thread_start_enabled.write().unwrap() = false;
    }

    unsafe fn ClassPrepare(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject, klass: *mut _jobject) {
        if *self.class_prepare_enabled.read().unwrap() {
            (self.class_prepare_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, klass);
        }
    }

    fn ClassPrepare_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("ClassPrepare");
        *self.class_prepare_enabled.write().unwrap() = true;
    }

    fn ClassPrepare_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("ClassPrepare");
        *self.class_prepare_enabled.write().unwrap() = false;
    }

    unsafe fn GarbageCollectionFinish(_jvmti_env: *mut *const jvmtiInterface_1_) {
        //todo blocking on having a garbage collector
        unimplemented!()
    }

    fn GarbageCollectionFinish_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("GarbageCollectionFinish");
        *self.garbage_collection_finish_enabled.write().unwrap() = true;
    }

    fn GarbageCollectionFinish_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("GarbageCollectionFinish");
        *self.garbage_collection_finish_enabled.write().unwrap() = false;
    }

    unsafe fn Breakpoint(&self, jvmti_env: *mut *const jvmtiInterface_1_, jni_env: *mut *const JNINativeInterface_, thread: *mut _jobject, method: *mut _jmethodID, location: i64) {
        if *self.breakpoint_enabled.read().unwrap() {
            (self.breakpoint_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location);
        }
    }

    fn Breakpoint_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("Breakpoint");
        *self.breakpoint_enabled.write().unwrap() = true;
    }

    fn Breakpoint_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("Breakpoint");
        *self.breakpoint_enabled.write().unwrap() = false;
    }
}

impl SharedLibJVMTI {
    pub fn agent_load(&self, state: &JVMState, _thread: &JavaThread) -> jvmtiError {
        //todo why is thread relevant/unused here
        unsafe {
            let agent_load_symbol = self.lib.get::<fn(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) -> jint>("Agent_OnLoad".as_bytes()).unwrap();
            let agent_load_fn_ptr = agent_load_symbol.deref();
            let args = CString::new("transport=dt_socket,server=y,suspend=y,address=5005").unwrap().into_raw();//todo parse these at jvm startup
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
            breakpoint_enabled: Default::default(),
        }
    }
}


#[allow(non_snake_case)]
pub unsafe extern "C" fn set_event_callbacks(env: *mut jvmtiEnv, callbacks: *const jvmtiEventCallbacks, _size_of_callbacks: jint) -> jvmtiError {
    //todo use size_of_callbacks ?
    let jvm = get_state(env);
    jvm.tracing.trace_jdwp_function_enter(jvm,"SetEventCallbacks");
    let mut callback_copy = jvmtiEventCallbacks{
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
        VMObjectAlloc: None
    };
    libc::memcpy(&mut callback_copy as *mut jvmtiEventCallbacks as *mut libc::c_void,callbacks as *const libc::c_void,size_of::<jvmtiEventCallbacks>());
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
        reserved77:_,
        reserved78:_,
        reserved79:_,
        ResourceExhausted,
        GarbageCollectionStart,
        GarbageCollectionFinish,
        ObjectFree,
        VMObjectAlloc
    } = callback_copy;

    if VMInit.is_some(){
        *jvm.jvmti_state.built_in_jdwp.vm_init_callback.write().unwrap() = VMInit;
    }
    if VMDeath.is_some(){
        *jvm.jvmti_state.built_in_jdwp.vm_death_callback.write().unwrap() = VMDeath;
    }
    if ThreadStart.is_some(){
        *jvm.jvmti_state.built_in_jdwp.thread_start_callback.write().unwrap() = ThreadStart;
    }
    if ThreadEnd.is_some(){
        *jvm.jvmti_state.built_in_jdwp.thread_end_callback.write().unwrap() = ThreadEnd;
    }
    if ClassFileLoadHook.is_some(){
        unimplemented!()
    }
    if ClassLoad.is_some(){
        *jvm.jvmti_state.built_in_jdwp.class_load_callback.write().unwrap() = ClassLoad;
    }
    if ClassPrepare.is_some(){
        *jvm.jvmti_state.built_in_jdwp.class_prepare_callback.write().unwrap() = ClassPrepare;
    }
    if VMStart.is_some(){
        unimplemented!()
    }
    if Exception.is_some(){
        *jvm.jvmti_state.built_in_jdwp.exception_callback.write().unwrap() = Exception;
    }
    if ExceptionCatch.is_some(){
        *jvm.jvmti_state.built_in_jdwp.exception_catch_callback.write().unwrap() = ExceptionCatch;
    }
    if SingleStep.is_some(){
        *jvm.jvmti_state.built_in_jdwp.single_step_callback.write().unwrap() = SingleStep;
    }
    if FramePop.is_some(){
        *jvm.jvmti_state.built_in_jdwp.frame_pop_callback.write().unwrap() = FramePop;
    }
    if Breakpoint.is_some(){
        *jvm.jvmti_state.built_in_jdwp.breakpoint_callback.write().unwrap() = Breakpoint;
    }
    if FieldAccess.is_some(){
        *jvm.jvmti_state.built_in_jdwp.field_access_callback.write().unwrap() = FieldAccess;
    }
    if FieldModification.is_some(){
        *jvm.jvmti_state.built_in_jdwp.field_modification_callback.write().unwrap() = FieldModification;
    }
    if MethodEntry.is_some(){
        *jvm.jvmti_state.built_in_jdwp.method_entry_callback.write().unwrap() = MethodEntry;
    }
    if MethodExit.is_some(){
        *jvm.jvmti_state.built_in_jdwp.method_exit_callback.write().unwrap() = MethodExit;
    }
    if NativeMethodBind.is_some(){
        unimplemented!()
    }
    if CompiledMethodLoad.is_some(){
        unimplemented!()
    }
    if CompiledMethodUnload.is_some(){
        unimplemented!()
    }
    if DynamicCodeGenerated.is_some(){
        unimplemented!()
    }
    if DataDumpRequest.is_some(){
        unimplemented!()
    }
    if reserved72.is_some(){
        unimplemented!()
    }
    if MonitorWait.is_some(){
        *jvm.jvmti_state.built_in_jdwp.monitor_wait_callback.write().unwrap() = MonitorWait;
    }
    if MonitorWaited.is_some(){
        *jvm.jvmti_state.built_in_jdwp.monitor_waited_callback.write().unwrap() = MonitorWaited;
    }
    if MonitorContendedEnter.is_some(){
        *jvm.jvmti_state.built_in_jdwp.monitor_conteded_enter_callback.write().unwrap() = MonitorContendedEnter;
    }
    if MonitorContendedEntered.is_some(){
        *jvm.jvmti_state.built_in_jdwp.monitor_conteded_entered_callback.write().unwrap() = MonitorContendedEntered;
    }
    if ResourceExhausted.is_some(){
        unimplemented!()
    }
    if GarbageCollectionStart.is_some(){
        unimplemented!()
    }
    if GarbageCollectionFinish.is_some(){
        *jvm.jvmti_state.built_in_jdwp.garbage_collection_finish_callback.write().unwrap() = GarbageCollectionFinish;
    }
    if ObjectFree.is_some(){
        unimplemented!()
    }
    if VMObjectAlloc.is_some(){
        unimplemented!()
    }
    jvm.tracing.trace_jdwp_function_exit(jvm,"SetEventCallbacks");
    jvmtiError_JVMTI_ERROR_NONE
}