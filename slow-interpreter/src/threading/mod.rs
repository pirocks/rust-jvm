use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, Condvar, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Sender};
use std::thread::LocalKey;
use std::time::Duration;

use classfile_view::loading::LoaderName;
use jvmti_jni_bindings::*;
use rust_jvm_common::classnames::ClassName;
use threads::{Thread, Threads};

use crate::{InterpreterStateGuard, JVMState, locate_init_system_class, run_main, set_properties};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter::{run_function, suspend_check};
use crate::interpreter_state::{CURRENT_INT_STATE_GUARD, CURRENT_INT_STATE_GUARD_VALID, InterpreterState, SuspendedStatus};
use crate::interpreter_util::push_new_object;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java::lang::thread::JThread;
use crate::java::lang::thread_group::JThreadGroup;
use crate::java_values::JavaValue;
use crate::jvmti::event_callbacks::ThreadJVMTIEnabledStatus;
use crate::stack_entry::StackEntry;
use crate::threading::monitors::Monitor;
use crate::threading::ResumeError::NotSuspended;

pub struct ThreadState {
    pub(crate) threads: Threads,
    main_thread: RwLock<Option<Arc<JavaThread>>>,
    pub(crate) all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread>>>>,
    pub system_thread_group: RwLock<Option<JThreadGroup>>,
    monitors: RwLock<Vec<Arc<Monitor>>>,
    pub(crate) int_state_guard: &'static LocalKey<RefCell<Option<*mut InterpreterStateGuard<'static>>>>,
    pub(crate) int_state_guard_valid: &'static LocalKey<RefCell<bool>>,
}


pub struct MainThreadStartInfo {
    pub args: Vec<String>
}

impl ThreadState {
    pub fn new() -> Self {
        Self {
            threads: Threads::new(),
            main_thread: RwLock::new(None),
            all_java_threads: RwLock::new(HashMap::new()),
            current_java_thread: &CURRENT_JAVA_THREAD,
            system_thread_group: RwLock::new(None),
            monitors: RwLock::new(vec![]),
            int_state_guard: &CURRENT_INT_STATE_GUARD,
            int_state_guard_valid: &CURRENT_INT_STATE_GUARD_VALID,
        }
    }

