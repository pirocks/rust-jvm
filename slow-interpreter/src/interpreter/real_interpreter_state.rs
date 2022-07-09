use std::ffi::c_void;
use std::ptr::NonNull;
use rust_jvm_common::runtime_type::RuntimeType;
use crate::{InterpreterStateGuard, JVMState, NewJavaValueHandle};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InterpreterJavaValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(Option<NonNull<c_void>>),
}

impl InterpreterJavaValue {
    pub fn null() -> Self{
        Self::Object(None)
    }

    pub fn from_raw(raw: u64, rtype: RuntimeType) -> Self {
        match rtype {
            RuntimeType::IntType => {
                Self::Int(raw as i32)
            }
            RuntimeType::FloatType => {
                Self::Float(f32::from_bits(raw as u32))
            }
            RuntimeType::DoubleType => {
                Self::Double(f64::from_bits(raw))
            }
            RuntimeType::LongType => {
                Self::Long(raw as i64)
            }
            RuntimeType::Ref(_) => {
                Self::Object(NonNull::new(raw as *mut c_void))
            }
            RuntimeType::TopType => {
                todo!()
            }
        }
    }

    pub fn to_new_java_handle<'gc>(self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        match self {
            InterpreterJavaValue::Int(int) => {
                NewJavaValueHandle::Int(int)
            }
            InterpreterJavaValue::Long(long) => {
                NewJavaValueHandle::Long(long)
            }
            InterpreterJavaValue::Float(float) => {
                NewJavaValueHandle::Float(float)
            }
            InterpreterJavaValue::Double(double) => {
                NewJavaValueHandle::Double(double)
            }
            InterpreterJavaValue::Object(None) => {
                NewJavaValueHandle::from_optional_object(None)
            }
            InterpreterJavaValue::Object(Some(nonnull)) => {
                NewJavaValueHandle::from_optional_object(Some(jvm.gc.register_root_reentrant(jvm, nonnull)))
            }
        }
    }

    pub fn to_raw(self) -> u64 {
        match self {
            InterpreterJavaValue::Int(int) => int as u32 as u64,
            InterpreterJavaValue::Long(long) => long as u64,
            InterpreterJavaValue::Float(float) => float.to_bits() as u64,
            InterpreterJavaValue::Double(double) => double.to_bits(),
            InterpreterJavaValue::Object(obj) => obj.map(|nonnull| nonnull.as_ptr() as u64).unwrap_or(0)
        }
    }

    pub fn unwrap_int(&self) -> i32{
        match self {
            InterpreterJavaValue::Int(int) => {
                *int
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_object(&self) -> Option<NonNull<c_void>>{
        match self {
            InterpreterJavaValue::Object(o) => {
                *o
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_long(&self) -> i64{
        match self {
            InterpreterJavaValue::Long(long) => {
                *long
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_float(&self) -> f32{
        match self {
            InterpreterJavaValue::Float(float) => {
                *float
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_double(&self) -> f64{
        match self {
            InterpreterJavaValue::Double(double) => {
                *double
            }
            _ => {
                panic!()
            }
        }
    }
}

pub struct RealInterpreterStateGuard<'gc, 'l, 'k> {
    interpreter_state: &'k mut InterpreterStateGuard<'gc, 'l>,
    jvm: &'gc JVMState<'gc>,
    pub current_stack_depth_from_start: u16,
}

impl<'gc, 'l, 'k> RealInterpreterStateGuard<'gc, 'l, 'k> {
    pub fn new(jvm: &'gc JVMState<'gc>, interpreter_state: &'k mut InterpreterStateGuard<'gc, 'l>) -> Self {
        Self {
            interpreter_state,
            jvm,
            current_stack_depth_from_start: 0,
        }
    }

    pub fn current_frame_mut(&mut self) -> InterpreterFrame<'gc, 'l, 'k, '_> {
        InterpreterFrame {
            inner: self
        }
    }

    pub fn inner(&'_ mut self) -> &'_ mut InterpreterStateGuard<'gc, 'l> {
        self.interpreter_state
    }
}


pub struct InterpreterFrame<'gc, 'l, 'k, 'j> {
    inner: &'j mut RealInterpreterStateGuard<'gc, 'l, 'k>,
}

impl<'gc, 'l, 'k, 'j> InterpreterFrame<'gc, 'l, 'k, 'j> {
    pub fn inner(&'_ mut self) -> &mut RealInterpreterStateGuard<'gc, 'l, 'k>{
        self.inner
    }

    pub fn pop(&mut self, runtime_type: RuntimeType) -> InterpreterJavaValue {
        if self.inner.current_stack_depth_from_start < 1{
            let jvm = self.inner.jvm;
            self.inner.inner().debug_print_stack_trace(jvm);
            panic!()
        }
        self.inner.current_stack_depth_from_start -= 1;
        let current_depth = self.inner.current_stack_depth_from_start;
        let current_frame = self.inner.interpreter_state.current_frame();
        let operand_stack = current_frame.operand_stack(self.inner.jvm);
        operand_stack.interpreter_get(current_depth, runtime_type)
    }

    pub fn push(&mut self, val: InterpreterJavaValue) {
        let current_depth = self.inner.current_stack_depth_from_start;
        self.inner.interpreter_state.current_frame_mut().operand_stack_mut().interpreter_set(current_depth, val);
        self.inner.current_stack_depth_from_start += 1;
    }

    pub fn local_get(&self, i: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        let current_frame = self.inner.interpreter_state.current_frame();
        let local_vars = current_frame.local_vars(self.inner.jvm);
        local_vars.interpreter_get(i, rtype)
    }

    pub fn operand_stack_get(&self, i: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        let current_frame = self.inner.interpreter_state.current_frame();
        let operand_stack = current_frame.operand_stack(self.inner.jvm);
        operand_stack.interpreter_get(i, rtype)
    }

    pub fn local_set(&mut self, i: u16, local: InterpreterJavaValue)  {
        let mut current_frame = self.inner.interpreter_state.current_frame_mut();
        let mut local_vars = current_frame.local_vars_mut(self.inner.jvm);
        local_vars.interpreter_set(i, local)
    }

    pub fn operand_stack_depth(&self) -> u16{
        self.inner.current_stack_depth_from_start
    }
}