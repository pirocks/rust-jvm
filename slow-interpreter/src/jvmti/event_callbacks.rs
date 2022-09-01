use std::ffi::{c_void, CString, OsString};
use std::mem::{size_of, transmute};
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, RwLock};

use libloading::Library;
use libloading::os::unix::RTLD_NOW;

use jvmti_jni_bindings::*;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::MethodId;

use crate::{InterpreterStateGuard, JavaThread, JVMState};
use crate::better_java_stack::frames::PushableFrame;
use crate::invoke_interface::get_invoke_interface;
use crate::java::lang::thread::JThread;
use crate::jvmti::{get_jvmti_interface, get_state};
use crate::rust_jni::interface::get_interface;
use crate::rust_jni::interface::local_frame::{new_local_ref_public};
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
    pub thread_start_callback: RwLock<jvmtiEventThreadStart>,
    thread_start_enabled: RwLock<bool>,

    exception_callback: RwLock<jvmtiEventException>,
    thread_end_callback: RwLock<jvmtiEventThreadEnd>,
    class_prepare_callback: RwLock<jvmtiEventClassPrepare>,
    garbage_collection_finish_callback: RwLock<jvmtiEventGarbageCollectionFinish>,
    breakpoint_callback: RwLock<jvmtiEventBreakpoint>,
    frame_pop_callback: RwLock<jvmtiEventFramePop>,

    class_load_callback: RwLock<jvmtiEventClassLoad>,
    exception_catch_callback: RwLock<jvmtiEventExceptionCatch>,
    single_step_callback: RwLock<jvmtiEventSingleStep>,
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
    _frame_pop_enabled: bool,
}

impl Default for ThreadJVMTIEnabledStatus {
    fn default() -> Self {
        Self {
            exception_enabled: false,
            thread_end_enabled: false,
            class_prepare_enabled: false,
            garbage_collection_finish_enabled: false,
            breakpoint_enabled: false,
            _frame_pop_enabled: false,
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
    thread: jthread,
}

#[derive(Clone)]
pub struct ThreadStartEvent {
    thread: jthread,
}

#[derive(Clone)]
pub struct BreakpointEvent {
    thread: *mut _jobject,
    method: *mut _jmethodID,
    location: i64,
}

#[derive(Clone)]
pub struct FramePopEvent {
    thread: jthread,
    method: jmethodID,
    was_popped_by_exception: jboolean,
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
    pub fn vm_inited<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, main_thread: Arc<JavaThread<'gc>>) {
        if *self.vm_init_enabled.read().unwrap() {
            unsafe {
                let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,LoaderName::BootstrapLoader, vec![],"vm_inited")*/);
                let main_thread_object = main_thread.thread_object();
                let event = VMInitEvent { thread: new_local_ref_public(todo!()/*main_thread_object.object().to_gc_managed().into()*/, int_state) };
                self.VMInit(jvm, int_state, event);
                assert!(self.thread_start_callback.read().unwrap().is_some());
                int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending excpetion anyway
            }
        }
    }

    pub fn thread_start<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, jthread: JThread<'gc>) {
        if *self.thread_start_enabled.read().unwrap() {
            let event_handling_frame = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,LoaderName::BootstrapLoader, vec![],"thread_start")*/);
            while !jvm.vm_live() {} //todo ofc theres a better way of doing this, but we are required to wait for vminit by the spec.
            assert!(jvm.vm_live());
            unsafe {
                let thread = new_local_ref_public(todo!()/*jthread.object().to_gc_managed().into()*/, int_state);
                let event = ThreadStartEvent { thread };
                self.ThreadStart(jvm, int_state, event);
            }
            int_state.pop_frame(jvm, event_handling_frame, false); //todo check for pending excpetion anyway
        }
    }

    pub fn class_prepare<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, class: &CClassName, int_state: &mut impl PushableFrame<'gc>) {
        if jvm.thread_state.get_current_thread().jvmti_event_status().class_prepare_enabled {
            unsafe {
                todo!();
                // let frame_for_event = int_state.push_frame(StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"class_prepare"));
                // //give the other events this long thing
                // let current_thread_from_rust = jvm.thread_state.try_get_current_thread().and_then(|t| t.try_thread_object()).and_then(|jt| jt.object().into());
                // let thread = new_local_ref_public_new(todo!()/*current_thread_from_rust*/, int_state);
                // let klass_obj = get_or_create_class_object(jvm, class.clone().into(), int_state).unwrap();
                // let klass = to_object(klass_obj.to_gc_managed().into());
                // let event = ClassPrepareEvent { thread, klass };
                // self.ClassPrepare(jvm, int_state, event);
                // int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending excpetion anyway
            }
        }
    }

    pub fn breakpoint<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, method: MethodId, location: i64, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) {
        if jvm.thread_state.get_current_thread().jvmti_event_status().breakpoint_enabled {
            unsafe {
                let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"breakpoint")*/);
                let thread = new_local_ref_public(todo!()/*jvm.thread_state.get_current_thread().thread_object().object().into()*/, int_state);
                let method = transmute(method);
                self.Breakpoint(jvm, int_state, BreakpointEvent { thread, method, location });
                int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending excpetion anyway
            }
        }
    }

    pub fn frame_pop<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, method: MethodId, was_popped_by_exception: jboolean, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) {
        if jvm.thread_state.get_current_thread().jvmti_event_status().breakpoint_enabled {
            unsafe {
                let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"frame_pop")*/);
                let thread = new_local_ref_public(todo!()/*jvm.thread_state.get_current_thread().thread_object().object().into()*/, int_state);
                let method = transmute(method);
                self.FramePop(jvm, int_state, FramePopEvent { thread, method, was_popped_by_exception });
                int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending excpetion anyway
            }
        }
    }
}