    pub fn setup_main_thread(&'static self, jvm: &'static JVMState) -> (Arc<JavaThread>, Sender<MainThreadStartInfo>) {
        let main_thread = ThreadState::bootstrap_main_thread(jvm, &jvm.thread_state.threads);
        *self.main_thread.write().unwrap() = main_thread.clone().into();
        let (main_send, main_recv) = channel();
        let main_thread_clone = main_thread.clone();
        main_thread.clone().underlying_thread.start_thread(box move |_| {
            jvm.thread_state.set_current_thread(main_thread.clone());
            main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
            assert!(main_thread.interpreter_state.read().unwrap().call_stack.is_empty());
            let mut int_state = InterpreterStateGuard::new(jvm, &main_thread);
            main_thread.notify_alive();//is this too early?
            int_state.register_interpreter_state_guard(jvm);
            jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.agent_load(jvm, &mut int_state));// technically this is to late and should have been called earlier, but needs to be on this thread.
            ThreadState::jvm_init_from_main_thread(jvm, &mut int_state);

            assert!(!jvm.live.load(Ordering::SeqCst));
            jvm.live.store(true, Ordering::SeqCst);
            if let Some(jvmti) = jvm.jvmti_state.as_ref() {
                jvmti.built_in_jdwp.vm_inited(jvm, &mut int_state, main_thread.clone())
            }
            let MainThreadStartInfo { args } = main_recv.recv().unwrap();
            //from the jvmti spec:
            //"The thread start event for the main application thread is guaranteed not to occur until after the handler for the VM initialization event returns. "
            if let Some(jvmti) = jvm.jvmti_state.as_ref() {
                jvmti.built_in_jdwp.thread_start(jvm, &mut int_state, main_thread.thread_object())
            }
            let push_guard = int_state.push_frame(StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader));//todo think this is correct, check
            unsafe { jvm.libjava.load(jvm, &mut int_state); }//todo not sure if this should be here
            //handle any excpetions from here
            int_state.pop_frame(jvm, push_guard, false);
            let main_frame_guard = int_state.push_frame(StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader));
            run_main(args, jvm, &mut int_state).unwrap();
            //todo handle exception exit from main
            int_state.pop_frame(jvm, main_frame_guard, false);
            main_thread.notify_terminated()
        }, box ());
        (main_thread_clone, main_send)
    }

    fn jvm_init_from_main_thread(jvm: &JVMState, int_state: &mut InterpreterStateGuard) {
        let main_thread = jvm.thread_state.get_main_thread();
        main_thread.thread_object.read().unwrap().as_ref().unwrap().set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        let system_class = assert_inited_or_initing_class(jvm, int_state, ClassName::system().into());

        let init_method_view = locate_init_system_class(&system_class);
        let mut locals = vec![];
        for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
            locals.push(JavaValue::Top);
        }
        let initialize_system_frame = StackEntry::new_java_frame(jvm, system_class.clone(), init_method_view.method_i() as u16, locals);
        let init_frame_guard = int_state.push_frame(initialize_system_frame);
        assert!(Arc::ptr_eq(&main_thread, &jvm.thread_state.get_current_thread()));
        match run_function(&jvm, int_state) {
            Ok(_) => {}
            Err(_) => todo!()
        }
        let function_return = int_state.function_return_mut();
        if *function_return {
            *function_return = false;
        }
        if int_state.throw().is_some() {
            unimplemented!()
        }
        set_properties(jvm, int_state);
        //todo read and copy props here
        let key = JString::from_rust(jvm, int_state, "java.home".to_string()).expect("todo");
        let value = JString::from_rust(jvm, int_state, "/home/francis/build/openjdk-debug/jdk8u/build/linux-x86_64-normal-server-slowdebug/jdk/".to_string()).expect("todo");
        System::props(jvm, int_state).set_property(jvm, int_state, key, value).expect("todo");

        //todo should handle excpetions here
        int_state.pop_frame(jvm, init_frame_guard, false);
    }

    pub fn get_main_thread(&self) -> Arc<JavaThread> {
        self.main_thread.read().unwrap().as_ref().unwrap().clone()
    }

    pub(crate) fn set_current_thread(&'static self, thread: Arc<JavaThread>) {
        self.current_java_thread.with(|refcell| {
            assert!(refcell.borrow().is_none());
            *refcell.borrow_mut() = thread.into();
        })
    }

    fn bootstrap_main_thread(jvm: &'static JVMState, threads: &Threads) -> Arc<JavaThread> {
        let bootstrap_underlying_thread = threads.create_thread("Bootstrap Thread".to_string().into());
        let bootstrap_thread = Arc::new(JavaThread {
            java_tid: 0,
            underlying_thread: bootstrap_underlying_thread,
            thread_object: RwLock::new(None),
            interpreter_state: RwLock::new(InterpreterState::default()),
            suspended: SuspendedStatus::default(),
            invisible_to_java: true,
            jvmti_events_enabled: Default::default(),
            status: Default::default(),
            thread_local_storage: RwLock::new(null_mut()),
            await_on_park: Condvar::new(),
            num_parks: Mutex::new(0),
        });
        jvm.thread_state.set_current_thread(bootstrap_thread.clone());
        bootstrap_thread.notify_alive();
        let mut new_int_state = InterpreterStateGuard::new(jvm, &bootstrap_thread);
        new_int_state.register_interpreter_state_guard(jvm);
        let frame = StackEntry::new_completely_opaque_frame(LoaderName::BootstrapLoader);
        let frame_for_bootstrapping = new_int_state.push_frame(frame);

        let thread_classfile = check_initing_or_inited_class(jvm, &mut new_int_state, ClassName::thread().into()).expect("couldn't load thread class");

        push_new_object(jvm, &mut new_int_state, &thread_classfile);
        let thread_object = new_int_state.pop_current_operand_stack().cast_thread();
        thread_object.set_priority(JVMTI_THREAD_NORM_PRIORITY as i32);
        *bootstrap_thread.thread_object.write().unwrap() = thread_object.into();
        let thread_group_class = check_initing_or_inited_class(jvm, &mut new_int_state, ClassName::Str("java/lang/ThreadGroup".to_string()).into()).expect("couldn't load thread group class");
        let system_thread_group = JThreadGroup::init(jvm, &mut new_int_state, thread_group_class).expect("todo");
        *jvm.thread_state.system_thread_group.write().unwrap() = system_thread_group.clone().into();
        let main_jthread = JThread::new(jvm, &mut new_int_state, system_thread_group, "Main".to_string()).expect("todo");
        new_int_state.pop_frame(jvm, frame_for_bootstrapping, false);
        bootstrap_thread.notify_terminated();
        JavaThread::new(jvm, main_jthread, threads.create_thread("Main Java Thread".to_string().into()), false)
    }

    pub fn get_current_thread_name(&self) -> String {
        let current_thread = self.get_current_thread();
        let thread_object = current_thread.thread_object.read().unwrap();
        thread_object.as_ref().map(|jthread| jthread.name().to_rust_string())
            .unwrap_or(std::thread::current().name().unwrap_or("unknown").to_string())
    }

    pub fn try_get_current_thread(&self) -> Option<Arc<JavaThread>> {
        self.current_java_thread.with(|thread_refcell| {
            thread_refcell.borrow().clone()
        })
    }

    pub fn new_monitor(&self, name: String) -> Arc<Monitor> {
        let mut monitor_guard = self.monitors.write().unwrap();
        let index = monitor_guard.len();
        let res = Arc::new(Monitor::new(name, index));
        monitor_guard.push(res.clone());
        res
    }

    pub fn get_current_thread(&self) -> Arc<JavaThread> {
        self.try_get_current_thread().unwrap()
    }

    pub fn get_monitor(&self, monitor: jrawMonitorID) -> Arc<Monitor> {
        self.try_get_monitor(monitor).unwrap()
    }

    pub fn try_get_monitor(&self, monitor: jrawMonitorID) -> Option<Arc<Monitor>> {
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard.get(monitor as usize).cloned();
        std::mem::drop(monitors_read_guard);
        monitor
    }

    pub fn get_thread_by_tid(&self, tid: JavaThreadId) -> Arc<JavaThread> {
        self.try_get_thread_by_tid(tid).unwrap()
    }

    pub fn try_get_thread_by_tid(&self, tid: JavaThreadId) -> Option<Arc<JavaThread>> {
        self.all_java_threads.read().unwrap().get(&tid).cloned()
    }

    pub fn start_thread_from_obj(&self, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, obj: JThread, invisible_to_java: bool) -> Arc<JavaThread> {
        let underlying = self.threads.create_thread(obj.name().to_rust_string().into());

        let (send, recv) = channel();
        let java_thread = JavaThread::new(jvm, obj.clone(), underlying, invisible_to_java);
        let loader_name = obj.get_context_class_loader(jvm, int_state).expect("todo").map(|class_loader| class_loader.to_jvm_loader(jvm)).unwrap_or(LoaderName::BootstrapLoader);
        java_thread.clone().underlying_thread.start_thread(box move |_data| {
            send.send(java_thread.clone()).unwrap();
            jvm.thread_state.set_current_thread(java_thread.clone());
            java_thread.notify_alive();
            let mut interpreter_state_guard = InterpreterStateGuard::new(jvm, &java_thread);// { int_state: java_thread.interpreter_state.write().unwrap().into(), thread: &java_thread };
            interpreter_state_guard.register_interpreter_state_guard(jvm);

            if let Some(jvmti) = jvm.jvmti_state.as_ref() {
                jvmti.built_in_jdwp.thread_start(jvm, &mut interpreter_state_guard, java_thread.clone().thread_object())
            }

            let frame_for_run_call = interpreter_state_guard.push_frame(StackEntry::new_completely_opaque_frame(loader_name));
            java_thread.thread_object.read().unwrap().as_ref().unwrap().run(jvm, &mut interpreter_state_guard);
            //todo handle thread exiting with exception
            interpreter_state_guard.pop_frame(jvm, frame_for_run_call, false);
            java_thread.notify_terminated();
        }, box ());//todo is this Data really needed since we have a closure
        recv.recv().unwrap()
    }


    pub fn get_all_threads(&self) -> RwLockReadGuard<HashMap<JavaThreadId, Arc<JavaThread>>> {
        self.all_java_threads.read().unwrap()
    }

    pub fn get_all_alive_threads(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Vec<Arc<JavaThread>> {
        self.all_java_threads.read().unwrap().values().filter(|thread| {
            //don't use is_alive for this
            todo!()
            // thread.thread_object().is_alive(jvm, int_state) != 0
        }).cloned().collect::<Vec<_>>()
    }

    pub fn get_system_thread_group(&self) -> JThreadGroup {
        self.system_thread_group.read().unwrap().as_ref().unwrap().clone()
    }
}

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread>>> = RefCell::new(None);
}

