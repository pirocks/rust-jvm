use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::mem::transmute;
use std::ops::Deref;
use std::sync::{Arc, RwLockWriteGuard};

use itertools::Itertools;

use classfile_view::loading::{ClassWithLoader, LoaderName};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::vtype::VType;
use rust_jvm_common::classfile::CPIndex;
use verification::OperandStack;
use verification::verifier::Frame;

use crate::interpreter_state::AddFrameNotifyError::{NothingAtDepth, Opaque};
use crate::java_values::{JavaValue, Object};
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntry;
use crate::threading::JavaThread;

#[derive(Debug)]
pub struct InterpreterState {
    pub throw: Option<Arc<Object>>,
    pub function_return: bool,
    pub(crate) call_stack: Vec<StackEntry>,
    pub(crate) should_frame_pop_notify: HashSet<usize>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        InterpreterState {
            throw: None,
            function_return: false,
            /*suspended: RwLock::new(SuspendedStatus {
            suspended: false,
            suspended_lock: Arc::new(Mutex::new(())),
        }),*/
            call_stack: vec![],
            should_frame_pop_notify: HashSet::new(),
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
            registered: true,
        }
    }

    pub fn current_loader(&self) -> LoaderName {
        self.current_frame().loader()
    }

    pub fn current_class_view(&self) -> Arc<dyn ClassView> {
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
            let method_id = jvm.method_table.write().unwrap().get_method_id(runtime_class.clone(), method_i);
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
            if stack_entry.try_method_i().is_some() /*&& stack_entry.method_i() > 0*/ {
                let type_ = stack_entry.class_pointer().view().type_();
                let view = stack_entry.class_pointer().view();
                let method_view = view.method_view_i(stack_entry.method_i() as usize);
                let meth_name = method_view.name();
                if method_view.is_native() {
                    println!("{:?}.{} {} {}", type_, meth_name, method_view.desc_str(), i)
                } else {
                    println!("{:?}.{} {} {} pc: {} {}", type_
                             , meth_name,
                             method_view.desc_str(), i, stack_entry
                                 .pc(), stack_entry.loader())
                }
            } else {
                println!("missing");
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
            return Err(Opaque);
        }
        int_state.should_frame_pop_notify.insert(depth);
        Ok(())
    }

    pub fn verify_frame(&mut self, jvm: &JVMState) {
        if let Some(method_id) = self.current_frame().current_method_id(jvm) {
            let guard = jvm.function_frame_type_data.read().unwrap();
            let Frame { stack_map, locals, .. } = match guard.get(&method_id) {
                Some(x) => x,
                None => {
                    // eprintln!("Warning, missing verification data for: {:?}", self.current_class_view().name());
                    return;
                }
            }.get(&self.current_pc()).unwrap();
            let local_java_vals = self.current_frame().local_vars();
            let java_val_stack = self.current_frame().operand_stack();
            let stack_map = remove_tops(stack_map);
            if stack_map.len() != java_val_stack.len() {
                dbg!(&stack_map.data.iter().rev().collect_vec());
                dbg!(&java_val_stack);
                self.debug_print_stack_trace();
                dbg!(self.current_pc());
                panic!()
            }
            for (jv, type_) in java_val_stack.iter().zip(stack_map.data.iter().rev()) {
                if !compatible_with_type(jv, type_) {
                    dbg!(jv);
                    dbg!(type_);
                    dbg!(&stack_map.data.iter().rev().collect_vec());
                    dbg!(&java_val_stack);
                    self.debug_print_stack_trace();
                    dbg!(self.current_pc());
                    panic!()
                }
            }
            assert_eq!(local_java_vals.len(), locals.deref().len());
            for (jv, type_) in local_java_vals.iter().zip(locals.iter()) {
                if !compatible_with_type(jv, type_) {
                    dbg!(jv);
                    dbg!(type_);
                    dbg!(&local_java_vals);
                    dbg!(&local_java_vals.iter().map(|jv| jv.to_type()).collect_vec());
                    dbg!(&locals);
                    self.debug_print_stack_trace();
                    dbg!(self.current_pc());
                    panic!()
                }
            }
        }
    }
}