#[allow(non_snake_case)]
pub trait DebuggerEventConsumer {
    unsafe fn VMInit<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, vminit: VMInitEvent);

    fn VMInit_enable(&self, trace: &TracingSettings);
    fn VMInit_disable(&self, trace: &TracingSettings);

    unsafe fn VMDeath(&self, jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv);

    fn VMDeath_enable(&self, trace: &TracingSettings);
    fn VMDeath_disable(&self, trace: &TracingSettings);

    unsafe fn ThreadStart<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: ThreadStartEvent);

    fn ThreadStart_enable(&self, trace: &TracingSettings);
    fn ThreadStart_disable(&self, trace: &TracingSettings);

    unsafe fn Exception<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: ExceptionEvent);

    fn Exception_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
    fn Exception_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);

    unsafe fn ThreadEnd(jvmti_env: *mut jvmtiEnv, jni_env: *mut JNIEnv, thread: jthread);

    fn ThreadEnd_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
    fn ThreadEnd_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);

    unsafe fn ClassPrepare<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: ClassPrepareEvent);

    fn ClassPrepare_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
    fn ClassPrepare_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);

    unsafe fn GarbageCollectionFinish(jvmti_env: *mut jvmtiEnv);
    fn GarbageCollectionFinish_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
    fn GarbageCollectionFinish_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);

    unsafe fn Breakpoint<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: BreakpointEvent);
    fn Breakpoint_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
    fn Breakpoint_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);

    unsafe fn FramePop<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: FramePopEvent);
    fn FramePop_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
    fn FramePop_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>);
}