pub type JavaThreadId = i64;

#[derive(Debug)]
pub struct ThreadStatus {
    alive: bool,
    terminated: bool,
    runnable: bool,
    blocked_on_monitor_enter: bool,
    waiting: bool,
    waiting_indefinitely: bool,
    waiting_timeout: bool,
    sleeping: bool,
    in_object_wait: bool,
    parked: bool,
    // suspended: bool,
    interrupted: bool,
    //todo how to handle native?
}

impl ThreadStatus {
    fn get_thread_status_number(&self, thread: &JavaThread) -> jint {
        let mut res = 0;
        if self.alive {
            res |= JVMTI_THREAD_STATE_ALIVE;
        }
        if self.terminated {
            res |= JVMTI_THREAD_STATE_TERMINATED;
        }
        if self.runnable {
            res |= JVMTI_THREAD_STATE_RUNNABLE;
        }
        if self.blocked_on_monitor_enter {
            res |= JVMTI_THREAD_STATE_BLOCKED_ON_MONITOR_ENTER;
        }
        if self.waiting {
            res |= JVMTI_THREAD_STATE_WAITING;
        }
        if self.waiting_indefinitely {
            res |= JVMTI_THREAD_STATE_WAITING_INDEFINITELY;
        }
        if self.waiting_timeout {
            res |= JVMTI_THREAD_STATE_WAITING_WITH_TIMEOUT;
        }
        if self.sleeping {
            res |= JVMTI_THREAD_STATE_SLEEPING;
        }
        if self.in_object_wait {
            res |= JVMTI_THREAD_STATE_IN_OBJECT_WAIT;
        }
        if self.parked {
            res |= JVMTI_THREAD_STATE_PARKED;
        }
        if *thread.suspended.suspended.lock().unwrap() {
            res |= JVMTI_THREAD_STATE_SUSPENDED;
        }
        if self.interrupted {
            res |= JVMTI_THREAD_STATE_INTERRUPTED;
        }
        res as jint
    }
}

