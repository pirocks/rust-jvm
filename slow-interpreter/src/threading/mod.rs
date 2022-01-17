use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::transmute;
use std::ops::Deref;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Sender};
use std::thread::LocalKey;
use std::time::Duration;

use crossbeam::thread::Scope;
use libloading::Symbol;
use num::Integer;
use wtf8::Wtf8Buf;
use another_jit_vm_ir::ir_stack::IRStackMut;

use jvmti_jni_bindings::*;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::JavaThreadId;
use rust_jvm_common::loading::LoaderName;
use threads::{Thread, Threads};

use crate::{InterpreterStateGuard, JVMState, run_main, set_properties};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class, check_loaded_class};
use crate::interpreter::{run_function, safepoint_check, WasException};
use crate::interpreter_state::{CURRENT_INT_STATE_GUARD, CURRENT_INT_STATE_GUARD_VALID, InterpreterState};
use crate::interpreter_util::new_object;
use crate::invoke_interface::get_invoke_interface;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java::lang::thread::JThread;
use crate::java::lang::thread_group::JThreadGroup;
use crate::java_values::JavaValue;
use crate::jit_common::java_stack::JavaStatus;
use crate::jvmti::event_callbacks::ThreadJVMTIEnabledStatus;
use crate::stack_entry::StackEntry;
use crate::threading::safepoints::{Monitor2, SafePoint};

pub struct ThreadState<'gc_life> {
    pub threads: Threads<'gc_life>,
    main_thread: RwLock<Option<Arc<JavaThread<'gc_life>>>>,
    pub(crate) all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread<'gc_life>>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread<'static>>>>>,
    pub system_thread_group: RwLock<Option<JThreadGroup<'gc_life>>>,
    monitors: RwLock<Vec<Arc<Monitor2>>>,
    pub(crate) int_state_guard: &'static LocalKey<RefCell<Option<*mut InterpreterStateGuard<'static,'static>>>>,
    pub(crate) int_state_guard_valid: &'static LocalKey<RefCell<bool>>,
}

pub struct MainThreadStartInfo {
    pub args: Vec<String>,
}

impl<'gc_life> ThreadState<'gc_life> {
    pub fn new(scope: Scope<'gc_life>) -> Self {
        Self {
            threads: Threads::new(scope),
            main_thread: RwLock::new(None),
            all_java_threads: RwLock::new(HashMap::new()),
            current_java_thread: &CURRENT_JAVA_THREAD,
            system_thread_group: RwLock::new(None),
            monitors: RwLock::new(vec![]),
            int_state_guard: &CURRENT_INT_STATE_GUARD,
            int_state_guard_valid: &CURRENT_INT_STATE_GUARD_VALID,
        }
    }

