use std::ffi::c_void;
use std::ptr::NonNull;

use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValueCommon, JVMState, NewJavaValueHandle};
use crate::better_java_stack::frames::HasFrame;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum InterpreterJavaValue {
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Object(Option<NonNull<c_void>>),
}

impl InterpreterJavaValue {
    pub fn null() -> Self {
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

    pub fn unwrap_int(&self) -> i32 {
        match self {
            InterpreterJavaValue::Int(int) => {
                *int
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_object(&self) -> Option<NonNull<c_void>> {
        match self {
            InterpreterJavaValue::Object(o) => {
                *o
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_long(&self) -> i64 {
        match self {
            InterpreterJavaValue::Long(long) => {
                *long
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_float(&self) -> f32 {
        match self {
            InterpreterJavaValue::Float(float) => {
                *float
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_double(&self) -> f64 {
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
    interpreter_state: &'k mut JavaInterpreterFrame<'gc, 'l>,
    jvm: &'gc JVMState<'gc>,
    pub(crate) current_stack_depth_from_start: u16,
}

impl<'gc, 'l, 'k> RealInterpreterStateGuard<'gc, 'l, 'k> {
    pub fn new(jvm: &'gc JVMState<'gc>, interpreter_state: &'k mut JavaInterpreterFrame<'gc, 'l>) -> Self {
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

    pub fn inner(&'_ mut self) -> &'_ mut JavaInterpreterFrame<'gc, 'l> {
        self.interpreter_state
    }
}


pub struct InterpreterFrame<'gc, 'l, 'k, 'j> {
    inner: &'j mut RealInterpreterStateGuard<'gc, 'l, 'k>,
}

impl<'gc, 'l, 'k, 'j> InterpreterFrame<'gc, 'l, 'k, 'j> {
    pub fn inner(&'_ mut self) -> &mut RealInterpreterStateGuard<'gc, 'l, 'k> {
        self.inner
    }

    pub fn pop_all(&mut self) {
        self.inner.current_stack_depth_from_start = 0;
        self.inner.interpreter_state.pop_all();
    }

    pub fn pop(&mut self, runtime_type: RuntimeType) -> InterpreterJavaValue {
        self.inner.current_stack_depth_from_start -= 1;
        self.inner.interpreter_state.pop_os(runtime_type)
    }

    pub fn push(&mut self, val: InterpreterJavaValue) {
        self.inner.current_stack_depth_from_start += 1;
        self.inner.interpreter_state.push_os(val);
    }

    pub fn local_get(&self, i: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        self.inner.interpreter_state.local_get_handle(i, rtype).to_interpreter_jv()
    }

    pub fn operand_stack_get(&self, i: u16, rtype: RuntimeType) -> InterpreterJavaValue {
        todo!()/*let current_frame = self.inner.interpreter_state.current_frame();
        let operand_stack = current_frame.operand_stack(self.inner.jvm);
        operand_stack.interpreter_get(i, rtype)*/
    }

    pub fn local_set(&mut self, i: u16, local: InterpreterJavaValue) {
        let jvm = self.inner.jvm;
        let new_java_value_handle = local.to_new_java_handle(jvm);
        self.inner.interpreter_state.local_set_njv(i, new_java_value_handle.as_njv())
    }

    pub fn operand_stack_depth(&self) -> u16 {
        self.inner.current_stack_depth_from_start
    }
}