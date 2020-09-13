use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::sync::atomic::Ordering;
use std::sync::mpsc::{channel, Sender};
use std::thread::LocalKey;

use jvmti_jni_bindings::*;
use rust_jvm_common::classnames::ClassName;
use userspace_threads::{Thread, Threads};

use crate::{InterpreterState, InterpreterStateGuard, JVMState, run_main, set_properties, SuspendedStatus};
use crate::interpreter::run_function;
use crate::interpreter_util::{check_inited_class, push_new_object};
use crate::java::lang::thread::JThread;
use crate::java::lang::thread_group::JThreadGroup;
use crate::jvmti::event_callbacks::ThreadJVMTIEnabledStatus;
use crate::jvmti::get_jvmti_interface;
use crate::stack_entry::StackEntry;
use crate::threading::monitors::Monitor;

pub struct ThreadState {
    pub(crate) threads: Threads,
    main_thread: RwLock<Option<Arc<JavaThread>>>,
    pub(crate) all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread>>>>,
    pub system_thread_group: RwLock<Option<JThreadGroup>>,
    monitors: RwLock<Vec<Arc<Monitor>>>,
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
        }
    }

    pub fn setup_main_thread(&'static self, jvm: &'static JVMState) -> (Arc<JavaThread>, Sender<MainThreadStartInfo>) {
        let main_thread = ThreadState::bootstrap_main_thread(jvm, &jvm.thread_state.threads);
        *self.main_thread.write().unwrap() = main_thread.clone().into();
        let (main_send, main_recv) = channel();
        let main_thread_clone = main_thread.clone();
        main_thread.clone().underlying_thread.start_thread(box move |_| {
            jvm.thread_state.set_current_thread(main_thread.clone());
            let mut int_state = InterpreterStateGuard { int_state: main_thread.interpreter_state.write().unwrap().into(), thread: &main_thread };
            ThreadState::jvm_init_from_main_thread(jvm, &mut int_state);
            assert!(!jvm.live.load(Ordering::SeqCst));
            jvm.live.store(true, Ordering::SeqCst);
            jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.vm_inited(jvm, &mut int_state, main_thread.clone()));
            let MainThreadStartInfo { args } = main_recv.recv().unwrap();
            //from the jvmti spec:
            //"he thread start event for the main application thread is guaranteed not to occur until after the handler for the VM initialization event returns. "
            main_thread.notify_alive();
            jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.thread_start(jvm, &mut int_state, main_thread.thread_object()));
            run_main(args, jvm, &mut int_state).unwrap();
            main_thread.notify_terminated()
        }, box ());
        (main_thread_clone, main_send)
    }

    fn jvm_init_from_main_thread(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) {
        let main_thread = &jvm.thread_state.get_main_thread();
        assert!(Arc::ptr_eq(main_thread, &jvm.thread_state.get_current_thread()));
        get_jvmti_interface(jvm, int_state);//this has the side effect off getting the right int_state for agent__load, but this is yuck todo
        jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.agent_load(jvm, int_state));// technically this is to late and should have been called earlier, but needs to be on this thread.
        run_function(&jvm, int_state);
        let function_return = int_state.function_return_mut();
        if *function_return {
            *function_return = false;
        }
        if int_state.throw().is_some() || *int_state.terminate() {
            unimplemented!()
        }
        set_properties(jvm, int_state);
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

    fn bootstrap_main_thread<'l>(jvm: &'static JVMState, threads: &Threads) -> Arc<JavaThread> {
        let (main_thread_obj_send, main_thread_obj_recv) = channel();
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
        });
        let underlying = &bootstrap_thread.clone().underlying_thread;
        jvm.thread_state.set_current_thread(bootstrap_thread.clone());
        let target_classfile = check_inited_class(jvm,
                                                  &mut InterpreterStateGuard { int_state: bootstrap_thread.interpreter_state.write().unwrap().into(), thread: &bootstrap_thread },
                                                  &ClassName::thread().into(),
                                                  jvm.bootstrap_loader.clone(),
        );

        //todo why is this a separate thread
        underlying.start_thread(box move |_data: Box<dyn Any>| {
            let frame = StackEntry::new_completely_opaque_frame();
            let mut new_int_state = InterpreterStateGuard { int_state: bootstrap_thread.interpreter_state.write().unwrap().into(), thread: &bootstrap_thread };
            let frame_for_bootstrapping = new_int_state.push_frame(frame);
            push_new_object(jvm, &mut new_int_state, &target_classfile, None);
            let thread_object = new_int_state.pop_current_operand_stack().cast_thread();
            thread_object.set_priority(5);
            *bootstrap_thread.thread_object.write().unwrap() = thread_object.into();
            jvm.thread_state.set_current_thread(bootstrap_thread.clone());
            bootstrap_thread.notify_alive();
            // push_new_object(jvm, &mut new_int_state,  &target_classfile, None);
            // let jthread = new_int_state.pop_current_operand_stack().cast_thread();
            let system_thread_group = JThreadGroup::init(jvm, &mut new_int_state);
            *jvm.thread_state.system_thread_group.write().unwrap() = system_thread_group.clone().into();
            let jthread = JThread::new(jvm, &mut new_int_state, system_thread_group, "Main".to_string());
            bootstrap_thread.notify_terminated();
            new_int_state.pop_frame(frame_for_bootstrapping);
            main_thread_obj_send.send(jthread).unwrap();
        }, box ());
        let thread_obj: JThread = main_thread_obj_recv.recv().unwrap();
        JavaThread::new(jvm, thread_obj, threads.create_thread("Main Java Thread".to_string().into()), false)
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

    pub fn start_thread_from_obj<'l>(&self, jvm: &'static JVMState, obj: JThread, invisible_to_java: bool) -> Arc<JavaThread> {
        let underlying = self.threads.create_thread(obj.name().to_rust_string().into());

        let (send, recv) = channel();
        let java_thread = JavaThread::new(jvm, obj, underlying, invisible_to_java);
        java_thread.clone().underlying_thread.start_thread(box move |_data| {
            send.send(java_thread.clone()).unwrap();
            let mut interpreter_state_guard = InterpreterStateGuard { int_state: java_thread.interpreter_state.write().unwrap().into(), thread: &java_thread };

            jvm.thread_state.set_current_thread(java_thread.clone());
            java_thread.notify_alive();
            jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.thread_start(jvm, &mut interpreter_state_guard, java_thread.clone().thread_object()));

            let frame_for_run_call = interpreter_state_guard.push_frame(StackEntry::new_completely_opaque_frame());
            java_thread.thread_object.read().unwrap().as_ref().unwrap().run(jvm, &mut interpreter_state_guard);
            interpreter_state_guard.pop_frame(frame_for_run_call);
            java_thread.notify_terminated();
        }, box ());//todo is this Data really needed since we have a closure
        recv.recv().unwrap()
    }


    pub fn get_all_threads(&self) -> RwLockReadGuard<HashMap<JavaThreadId, Arc<JavaThread>>> {
        self.all_java_threads.read().unwrap()
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
}

impl JavaThread {
    pub fn new(jvm: &'static JVMState, thread_obj: JThread, underlying: Thread, invisible_to_java: bool) -> Arc<JavaThread> {
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
        let obj = self.thread_object();
        obj.set_thread_status(status.get_thread_status_number(self))
    }

    pub fn is_alive(&self) -> bool {
        self.status.read().unwrap().alive
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

        let obj = self.thread_object();
        obj.set_thread_status(status.get_thread_status_number(self))
    }

    pub fn status_number(&self) -> jint {
        self.status.read().unwrap().get_thread_status_number(self)
    }
}


pub mod monitors;