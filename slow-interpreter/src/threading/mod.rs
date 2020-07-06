use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, RwLock,  RwLockReadGuard};
use std::thread::LocalKey;

use jvmti_jni_bindings::jrawMonitorID;
use userspace_threads::{Thread, Threads};

use crate::{InterpreterState, JVMState, InterpreterStateGuard, SuspendedStatus, MainThreadInitializeInfo, set_properties, run_main};
use crate::java::lang::thread::JThread;
use crate::stack_entry::StackEntry;
use crate::threading::monitors::Monitor;
use std::sync::mpsc::{channel, Sender, Receiver};
use rust_jvm_common::classnames::ClassName;
use crate::interpreter_util::{check_inited_class, push_new_object};
use std::any::Any;
use crate::interpreter::run_function;
use crate::java::lang::thread_group::JThreadGroup;

pub struct ThreadState {
    pub(crate) threads: Threads,
    main_thread: RwLock<Option<Arc<JavaThread>>>,
    all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread>>>,
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

    pub fn setup_main_thread(&'static self, jvm: &'static JVMState) -> (Arc<JavaThread>, Sender<MainThreadInitializeInfo>, Sender<MainThreadStartInfo>) {
        let main_thread = ThreadState::bootstrap_main_thread(jvm, &jvm.thread_state.threads);
        *self.main_thread.write().unwrap() = main_thread.clone().into();
        self.all_java_threads.write().unwrap().insert(main_thread.java_tid, main_thread.clone());
        let (init_send, init_recv) = channel();
        let (main_send, main_recv) = channel();
        let main_thread_clone = main_thread.clone();
        main_thread.clone().underlying_thread.start_thread(box move |_| {
            jvm.thread_state.set_current_thread(main_thread.clone());
            ThreadState::jvm_init_from_main_thread(jvm, init_recv);
            let mut int_state = InterpreterStateGuard { int_state: main_thread.interpreter_state.write().unwrap().into(), thread: &main_thread.clone() };
            let MainThreadStartInfo { args } = main_recv.recv().unwrap();
            run_main(args, jvm, &mut int_state).unwrap();
        }, box ());
        (main_thread_clone, init_send, main_send)
    }

    fn jvm_init_from_main_thread(jvm: &'static JVMState, init_recv: Receiver<MainThreadInitializeInfo>) {
        let _init_info = init_recv.recv().unwrap();
        let main_thread = &jvm.thread_state.get_main_thread();
        assert!(Arc::ptr_eq(main_thread ,&jvm.thread_state.get_current_thread()));
        let mut int_state = InterpreterStateGuard { int_state: main_thread.interpreter_state.write().unwrap().into(), thread: main_thread };
        run_function(&jvm, &mut int_state);
        let function_return = int_state.function_return_mut();
        if *function_return {
            *function_return = false;
        }
        if int_state.throw_mut().is_some() || *int_state.terminate_mut() {
            unimplemented!()
        }
        *jvm.live.write().unwrap() = true;
        set_properties(jvm, &mut int_state);
        jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.vm_inited(jvm, jvm.thread_state.get_main_thread()));
        // drop(int_state);
    }

    pub fn get_main_thread(&self) -> Arc<JavaThread> {
        self.main_thread.read().unwrap().as_ref().unwrap().clone()
    }

    fn set_current_thread(&'static self, thread: Arc<JavaThread>){
        self.current_java_thread.with(|refcell|{
            assert!(refcell.borrow().is_none());
            *refcell.borrow_mut() = thread.into();
        })
    }

    fn bootstrap_main_thread<'l>(jvm: &'static JVMState, threads: &Threads) -> Arc<JavaThread> {
        let (main_thread_obj_send, main_thread_obj_recv) = channel();
        let bootstrap_underlying_thread = threads.create_thread();
        let bootstrap_thread = Arc::new(JavaThread {
            java_tid: 0,
            underlying_thread: bootstrap_underlying_thread,
            thread_object: RwLock::new(None),
            interpreter_state: RwLock::new(InterpreterState::default()),
            suspended: RwLock::new(SuspendedStatus::default()),
            invisible_to_java: true
        });
        let underlying = &bootstrap_thread.clone().underlying_thread;
        jvm.thread_state.set_current_thread(bootstrap_thread.clone());
        let target_classfile = check_inited_class(jvm,
            &mut InterpreterStateGuard { int_state: bootstrap_thread.interpreter_state.write().unwrap().into(), thread: &bootstrap_thread },
            &ClassName::thread().into(),
            jvm.bootstrap_loader.clone(),
        );


        underlying.start_thread(box move |_data: Box<dyn Any>| {
            let frame = StackEntry {
                class_pointer: target_classfile.clone(),
                method_i: std::u16::MAX,
                local_vars: vec![],
                operand_stack: vec![],
                pc: std::usize::MAX,
                pc_offset: -1,
            };
            let mut new_int_state = InterpreterStateGuard { int_state: bootstrap_thread.interpreter_state.write().unwrap().into(), thread: &bootstrap_thread };
            new_int_state.push_frame(frame);
            push_new_object(jvm, &mut new_int_state, &target_classfile, None);
            let thread_object = new_int_state.pop_current_operand_stack().cast_thread();
            thread_object.set_priority(5);
            *bootstrap_thread.thread_object.write().unwrap() = thread_object.into();
            jvm.thread_state.set_current_thread(bootstrap_thread.clone());
            // push_new_object(jvm, &mut new_int_state,  &target_classfile, None);
            // let jthread = new_int_state.pop_current_operand_stack().cast_thread();
            let system_thread_group = JThreadGroup::init(jvm, &mut new_int_state);
            *jvm.thread_state.system_thread_group.write().unwrap() = system_thread_group.clone().into();
            let jthread = JThread::new(jvm, &mut new_int_state,system_thread_group,"Main".to_string());
            main_thread_obj_send.send(jthread).unwrap();
        }, box ());
        let thread_obj: JThread = main_thread_obj_recv.recv().unwrap();
        JavaThread::new(jvm, thread_obj, threads.create_thread(), false)
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
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard[monitor as usize].clone();
        std::mem::drop(monitors_read_guard);
        monitor
    }

    pub fn get_thread_by_tid(&self, tid: JavaThreadId) -> Arc<JavaThread> {
        self.try_get_thread_by_tid(tid).unwrap()
    }

    pub fn try_get_thread_by_tid(&self, tid: JavaThreadId) -> Option<Arc<JavaThread>> {
        self.all_java_threads.read().unwrap().get(&tid).cloned()
    }

    pub fn start_thread_from_obj<'l>(&self, jvm: &'static JVMState, obj: JThread, int_state: & mut InterpreterStateGuard, invisible_to_java: bool) -> Arc<JavaThread> {
        let underlying = self.threads.create_thread();

        let (send, recv) = channel();
        let thread_class = check_inited_class(jvm, int_state, &ClassName::thread().into(), int_state.current_loader(jvm).clone());
        let java_thread = JavaThread::new(jvm,obj, underlying, invisible_to_java);
        java_thread.clone().underlying_thread.start_thread(box move |_data| {
            send.send(java_thread.clone()).unwrap();
            let new_thread_frame = StackEntry {
                class_pointer: thread_class.clone(),
                method_i: std::u16::MAX,
                local_vars: vec![],
                operand_stack: vec![],
                pc: std::usize::MAX,
                pc_offset: -1,
            };
            let mut interpreter_state_guard = InterpreterStateGuard { int_state: java_thread.interpreter_state.write().unwrap().into(), thread: &java_thread };
            interpreter_state_guard.push_frame(new_thread_frame);
            java_thread.thread_object.read().unwrap().as_ref().unwrap().run(jvm, &mut interpreter_state_guard);
            interpreter_state_guard.pop_frame();
        }, box ());//todo is this Data really needed since we have a closure
        recv.recv().unwrap()
    }


    pub fn get_all_threads(&self) -> RwLockReadGuard<HashMap<JavaThreadId, Arc<JavaThread>>> {
        self.all_java_threads.read().unwrap()
    }
}

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread>>> = RefCell::new(None);
}