fn compatible_with_type(jv: &JavaValue, type_: &VType) -> bool {
    match type_ {
        VType::DoubleType => {
            jv.unwrap_double();
            true
        }
        VType::FloatType => {
            jv.unwrap_float();
            true
        }
        VType::IntType => {
            jv.unwrap_int();
            true
        }
        VType::LongType => {
            jv.unwrap_long();
            true
        }
        VType::Class(ClassWithLoader { class_name, .. }) => {
            match jv.unwrap_object() {
                None => true,
                Some(obj) => {
                    true//todo need more granular
                    // obj.unwrap_normal_object().class_pointer.ptypeview().unwrap_class_type() == class_name.clone()
                }
            }
        }
        VType::ArrayReferenceType(array_ref) => {
            if jv.unwrap_object().is_none() {
                return true;
            }
            let elem_type = jv.unwrap_array().elem_type.clone();
            match &elem_type {
                PTypeView::ByteType => array_ref == &PTypeView::ByteType,
                PTypeView::CharType => array_ref == &PTypeView::CharType,
                PTypeView::DoubleType => todo!(),
                PTypeView::FloatType => todo!(),
                PTypeView::IntType => array_ref == &PTypeView::IntType,
                PTypeView::LongType => array_ref == &PTypeView::LongType,
                PTypeView::Ref(ref_) => {
                    match ref_ {
                        ReferenceTypeView::Class(class_) => {
                            true//todo need more granular.
                            // &PTypeView::Ref(ReferenceTypeView::Class(class_.clone())) == array_ref
                        }
                        ReferenceTypeView::Array(array_) => {
                            true//todo need more granular
                        }
                    }
                }
                PTypeView::ShortType => todo!(),
                PTypeView::BooleanType => array_ref == &PTypeView::BooleanType,
                PTypeView::VoidType => todo!(),
                PTypeView::TopType => todo!(),
                PTypeView::NullType => todo!(),
                PTypeView::Uninitialized(_) => todo!(),
                PTypeView::UninitializedThis => todo!(),
                PTypeView::UninitializedThisOrClass(_) => todo!()
            }
        }
        VType::VoidType => todo!(),
        VType::TopType => {
            match jv {
                JavaValue::Top => true,
                _ => true
            }
        }
        VType::NullType => {
            jv.unwrap_object();
            true
        }
        VType::Uninitialized(_) => {
            jv.unwrap_object_nonnull();
            true
        }
        VType::UninitializedThis => {
            jv.unwrap_object_nonnull();
            true
        }
        VType::UninitializedThisOrClass(_) => {
            jv.unwrap_object_nonnull();
            true
        }
        VType::TwoWord => todo!(),
        VType::OneWord => todo!(),
        VType::Reference => todo!(),
        VType::UninitializedEmpty => todo!(),
    }
}

fn remove_tops(stack_map: &OperandStack) -> OperandStack {
    //todo this is jank, should be idiomatic way to do this
    let mut expecting_top = false;

    let mut data = stack_map.data.iter().rev().flat_map(|cur| {
        if expecting_top {
            assert_eq!(cur, &VType::TopType);
            expecting_top = false;
            return None;
        }
        if &VType::LongType == cur || &VType::DoubleType == cur {
            expecting_top = true;
        }
        Some(cur.clone())
    }).collect::<VecDeque<_>>();
    data = data.into_iter().rev().collect();
    assert!(!expecting_top);
    OperandStack {
        data
    }
}


pub enum AddFrameNotifyError {
    Opaque,
    NothingAtDepth,
}


#[must_use = "Must handle frame push guard. "]
pub struct FramePushGuard {
    correctly_exited: bool,
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