    pub fn setup_main_thread(&self, jvm: &'gc_life JVMState<'gc_life>, main_thread: &'gc_life Arc<JavaThread<'gc_life>>) -> Sender<MainThreadStartInfo> {
        *self.main_thread.write().unwrap() = main_thread.clone().into();
        let (main_send, main_recv) = channel();
        main_thread.clone().underlying_thread.start_thread(
            box move |_| {
                jvm.thread_state.set_current_thread(main_thread.clone());
                main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
                // assert!(match main_thread.interpreter_state.read().unwrap().deref() {
                //     InterpreterState::LegacyInterpreter { call_stack, .. } => call_stack.is_empty(),
                //     InterpreterState::Jit { .. } => {}//todo!()
                // });
                let mut int_state = InterpreterStateGuard::new(jvm, main_thread.clone(), todo!()/*main_thread.interpreter_state.write().unwrap().into()*/);
                main_thread.notify_alive(jvm); //is this too early?
                int_state.register_interpreter_state_guard(jvm);
                jvm.jvmti_state().map(|jvmti| jvmti.built_in_jdwp.agent_load(jvm, &mut int_state)); // technically this is to late and should have been called earlier, but needs to be on this thread.
                ThreadState::jvm_init_from_main_thread(jvm, &mut int_state);

                assert!(!jvm.live.load(Ordering::SeqCst));
                jvm.live.store(true, Ordering::SeqCst);
                if let Some(jvmti) = jvm.jvmti_state() {
                    jvmti.built_in_jdwp.vm_inited(jvm, &mut int_state, main_thread.clone())
                }
                let MainThreadStartInfo { args } = main_recv.recv().unwrap();
                //from the jvmti spec:
                //"The thread start event for the main application thread is guaranteed not to occur until after the handler for the VM initialization event returns. "
                if let Some(jvmti) = jvm.jvmti_state() {
                    jvmti.built_in_jdwp.thread_start(jvm, &mut int_state, main_thread.thread_object())
                }
                let push_guard = int_state.push_frame(StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader, vec![])); //todo think this is correct, check
                //handle any excpetions from here
                int_state.pop_frame(jvm, push_guard, false);
                let main_frame_guard = int_state.push_frame(StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader, vec![]));
                run_main(args, jvm, &mut int_state).unwrap();
                //todo handle exception exit from main
                int_state.pop_frame(jvm, main_frame_guard, false);
                main_thread.notify_terminated(jvm)
            },
            box (),
        );
        main_send
    }

    fn jvm_init_from_main_thread(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) {
        let main_thread = jvm.thread_state.get_main_thread();
        main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        let system_class = assert_inited_or_initing_class(jvm, CClassName::system().into());

        let system = &system_class;
        let system_view = system.view();
        let method_views = system_view.lookup_method_name(MethodName::method_initializeSystemClass());
        let init_method_view = method_views.first().unwrap().clone();
        let mut locals = vec![];
        for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
            locals.push(JavaValue::Top);
        }
        let initialize_system_frame = StackEntry::new_java_frame(jvm, system_class.clone(), init_method_view.method_i() as u16, locals);
        let mut init_frame_guard = int_state.push_frame(initialize_system_frame);
        assert!(Arc::ptr_eq(&main_thread, &jvm.thread_state.get_current_thread()));
        match run_function(&jvm, int_state, &mut init_frame_guard) {
            Ok(_) => {}
            Err(_) => todo!(),
        }
        if int_state.function_return() {
            int_state.set_function_return(false);
        }
        if int_state.throw().is_some() {
            unimplemented!()
        }
        set_properties(jvm, int_state).expect("todo");
        //todo read and copy props here
        let key = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("java.home".to_string())).expect("todo");
        let value = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/".to_string())).expect("todo");
        System::props(jvm, int_state).set_property(jvm, int_state, key, value).expect("todo");

        //todo should handle excpetions here
        int_state.pop_frame(jvm, init_frame_guard, false);
        if !jvm.config.compiled_mode_active {
        }
    }

    pub fn get_main_thread(&self) -> Arc<JavaThread<'gc_life>> {
        self.main_thread.read().unwrap().as_ref().unwrap().clone()
    }

    pub(crate) fn set_current_thread(&'_ self, thread: Arc<JavaThread<'gc_life>>) {
        self.current_java_thread.with(|refcell| {
            assert!(refcell.borrow().is_none());
            unsafe {
                *refcell.borrow_mut() = transmute(Some(thread));
            }
        })
    }

    pub fn bootstrap_main_thread(jvm: &'vm_life JVMState<'vm_life>, threads: &'vm_life Threads<'vm_life>) -> Arc<JavaThread<'vm_life>> {
        let bootstrap_underlying_thread = threads.create_thread("Bootstrap Thread".to_string().into());
        let bootstrap_thread = Arc::new(JavaThread {
            java_tid: 0,
            underlying_thread: bootstrap_underlying_thread,
            thread_object: RwLock::new(None),
            interpreter_state: Mutex::new(InterpreterState::new(jvm)),
            invisible_to_java: true,
            jvmti_events_enabled: Default::default(),
            thread_local_storage: RwLock::new(null_mut()),
            safepoint_state: SafePoint::new(),
            thread_status: RwLock::new(ThreadStatus { terminated: false, alive: false, interrupted: false }),
        });
        jvm.thread_state.set_current_thread(bootstrap_thread.clone());
        bootstrap_thread.notify_alive(jvm);
        let mut interpreter_state_guard = bootstrap_thread.interpreter_state.lock().unwrap();
        let mut new_int_state = InterpreterStateGuard::LocalInterpreterState {
            int_state: IRStackMut::from_stack_start(&mut interpreter_state_guard.call_stack.inner),
            thread: jvm.thread_state.get_current_thread(),
            registered: false,
            jvm
        };
        new_int_state.register_interpreter_state_guard(jvm);
        unsafe {
            jvm.native_libaries.load(jvm, &mut new_int_state, &jvm.native_libaries.libjava_path, "java".to_string());
            {
                let native_libs_guard = jvm.native_libaries.native_libs.read().unwrap();
                let libjava_native_lib = native_libs_guard.get("java").unwrap();
                let setup_hack_symbol: Symbol<unsafe extern "system" fn(*const JNIInvokeInterface_)> = libjava_native_lib.library.get("setup_jvm_pointer_hack".as_bytes()).unwrap();
                (*setup_hack_symbol.deref())(get_invoke_interface(jvm, &mut new_int_state))
            }
        }
        let frame = StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader, vec![]);
        let frame_for_bootstrapping = new_int_state.push_frame(frame);
        let object_rc = check_loaded_class(jvm, &mut new_int_state, CClassName::object().into()).expect("This should really never happen, since it is equivalent to a class not found exception on java/lang/Object");
        jvm.verify_class_and_object(object_rc, jvm.classes.read().unwrap().class_class.clone());
        let thread_classfile = check_initing_or_inited_class(jvm, &mut new_int_state, CClassName::thread().into()).expect("couldn't load thread class");

        let thread_object = new_object(jvm, &mut new_int_state, &thread_classfile).cast_thread();
        thread_object.set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        *bootstrap_thread.thread_object.write().unwrap() = thread_object.into();
        let thread_group_class = check_initing_or_inited_class(jvm, &mut new_int_state, CClassName::thread_group().into()).expect("couldn't load thread group class");
        let system_thread_group = JThreadGroup::init(jvm, &mut new_int_state, thread_group_class).expect("todo");
        *jvm.thread_state.system_thread_group.write().unwrap() = system_thread_group.clone().into();
        let main_jthread = JThread::new(jvm, &mut new_int_state, system_thread_group, "Main".to_string()).expect("todo");
        new_int_state.pop_frame(jvm, frame_for_bootstrapping, false);
        bootstrap_thread.notify_terminated(jvm);
        JavaThread::new(jvm, main_jthread, threads.create_thread("Main Java Thread".to_string().into()), false)
    }

    pub fn get_current_thread_name(&self, jvm: &'gc_life JVMState<'gc_life>) -> String {
        let current_thread = self.get_current_thread();
        let thread_object = current_thread.thread_object.read().unwrap();
        thread_object.as_ref().map(|jthread| jthread.name(jvm).to_rust_string(jvm)).unwrap_or(std::thread::current().name().unwrap_or("unknown").to_string())
    }

    pub fn try_get_current_thread(&self) -> Option<Arc<JavaThread<'gc_life>>> {
        self.current_java_thread.with(|thread_refcell| unsafe { transmute(thread_refcell.borrow().clone()) })
    }

    pub fn new_monitor(&self, _name: String) -> Arc<Monitor2> {
        let mut monitor_guard = self.monitors.write().unwrap();
        let index = monitor_guard.len();
        let res = Arc::new(Monitor2::new(index));
        monitor_guard.push(res.clone());
        res
    }

    pub fn get_current_thread(&self) -> Arc<JavaThread<'gc_life>> {
        self.try_get_current_thread().unwrap()
    }

    pub fn get_current_thread_tid_or_invalid(&self) -> jlong {
        match self.try_get_current_thread() {
            None => -1,
            Some(current_thread) => current_thread.java_tid,
        }
    }

    pub fn get_monitor(&self, monitor: jrawMonitorID) -> Arc<Monitor2> {
        self.try_get_monitor(monitor).unwrap()
    }

    pub fn try_get_monitor(&self, monitor: jrawMonitorID) -> Option<Arc<Monitor2>> {
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard.get(monitor as usize).cloned();
        std::mem::drop(monitors_read_guard);
        monitor
    }

    pub fn get_thread_by_tid(&self, tid: JavaThreadId) -> Arc<JavaThread<'gc_life>> {
        self.try_get_thread_by_tid(tid).unwrap()
    }

    pub fn try_get_thread_by_tid(&self, tid: JavaThreadId) -> Option<Arc<JavaThread<'gc_life>>> {
        self.all_java_threads.read().unwrap().get(&tid).cloned()
    }

    pub fn start_thread_from_obj(&'gc_life self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, obj: JThread<'gc_life>, invisible_to_java: bool) -> Arc<JavaThread<'gc_life>> {
        let underlying = self.threads.create_thread(obj.name(jvm).to_rust_string(jvm).into());

        let (send, recv) = channel();
        let java_thread: Arc<JavaThread<'gc_life>> = JavaThread::new(jvm, obj.clone(), underlying, invisible_to_java);
        let loader_name = obj.get_context_class_loader(jvm, int_state).expect("todo").map(|class_loader| class_loader.to_jvm_loader(jvm)).unwrap_or(LoaderName::BootstrapLoader);
        java_thread.clone().underlying_thread.start_thread(
            box move |_data| {
                send.send(java_thread.clone()).unwrap();
                jvm.thread_state.set_current_thread(java_thread.clone());
                java_thread.notify_alive(jvm);
                Self::foo(jvm, java_thread, loader_name)
            },
            box (),
        ); //todo is this Data really needed since we have a closure
        recv.recv().unwrap()
    }

    fn foo<'l>(jvm: &'gc_life JVMState<'gc_life>, java_thread: Arc<JavaThread<'gc_life>>, loader_name: LoaderName) {
        let java_thread_clone: Arc<JavaThread<'gc_life>> = java_thread.clone();
        // let option: Option<RwLockWriteGuard<'_, InterpreterState>> = java_thread.interpreter_state.write().unwrap().into();
        let state = java_thread_clone.interpreter_state.lock().unwrap().into();
        let mut interpreter_state_guard = InterpreterStateGuard::new(jvm, java_thread_clone.clone(), state); // { int_state: , thread: &java_thread };
        interpreter_state_guard.register_interpreter_state_guard(jvm);

        if let Some(jvmti) = jvm.jvmti_state() {
            jvmti.built_in_jdwp.thread_start(jvm, &mut interpreter_state_guard, java_thread.clone().thread_object())
        }

        let frame_for_run_call = interpreter_state_guard.push_frame(StackEntry::new_completely_opaque_frame(loader_name, vec![]));
        if let Err(WasException {}) = java_thread.thread_object.read().unwrap().as_ref().unwrap().run(jvm, &mut interpreter_state_guard) {
            JavaValue::Object(todo!() /*interpreter_state_guard.throw()*/).cast_throwable().print_stack_trace(jvm, &mut interpreter_state_guard).expect("Exception occured while printing exception. Something is pretty messed up");
            interpreter_state_guard.set_throw(None);
        };
        if let Err(WasException {}) = java_thread.thread_object.read().unwrap().as_ref().unwrap().exit(jvm, &mut interpreter_state_guard) {
            eprintln!("Exception occured exiting thread, something is pretty messed up");
            panic!()
        }

        interpreter_state_guard.pop_frame(jvm, frame_for_run_call, false);
        java_thread.notify_terminated(jvm);
    }

    pub fn get_all_threads(&self) -> RwLockReadGuard<HashMap<JavaThreadId, Arc<JavaThread<'gc_life>>>> {
        self.all_java_threads.read().unwrap()
    }

    pub fn get_all_alive_threads(&self) -> Vec<Arc<JavaThread<'gc_life>>> {
        self.all_java_threads
            .read()
            .unwrap()
            .values()
            .filter(|_thread| {
                //don't use is_alive for this
                // todo!()
                true
                // thread.thread_object().is_alive(jvm, int_state) != 0
            })
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn get_system_thread_group(&self) -> JThreadGroup<'gc_life> {
        self.system_thread_group.read().unwrap().as_ref().unwrap().clone()
    }
}

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread<'static>>>> = RefCell::new(None);
}