pub type JavaThreadId = i64;

#[derive(Debug)]
pub struct JavaThread {
    pub java_tid: JavaThreadId,
    underlying_thread: Thread,
    thread_object: RwLock<Option<JThread>>,
    pub interpreter_state: RwLock<InterpreterState>,
    pub suspended: RwLock<SuspendedStatus>,
    pub invisible_to_java: bool
}

impl JavaThread {
    pub fn new(jvm: &'static JVMState, thread_obj: JThread, underlying: Thread,invisible_to_java : bool) -> Arc<JavaThread> {
        let res = Arc::new(JavaThread {
            java_tid: thread_obj.tid(),
            underlying_thread: underlying,
            thread_object: RwLock::new(thread_obj.into()),
            interpreter_state: RwLock::new(InterpreterState::default()),
            suspended: RwLock::new(SuspendedStatus::default()),
            invisible_to_java
        });
        jvm.thread_state.all_java_threads.write().unwrap().insert(res.java_tid,res.clone());
        res
    }

    /*   fn set_current_thread(&self, java_thread: Arc<JavaThread>) {
           self.current_java_thread.with(|x| x.replace(java_thread.into()));
       }*/

    /*pub fn get_current_frame(&self) -> &StackEntry {
        self.call_stack.read().unwrap().last().unwrap()
    }*/

    /*pub fn get_current_frame_mut(&self) -> &mut StackEntry {
        self.call_stack.write().unwrap().last_mut().unwrap()
    }*/

    /*pub fn get_previous_frame(&self) -> &StackEntry {
        let guard = self.call_stack.read().unwrap();
        &guard[guard.len()-2]
    }*/

    /*pub fn get_frames(&self) -> RwLockReadGuard<Vec<StackEntry>>{
        self.call_stack.read().unwrap()
    }

    pub fn get_frames_mut(&self) -> RwLockWriteGuard<Vec<StackEntry>>{
        self.call_stack.write().unwrap()
    }*/

    /*pub fn get_previous_frame_mut(&self) -> &mut StackEntry {
        let mut guard = self.call_stack.write().unwrap();
        let len = guard.len();
        &mut guard[len -2]
    }*/


    pub fn get_underlying(&self) -> &Thread {
        &self.underlying_thread
    }

    pub fn thread_object(&self) -> JThread {
        self.thread_object.read().unwrap().as_ref().unwrap().clone()
    }
}


pub mod monitors;