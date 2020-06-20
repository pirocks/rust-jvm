use std::sync::{RwLock, Arc};
use crate::JavaThread;
use std::collections::HashMap;
use std::thread::{ThreadId, LocalKey};
use std::cell::RefCell;
use crate::monitor::Monitor;
use jvmti_jni_bindings::jrawMonitorID;
use crate::java_values::Object;
use crate::threading::monitors::Monitor;

pub struct ThreadState {
    main_thread: RwLock<Option<Arc<JavaThread>>>,
    pub alive_threads: RwLock<HashMap<ThreadId, Arc<JavaThread>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread>>>>,
    pub system_thread_group: RwLock<Option<Arc<Object>>>,
    monitors: RwLock<Vec<Arc<Monitor>>>,
}

impl ThreadState {
    pub fn get_monitor(&self, monitor: jrawMonitorID) -> Arc<Monitor> {
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard[monitor as usize].clone();
        std::mem::drop(monitors_read_guard);
        monitor
    }
}

thread_local! {
    static JVMTI_TLS: RefCell<*mut c_void> = RefCell::new(std::ptr::null_mut());
}

thread_local! {
    static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread>>> = RefCell::new(None);
}



type ThreadId = i64;

#[derive(Debug)]
pub struct JavaThread {
    pub java_tid: ThreadId,
    pub call_stack: RefCell<Vec<Rc<StackEntry>>>,
    pub thread_object: RefCell<Option<JThread>>,
    //for the main thread the object may not exist for a bit,b/c the code to create that object needs to run on a thread
    //todo maybe this shouldn't be private?
    pub interpreter_state: InterpreterState,
    pub unix_tid: Pid,
}

//todo is this correct?
unsafe impl Send for JavaThread {}

unsafe impl Sync for JavaThread {}

impl JavaThread {
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