impl Default for ThreadStatus {
    fn default() -> Self {
        Self {
            alive: false,
            terminated: false,
            runnable: false,
            blocked_on_monitor_enter: false,
            waiting: false,
            waiting_indefinitely: false,
            waiting_timeout: false,
            sleeping: false,
            in_object_wait: false,
            parked: false,
            // suspended: false,
            interrupted: false,
        }
    }
}

#[derive(Debug)]
pub struct JavaThread {
    pub java_tid: JavaThreadId,
    underlying_thread: Thread,
    thread_object: RwLock<Option<JThread>>,
    pub interpreter_state: RwLock<InterpreterState>,
    pub suspended: SuspendedStatus,
    pub invisible_to_java: bool,
    jvmti_events_enabled: RwLock<ThreadJVMTIEnabledStatus>,
    status: RwLock<ThreadStatus>,
    pub thread_local_storage: RwLock<*mut c_void>,
    await_on_park: Condvar,
    num_parks: Mutex<isize>,
}

impl JavaThread {
    pub fn new(jvm: &JVMState, thread_obj: JThread, underlying: Thread, invisible_to_java: bool) -> Arc<JavaThread> {
        let res = Arc::new(JavaThread {
            java_tid: thread_obj.tid(),
            underlying_thread: underlying,
            thread_object: RwLock::new(thread_obj.into()),
            interpreter_state: RwLock::new(InterpreterState::default()),
            suspended: SuspendedStatus::default(),
            invisible_to_java,
            jvmti_events_enabled: RwLock::new(ThreadJVMTIEnabledStatus::default()),
            status: Default::default(),
            thread_local_storage: RwLock::new(null_mut()),
            await_on_park: Condvar::new(),
            num_parks: Mutex::new(0),
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

    pub fn get_underlying(&self) -> &Thread {
        &self.underlying_thread
    }

    pub fn thread_object(&self) -> JThread {
        self.try_thread_object().unwrap()
    }

    pub fn try_thread_object(&self) -> Option<JThread> {
        self.thread_object.read().unwrap().clone()
    }

    pub fn notify_alive(&self) {
        let mut status = self.status.write().unwrap();
        status.alive = true;
        status.runnable = true;//when a thread becomes alive it defaults to runnable
        if self.thread_object.read().unwrap().is_some() {
            let obj = self.thread_object();
            obj.set_thread_status(status.get_thread_status_number(self))
        }
    }

    pub fn is_alive(&self) -> bool {
        self.status.read().unwrap().alive
    }

    pub fn is_interrupted(&self) -> bool {
        self.status.read().unwrap().interrupted //todo should probably query suspended status as well
    }


    pub fn notify_terminated(&self) {
        let mut status = self.status.write().unwrap();

        status.alive = false;
        // status.suspended = false;
        status.interrupted = false;
        status.runnable = false;
        status.blocked_on_monitor_enter = false;
        status.waiting = false;
        status.waiting_indefinitely = false;
        status.waiting_timeout = false;
        status.in_object_wait = false;
        status.parked = false;
        status.sleeping = false;
        status.terminated = true;

        if self.thread_object.read().unwrap().is_some() {
            let obj = self.thread_object();
            obj.set_thread_status(status.get_thread_status_number(self))
        }//todo duplication
    }

    pub fn status_number(&self) -> jint {
        self.status.read().unwrap().get_thread_status_number(self)
    }

    pub fn park(&self, time_nanos: u64) {//perhaps this u64 should be a u128 but I believe its fine within my lifetime. Also why would you park a thread that long.
        unsafe { assert!(self.underlying_thread.is_this_thread()) }
        let mut num_parks = self.num_parks.lock().unwrap();
        *num_parks += 1;
        if *num_parks > 0 {
            self.status.write().unwrap().parked = true;
            drop(self.await_on_park.wait_timeout(num_parks, Duration::from_nanos(time_nanos)).unwrap());
        }
    }

    pub fn unpark(&self) {
        self.await_on_park.notify_one();
        let mut num_parks_guard = self.num_parks.lock().unwrap();
        *num_parks_guard -= 1;
        if *num_parks_guard <= 0 {
            self.status.write().unwrap().parked = false;
        }
        drop(num_parks_guard)
    }

    pub unsafe fn suspend_thread(&self, int_state: &mut InterpreterStateGuard) -> Result<(), SuspendError> {
        let SuspendedStatus { suspended, suspend_condvar: _ } = &self.suspended;
        let mut suspended_guard = suspended.lock().unwrap();
        let res = if *suspended_guard {
            Err(SuspendError::AlreadySuspended)
        } else {
            *suspended_guard = true;
            Ok(())
        };
        if self.underlying_thread.is_this_thread() {
            assert_eq!(self.java_tid, int_state.thread.java_tid);
            suspend_check(int_state);
        }
        if !self.is_alive() {
            Err(SuspendError::NotAlive)
        } else {
            res
        }
    }

    pub unsafe fn resume_thread(&self) -> Result<(), ResumeError> {
        let SuspendedStatus { suspended, suspend_condvar } = &self.suspended;
        let mut suspend_guard = suspended.lock().unwrap();
        if !*suspend_guard {
            Err(NotSuspended)
        } else {
            *suspend_guard = false;
            suspend_condvar.notify_one();//notify one and notify all should be the same here
            Ok(())
        }
    }

    pub fn is_this_thread(&self) -> bool {
        unsafe { self.underlying_thread.is_this_thread() }
    }
}

pub enum SuspendError {
    AlreadySuspended,
    NotAlive,
}

pub enum ResumeError {
    NotSuspended
}

pub mod monitors;