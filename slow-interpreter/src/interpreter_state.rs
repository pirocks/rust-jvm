use std::cell::RefCell;
use std::collections::HashSet;
use std::mem::transmute;
use std::sync::{Arc, RwLockWriteGuard};

use classfile_view::loading::LoaderName;
use classfile_view::view::{ClassView, HasAccessFlags};
use rust_jvm_common::classfile::CPIndex;

use crate::interpreter_state::AddFrameNotifyError::{NothingAtDepth, Opaque};
use crate::java_values::{JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntry;
use crate::threading::JavaThread;

#[derive(Debug)]
pub struct InterpreterState {
    pub terminate: bool,
    pub throw: Option<Arc<Object>>,
    pub function_return: bool,
    //todo find some way of clarifying these can only be acessed from one thread
    pub(crate) call_stack: Vec<StackEntry>,
    pub(crate) should_frame_pop_notify: HashSet<usize>
}

impl Default for InterpreterState {
    fn default() -> Self {
        InterpreterState {
            terminate: false,
            throw: None,
            function_return: false,
            /*suspended: RwLock::new(SuspendedStatus {
            suspended: false,
            suspended_lock: Arc::new(Mutex::new(())),
        }),*/
            call_stack: vec![],
            should_frame_pop_notify: HashSet::new()
        }
    }
}

pub struct InterpreterStateGuard<'l> {
    pub(crate) int_state: Option<RwLockWriteGuard<'l, InterpreterState>>,
    pub(crate) thread: &'l Arc<JavaThread>,
    pub(crate) registered: bool,
}


thread_local! {
pub static CURRENT_INT_STATE_GUARD_VALID :RefCell<bool> = RefCell::new(false);
}

thread_local! {
pub static CURRENT_INT_STATE_GUARD :RefCell<Option<*mut InterpreterStateGuard<'static>>> = RefCell::new(None);
}


