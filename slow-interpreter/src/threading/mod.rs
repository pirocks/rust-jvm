use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread::LocalKey;

use jvmti_jni_bindings::jrawMonitorID;
use userspace_threads::{Thread, Threads};

use crate::{InterpreterState, JVMState};
use crate::java::lang::thread::JThread;
use crate::java_values::Object;
use crate::stack_entry::StackEntry;
use crate::threading::monitors::Monitor;
use std::sync::mpsc::channel;
use rust_jvm_common::classnames::ClassName;
use crate::interpreter_util::check_inited_class;

pub struct ThreadState {
    threads: Threads,
    main_thread: RwLock<Option<Arc<JavaThread>>>,
    all_java_threads: RwLock<HashMap<JavaThreadId, Arc<JavaThread>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread>>>>,
    system_thread_group: RwLock<Option<Arc<Object>>>,
    monitors: RwLock<Vec<Arc<Monitor>>>,
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


    pub fn get_current_thread_name(&self) -> String {
        let current_thread = self.get_current_thread();
        let thread_object = current_thread.thread_object.borrow();
        thread_object.as_ref().map(|jthread| jthread.name().to_rust_string())
            .unwrap_or(std::thread::current().name().unwrap_or("unknown").to_string())
    }

    pub fn new_monitor(&self, name: String) -> Arc<Monitor> {
        let mut monitor_guard = self.thread_state.monitors.write().unwrap();
        let index = monitor_guard.len();
        let res = Arc::new(Monitor::new(name, index));
        monitor_guard.push(res.clone());
        res
    }

    pub fn get_current_thread(&self) -> Arc<JavaThread> {
        self.thread_state.current_java_thread.with(|thread_refcell| {
            thread_refcell.borrow().as_ref().unwrap().clone()
        })
    }

    pub fn get_monitor(&self, monitor: jrawMonitorID) -> Arc<Monitor> {
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard[monitor as usize].clone();
        std::mem::drop(monitors_read_guard);
        monitor
    }

    pub fn start_thread_from_obj(&self, jvm: &JVMState, obj: JThread, frame: &StackEntry) -> Arc<JavaThread> {
        let underlying = self.threads.create_thread();

        let (send, recv) = channel();
        let thread_class = check_inited_class(jvm, &ClassName::thread().into(), frame.class_pointer.loader(jvm).clone());
        underlying.start_thread(move |(data)|{
            let java_thread = Arc::new(JavaThread::new(obj, underlying));
            send.send(java_thread.clone()).unwrap();
            let new_thread_frame = Rc::new(StackEntry {
                        class_pointer: thread_class.clone(),
                        method_i: std::u16::MAX,
                        local_vars: RefCell::new(vec![]),
                        operand_stack: RefCell::new(vec![]),
                        pc: RefCell::new(std::usize::MAX),
                        pc_offset: RefCell::new(-1),
                    });
            java_thread.call_stack.write().unwrap().push(new_thread_frame.clone());
            java_thread.thread_object.read().unwrap().as_ref().unwrap().run(jvm, &new_thread_frame);
            java_thread.call_stack.write().unwrap().pop();
        }, unimplemented!());
        recv.recv().unwrap()
    }
}

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread>>> = RefCell::new(None);
}



pub type JavaThreadId = i64;

#[derive(Debug)]
pub struct JavaThread {
    java_tid: JavaThreadId,
    underlying_thread: Thread,
    call_stack: RwLock<Vec<Rc<StackEntry>>>,
    thread_object: RwLock<Option<JThread>>,
    interpreter_state: InterpreterState,
}

impl JavaThread {
    pub fn new(thread_obj: JThread, underlying: Thread) -> JavaThread {
        JavaThread {
            java_tid: thread_obj.tid(),
            underlying_thread: underlying,
            call_stack: RefCell::new(vec![]),
            thread_object: RefCell::new(thread_obj.into()),
            interpreter_state: InterpreterState::default(),
        }
    }

    fn set_current_thread(&self, java_thread: Arc<JavaThread>) {
        self.thread_state.current_java_thread.with(|x| x.replace(java_thread.into()));
    }

    pub fn get_current_frame(&self) -> Rc<StackEntry> {
        self.call_stack.borrow().last().unwrap().clone()
    }
    pub fn print_stack_trace(&self) {
        self.call_stack.borrow().iter().rev().enumerate().for_each(|(i, stack_entry)| {
            let name = stack_entry.class_pointer.view().name();
            let meth_name = stack_entry.class_pointer.view().method_view_i(stack_entry.method_i as usize).name();
            println!("{}.{} {} pc: {}", name.get_referred_name(), meth_name, i, stack_entry.pc.borrow())
        });
    }
}


pub mod monitors;