pub struct JavaThread<'vm_life> {
    pub java_tid: JavaThreadId,
    underlying_thread: Thread<'vm_life>,
    thread_object: RwLock<Option<JThread<'vm_life>>>,
    pub interpreter_state: Mutex<InterpreterState<'vm_life>>,
    pub invisible_to_java: bool,
    jvmti_events_enabled: RwLock<ThreadJVMTIEnabledStatus>,
    pub thread_local_storage: RwLock<*mut c_void>,
    pub safepoint_state: SafePoint<'vm_life>,
    pub thread_status: RwLock<ThreadStatus>,
}

impl<'gc_life> JavaThread<'gc_life> {
    pub fn is_alive(&self) -> bool {
        self.thread_status.read().unwrap().alive
    }

    pub fn new(jvm: &'gc_life JVMState<'gc_life>, thread_obj: JThread<'gc_life>, underlying: Thread<'gc_life>, invisible_to_java: bool) -> Arc<JavaThread<'gc_life>> {
        let res = Arc::new(JavaThread {
            java_tid: thread_obj.tid(jvm),
            underlying_thread: underlying,
            thread_object: RwLock::new(thread_obj.into()),
            interpreter_state: Mutex::new(InterpreterState::new(jvm)),
            invisible_to_java,
            jvmti_events_enabled: RwLock::new(ThreadJVMTIEnabledStatus::default()),
            thread_local_storage: RwLock::new(null_mut()),
            safepoint_state: SafePoint::new(),
            thread_status: RwLock::new(ThreadStatus { terminated: false, alive: false, interrupted: false }),
        });
        jvm.thread_state.all_java_threads.write().unwrap().insert(res.java_tid, res.clone());
        res
    }

    pub fn jvmti_event_status(&self) -> RwLockReadGuard<ThreadJVMTIEnabledStatus> {
        self.jvmti_events_enabled.read().unwrap()
    }

    pub fn jvmti_event_status_mut(&self) -> RwLockWriteGuard<ThreadJVMTIEnabledStatus> {
        self.jvmti_events_enabled.write().unwrap()
    }

    pub fn get_underlying(&self) -> &Thread<'gc_life> {
        &self.underlying_thread
    }

    pub fn thread_object(&self) -> JThread<'gc_life> {
        self.try_thread_object().unwrap()
    }

    pub fn try_thread_object(&self) -> Option<JThread<'gc_life>> {
        self.thread_object.read().unwrap().clone()
    }

    pub fn notify_alive(&self, jvm: &'gc_life JVMState<'gc_life>) {
        let mut status = self.thread_status.write().unwrap();
        status.alive = true;
        self.update_thread_object(jvm, status)
    }

    fn update_thread_object(&self, jvm: &'gc_life JVMState<'gc_life>, status: RwLockWriteGuard<ThreadStatus>) {
        if self.thread_object.read().unwrap().is_some() {
            let obj = self.thread_object();
            obj.set_thread_status(jvm, self.safepoint_state.get_thread_status_number(status.deref()))
        }
    }

    pub fn notify_terminated(&self, jvm: &'gc_life JVMState<'gc_life>) {
        let mut status = self.thread_status.write().unwrap();

        status.terminated = true;
        self.update_thread_object(jvm, status)
    }

    pub fn status_number(&self) -> jint {
        let status_guard = self.thread_status.read().unwrap();
        self.safepoint_state.get_thread_status_number(status_guard.deref())
    }

    pub fn park(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, time_nanos: Option<u128>) -> Result<(), WasException> {
        unsafe { assert!(self.underlying_thread.is_this_thread()) }
        const NANOS_PER_SEC: u128 = 1_000_000_000u128;
        self.safepoint_state.set_park(time_nanos.map(|time_nanos| {
            let (secs, nanos) = time_nanos.div_mod_floor(&NANOS_PER_SEC);
            Duration::new(secs as u64, nanos as u32)
        }));
        self.safepoint_state.check(jvm, int_state)
    }

    pub fn unpark(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) -> Result<(), WasException> {
        self.safepoint_state.set_unpark();
        self.safepoint_state.check(jvm, int_state)
    }

    pub unsafe fn gc_suspend(&self) {
        self.safepoint_state.set_gc_suspended().unwrap(); //todo should use gc flag for this
    }

    pub unsafe fn suspend_thread(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, without_self_suspend: bool) -> Result<(), SuspendError> {
        if !self.is_alive() {
            return Err(SuspendError::NotAlive);
        }
        self.safepoint_state.set_suspended()?;
        if self.underlying_thread.is_this_thread() {
            assert_eq!(self.java_tid, int_state.thread().java_tid);
            if !without_self_suspend {
                safepoint_check(jvm, int_state)?;
            }
        }
        Ok(())
    }

    pub unsafe fn resume_thread(&self) -> Result<(), ResumeError> {
        self.safepoint_state.set_unsuspended()
    }

    pub unsafe fn gc_resume_thread(&self) -> Result<(), ResumeError> {
        self.safepoint_state.set_gc_unsuspended()
    }

    pub fn is_this_thread(&self) -> bool {
        unsafe { self.underlying_thread.is_this_thread() }
    }
}

#[derive(Debug)]
pub struct ThreadStatus {
    pub terminated: bool,
    pub alive: bool,
    pub interrupted: bool,
}

#[derive(Debug)]
pub enum SuspendError {
    AlreadySuspended,
    NotAlive,
    WasException(WasException),
}

#[derive(Debug)]
pub enum ResumeError {
    NotSuspended,
}

impl From<WasException> for SuspendError {
    fn from(we: WasException) -> Self {
        Self::WasException(we)
    }
}

pub mod monitors;
pub mod safepoints;