#[allow(non_snake_case)]
impl DebuggerEventConsumer for SharedLibJVMTI {
    unsafe fn VMInit<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, vminit: VMInitEvent) {
        jvm.config.tracing.trace_event_trigger("VMInit");
        let VMInitEvent { thread } = vminit;
        let jvmti = get_jvmti_interface(jvm, int_state);
        let jni = get_interface(jvm, todo!()/*int_state*/);
        let guard = self.vm_init_callback.read().unwrap();
        let f_pointer = *guard.as_ref().unwrap();
        drop(guard);
        let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"VMInit")*/);
        f_pointer(jvmti, jni, thread);
        int_state.pop_frame(jvm, frame_for_event, false); //todo check for excpet anyway
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

    unsafe fn ThreadStart<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: ThreadStartEvent) {
        jvm.config.tracing.trace_event_trigger("ThreadStart");
        let jvmti_env = get_jvmti_interface(jvm, int_state);
        let jni_env = get_interface(jvm, todo!()/*int_state*/);
        let ThreadStartEvent { thread } = event;
        let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"ThreadStart")*/);
        if let Some(callback) = self.thread_start_callback.read().unwrap().as_ref() {
            callback(jvmti_env, jni_env, thread)
        }
        int_state.pop_frame(jvm, frame_for_event, false); //todo check for excpet anyway
    }

    fn ThreadStart_enable(&self, trace: &TracingSettings) {
        trace.trace_event_enable_global("ThreadStart");
        *self.thread_start_enabled.write().unwrap() = true;
    }
    fn ThreadStart_disable(&self, trace: &TracingSettings) {
        trace.trace_event_disable_global("ThreadStart");
        *self.thread_start_enabled.write().unwrap() = false;
    }

    unsafe fn Exception<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: ExceptionEvent) {
        let jni_env = get_interface(jvm, todo!()/*int_state*/);
        let jvmti_env = get_jvmti_interface(jvm, int_state);
        let ExceptionEvent { thread, method, location, exception, catch_method, catch_location } = event;
        let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"Exception")*/);
        (self.exception_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, method, location, exception, catch_method, catch_location);
        int_state.pop_frame(jvm, frame_for_event, false); //todo check for excpet anyway
    }

    fn Exception_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.exception_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "Exception")
    }
    fn Exception_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, java_thread: Option<Arc<JavaThread<'gc>>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.exception_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, java_thread, &disabler, "Exception")
    }

    unsafe fn ThreadEnd(_jvmti_env: *mut *const jvmtiInterface_1_, _jni_env: *mut *const JNINativeInterface_, _thread: jthread) {
        unimplemented!()
    }
    fn ThreadEnd_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.thread_end_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "ThreadEnd")
    }
    fn ThreadEnd_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.thread_end_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "ThreadEnd")
    }

    unsafe fn ClassPrepare<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: ClassPrepareEvent) {
        jvm.config.tracing.trace_event_trigger("ClassPrepare");
        let jvmti_env = get_jvmti_interface(jvm, int_state); //todo deal with these leaks
        let jni_env = get_interface(jvm, todo!()/*int_state*/);
        let ClassPrepareEvent { thread, klass } = event;
        let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"ClassPrepare")*/);
        (self.class_prepare_callback.read().unwrap().as_ref().unwrap())(jvmti_env, jni_env, thread, klass);
        int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending excpetion anyway
    }

    fn ClassPrepare_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.class_prepare_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "ClassPrepare")
    }

    fn ClassPrepare_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.class_prepare_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "ClassPrepare")
    }

    unsafe fn GarbageCollectionFinish(_jvmti_env: *mut *const jvmtiInterface_1_) {
        //todo blocking on having a garbage collector
        unimplemented!()
    }

    fn GarbageCollectionFinish_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.garbage_collection_finish_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "GarbageCollectionFinish")
    }

    fn GarbageCollectionFinish_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.garbage_collection_finish_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "GarbageCollectionFinish")
    }

    unsafe fn Breakpoint<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: BreakpointEvent) {
        jvm.config.tracing.trace_event_trigger("Breakpoint");
        let jvmti_env = get_jvmti_interface(jvm, int_state);
        let jni_env = get_interface(jvm, todo!()/*int_state*/);
        let BreakpointEvent { thread, method, location } = event;
        let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"Breakpoint")*/);
        let guard = self.breakpoint_callback.read().unwrap();
        let func_pointer = guard.as_ref().unwrap();
        (func_pointer)(jvmti_env, jni_env, thread, method, location);
        int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending excpetion anyway
    }

    fn Breakpoint_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.breakpoint_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "Breakpoint")
    }

    fn Breakpoint_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.breakpoint_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "Breakpoint")
    }

    unsafe fn FramePop<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, event: FramePopEvent) {
        jvm.config.tracing.trace_event_trigger("FramePop");
        //todo dup with above
        let jvmti_env = get_jvmti_interface(jvm, int_state);
        let jni_env = get_interface(jvm, todo!()/*int_state*/);
        let FramePopEvent { thread, method, was_popped_by_exception } = event;
        let frame_for_event = int_state.push_frame(todo!()/*StackEntryPush::new_completely_opaque_frame(jvm,int_state.current_loader(jvm), vec![],"FramePop")*/);
        let guard = self.frame_pop_callback.read().unwrap();
        let func_pointer = guard.as_ref().unwrap();
        (func_pointer)(jvmti_env, jni_env, thread, method, was_popped_by_exception);
        int_state.pop_frame(jvm, frame_for_event, false); //todo check for pending exception anyway
    }

    fn FramePop_enable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let enabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.breakpoint_enabled = true;
        };
        SharedLibJVMTI::enable_impl(jvm, tid, &enabler, "Breakpoint")
    }

    fn FramePop_disable<'gc>(&self, jvm: &'gc JVMState<'gc>, tid: Option<Arc<JavaThread<'gc>>>) {
        let disabler = |jvmti_event_status: &mut ThreadJVMTIEnabledStatus| {
            jvmti_event_status.breakpoint_enabled = false;
        };
        SharedLibJVMTI::disable_impl(jvm, tid, &disabler, "Breakpoint")
    }
}

impl SharedLibJVMTI {
    pub fn agent_load<'gc, 'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> jvmtiError {
        unsafe {
            let agent_load_symbol = self.lib.get::<fn(vm: *mut JavaVM, options: *mut c_char, reserved: *mut c_void) -> jint>("Agent_OnLoad".as_bytes()).unwrap();
            let agent_load_fn_ptr = agent_load_symbol.deref();
            let args = CString::new("transport=dt_socket,server=y,suspend=y,address=5005").unwrap().into_raw(); //todo parse these at jvm startup
            let interface: *const JNIInvokeInterface_ = get_invoke_interface(jvm, int_state);
            agent_load_fn_ptr(Box::leak(Box::new(interface)) as *mut *const JNIInvokeInterface_, args, std::ptr::null_mut()) as jvmtiError
            //todo leak
        }
    }
}