impl<'l> InterpreterStateGuard<'l> {
    pub fn register_interpreter_state_guard(&mut self, jvm: &JVMState) {
        let ptr = unsafe { transmute::<_, *mut InterpreterStateGuard<'static>>(self as *mut InterpreterStateGuard<'l>) };
        jvm.thread_state.int_state_guard.with(|refcell| refcell.replace(ptr.into()));
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(true));
        self.registered = true;
        assert!(self.thread.is_alive());
    }


    pub fn new(jvm: &JVMState, thread: &'l Arc<JavaThread>) -> InterpreterStateGuard<'l> {
        jvm.thread_state.int_state_guard_valid.with(|refcell| refcell.replace(false));
        Self {
            int_state: thread.interpreter_state.write().unwrap().into(),
            thread,
            registered: true,//todo this is probably redundant due to CURRENT_INT_STATE_GUARD_VALID
        }
    }

    pub fn current_loader(&self) -> LoaderName {
        self.current_frame().loader()
    }

    pub fn current_class_view(&self) -> &Arc<ClassView> {
        self.current_frame().class_pointer().view()
    }


    pub fn current_frame(&'l self) -> &'l StackEntry {
        self.int_state.as_ref().unwrap().call_stack.last().unwrap()
    }

    pub fn current_frame_mut(&mut self) -> &mut StackEntry {
        self.int_state.as_mut().unwrap()
            .call_stack.last_mut().unwrap()
    }

    pub fn push_current_operand_stack(&mut self, jval: JavaValue) {
        self.current_frame_mut().push(jval)
    }

    pub fn pop_current_operand_stack(&mut self) -> JavaValue {
        if self.int_state.as_ref().unwrap().call_stack.last().unwrap().operand_stack().is_empty() {
            self.debug_print_stack_trace();
            panic!()
        }
        let int_state = self.int_state.as_mut().unwrap();
        let current_frame = int_state.call_stack.last_mut().unwrap();
        current_frame.operand_stack_mut().pop().unwrap()
    }

    pub fn previous_frame_mut(&mut self) -> &mut StackEntry {
        let call_stack = &mut self.int_state.as_mut().unwrap().call_stack;
        let len = call_stack.len();
        &mut call_stack[len - 2]
    }

    pub fn previous_frame(&self) -> &StackEntry {
        let call_stack = &self.int_state.as_ref().unwrap().call_stack;
        let len = call_stack.len();
        &call_stack[len - 2]
    }

    pub fn previous_previous_frame(&self) -> &StackEntry {
        let call_stack = &self.int_state.as_ref().unwrap().call_stack;
        let len = call_stack.len();
        &call_stack[len - 3]
    }

    pub fn set_throw(&mut self, val: Option<Arc<Object>>) {
        match self.int_state.as_mut() {
            None => {
                self.thread.interpreter_state.write().unwrap().throw = val
            }
            Some(val_mut) => {
                val_mut.throw = val;
            }
        }
    }


    pub fn function_return_mut(&mut self) -> &mut bool {
        &mut self.int_state.as_mut().unwrap().function_return
    }

    pub fn terminate_mut(&mut self) -> &mut bool {
        &mut self.int_state.as_mut().unwrap().terminate
    }


    pub fn throw(&self) -> Option<Arc<Object>> {
        match self.int_state.as_ref() {
            None => {
                self.thread.interpreter_state.read().unwrap().throw.clone()
            }
            Some(int_state) => int_state.throw.clone(),
        }
    }

    pub fn function_return(&self) -> &bool {
        &self.int_state.as_ref().unwrap().function_return
    }

    pub fn terminate(&self) -> &bool {
        &self.int_state.as_ref().unwrap().terminate
    }

    pub fn push_frame(&mut self, frame: StackEntry) -> FramePushGuard {
        self.int_state.as_mut().unwrap().call_stack.push(frame);
        FramePushGuard::default()
    }

    pub fn pop_frame(&mut self, jvm: &JVMState, mut frame_push_guard: FramePushGuard, was_exception: bool) {
        frame_push_guard.correctly_exited = true;
        let depth = self.int_state.as_mut().unwrap().call_stack.len();
        if self.int_state.as_mut().unwrap().should_frame_pop_notify.contains(&depth) {
            let runtime_class = self.current_frame().class_pointer();
            let method_i = self.current_method_i();
            let method_id = jvm.method_table.read().unwrap().get_method_id(runtime_class.clone(), method_i);
            jvm.jvmti_state.as_ref().unwrap().built_in_jdwp.frame_pop(jvm, method_id, u8::from(was_exception), self)
        }
        self.int_state.as_mut().unwrap().call_stack.pop();
        assert!(self.thread.is_alive());
    }

    pub fn call_stack_depth(&self) -> usize {
        self.int_state.as_ref().unwrap().call_stack.len()
    }

    pub fn current_pc_mut(&mut self) -> &mut usize {
        self.current_frame_mut().pc_mut()
    }

    pub fn current_pc(&self) -> usize {
        self.current_frame().pc()
    }

    pub fn current_pc_offset_mut(&mut self) -> &mut isize {
        self.current_frame_mut().pc_offset_mut()
    }

    pub fn current_pc_offset(&self) -> isize {
        self.current_frame().pc_offset()
    }

    pub fn current_method_i(&self) -> CPIndex {
        self.current_frame().method_i()
    }

    pub fn debug_print_stack_trace(&self) {
        for (i, stack_entry) in self.int_state.as_ref().unwrap().call_stack.iter().enumerate().rev() {
            if stack_entry.try_method_i().is_some() && stack_entry.method_i() > 0 {
                let name = stack_entry.class_pointer().view().name();
                let method_view = stack_entry.class_pointer().view().method_view_i(stack_entry.method_i() as usize);
                let meth_name = method_view.name();
                if method_view.is_native() {
                    println!("{}.{} {} {}", name.get_referred_name(), meth_name, method_view.desc_str(), i)
                } else {
                    println!("{}.{} {} {} pc: {} {}", name
                        .get_referred_name(), meth_name,
                             method_view.desc_str(), i, stack_entry
                                 .pc(), stack_entry.loader())
                }
            }
        }
    }

    pub fn cloned_stack_snapshot(&self) -> Vec<StackEntry> {
        self.int_state.as_ref().unwrap().call_stack.clone()
    }

    pub fn depth(&self) -> usize {
        self.int_state.as_ref().unwrap().call_stack.len()
    }

    pub fn add_should_frame_pop_notify(&mut self, depth: usize) -> Result<(), AddFrameNotifyError> {
        let int_state = self.int_state.as_mut().unwrap();
        if depth >= int_state.call_stack.len() {
            return Err(NothingAtDepth);
        }
        let entry = &int_state.call_stack[depth];
        if entry.is_native() || entry.try_class_pointer().is_none() {
            return Err(Opaque)
        }
        int_state.should_frame_pop_notify.insert(depth);
        Ok(())
    }
}

pub enum AddFrameNotifyError {
    Opaque,
    NothingAtDepth,
}


#[must_use = "Must handle frame push guard. "]
pub struct FramePushGuard {
    correctly_exited: bool
}

impl Default for FramePushGuard {
    fn default() -> Self {
        FramePushGuard { correctly_exited: false }
    }
}

impl Drop for FramePushGuard {
    fn drop(&mut self) {
        // assert!(self.correctly_exited)
    }
}

#[derive(Debug)]
pub struct SuspendedStatus {
    pub suspended: std::sync::Mutex<bool>,
    pub suspend_condvar: std::sync::Condvar,
}

impl Default for SuspendedStatus {
    fn default() -> Self {
        Self {
            suspended: std::sync::Mutex::new(false),
            suspend_condvar: Default::default(),
        }
    }
}