impl SharedLibJVMTI {
    pub fn load_libjdwp(jdwp_path: &OsString) -> SharedLibJVMTI {
        SharedLibJVMTI {
            lib: Arc::new(Library::new(jdwp_path, RTLD_NOW).unwrap()),
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
    let tracing_guard = jvm.config.tracing.trace_jdwp_function_enter(jvm, "SetEventCallbacks");
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
        VMObjectAlloc,
    } = callback_copy;

    if VMInit.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.vm_init_callback.write().unwrap() = VMInit;
    }
    if VMDeath.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.vm_death_callback.write().unwrap() = VMDeath;
    }
    if ThreadStart.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.thread_start_callback.write().unwrap() = ThreadStart;
    }
    if ThreadEnd.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.thread_end_callback.write().unwrap() = ThreadEnd;
    }
    if ClassFileLoadHook.is_some() {
        unimplemented!()
    }
    if ClassLoad.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.class_load_callback.write().unwrap() = ClassLoad;
    }
    if ClassPrepare.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.class_prepare_callback.write().unwrap() = ClassPrepare;
    }
    if VMStart.is_some() {
        unimplemented!()
    }
    if Exception.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.exception_callback.write().unwrap() = Exception;
    }
    if ExceptionCatch.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.exception_catch_callback.write().unwrap() = ExceptionCatch;
    }
    if SingleStep.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.single_step_callback.write().unwrap() = SingleStep;
    }
    if FramePop.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.frame_pop_callback.write().unwrap() = FramePop;
    }
    if Breakpoint.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.breakpoint_callback.write().unwrap() = Breakpoint;
    }
    if FieldAccess.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.field_access_callback.write().unwrap() = FieldAccess;
    }
    if FieldModification.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.field_modification_callback.write().unwrap() = FieldModification;
    }
    if MethodEntry.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.method_entry_callback.write().unwrap() = MethodEntry;
    }
    if MethodExit.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.method_exit_callback.write().unwrap() = MethodExit;
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
        *jvm.jvmti_state().unwrap().built_in_jdwp.monitor_wait_callback.write().unwrap() = MonitorWait;
    }
    if MonitorWaited.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.monitor_waited_callback.write().unwrap() = MonitorWaited;
    }
    if MonitorContendedEnter.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.monitor_conteded_enter_callback.write().unwrap() = MonitorContendedEnter;
    }
    if MonitorContendedEntered.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.monitor_conteded_entered_callback.write().unwrap() = MonitorContendedEntered;
    }
    if ResourceExhausted.is_some() {
        unimplemented!()
    }
    if GarbageCollectionStart.is_some() {
        unimplemented!()
    }
    if GarbageCollectionFinish.is_some() {
        *jvm.jvmti_state().unwrap().built_in_jdwp.garbage_collection_finish_callback.write().unwrap() = GarbageCollectionFinish;
    }
    if ObjectFree.is_some() {
        //todo no gc, ignore
        // unimplemented!()
    }
    if VMObjectAlloc.is_some() {
        unimplemented!()
    }
    jvm.config.tracing.trace_jdwp_function_exit(tracing_guard, jvmtiError_JVMTI_ERROR_NONE)
}

impl SharedLibJVMTI {
    //todo these are essentially the same merge into one?
    fn disable_impl<'gc>(jvm: &'gc JVMState<'gc>, java_thread: Option<Arc<JavaThread<'gc>>>, disabler: &dyn Fn(&mut ThreadJVMTIEnabledStatus), event_name: &str) {
        jvm.config.tracing.trace_event_disable_global(event_name);
        match java_thread {
            None => {
                for java_thread in jvm.thread_state.all_java_threads.read().unwrap().values() {
                    disabler(&mut java_thread.jvmti_event_status_mut());
                }
            }
            Some(java_thread) => {
                disabler(&mut java_thread.jvmti_event_status_mut());
            }
        }
    }

    fn enable_impl<'gc>(jvm: &'gc JVMState<'gc>, java_thread: Option<Arc<JavaThread<'gc>>>, enabler: &dyn Fn(&mut ThreadJVMTIEnabledStatus), event_name: &str) {
        jvm.config.tracing.trace_event_enable_global(event_name);
        match java_thread {
            None => {
                for java_thread in jvm.thread_state.all_java_threads.read().unwrap().values() {
                    enabler(&mut java_thread.jvmti_event_status_mut());
                }
            }
            Some(java_thread) => {
                enabler(&mut java_thread.jvmti_event_status_mut());
            }
        }
    }
}