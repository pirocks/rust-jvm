use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::jvm_state::JVMState;

pub fn i2l<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let int = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Long(int as i64));
    PostInstructionAction::Next {}
}

pub fn i2s<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let int = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(int as i16 as i32));
    PostInstructionAction::Next {}
}

pub fn i2f<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let int = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Float(int as f32));
    PostInstructionAction::Next {}
}

pub fn l2f<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let long = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Float(long as f32));
    PostInstructionAction::Next {}
}

pub fn l2i<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let long = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Int(long as i32));
    PostInstructionAction::Next {}
}

pub fn l2d<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::LongType).unwrap_long();
    current_frame.push(InterpreterJavaValue::Double(val as f64));
    PostInstructionAction::Next {}
}

pub fn i2d<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let int = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Double(int as f64));
    PostInstructionAction::Next {}
}

pub fn i2c<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let int = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(int as u16 as i32));
    PostInstructionAction::Next {}
}

pub fn i2b<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let int = current_frame.pop(RuntimeType::IntType).unwrap_int();
    current_frame.push(InterpreterJavaValue::Int(int as u8 as i8 as i32));
    PostInstructionAction::Next {}
}

pub fn f2i<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let f = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Int(f as i32));
    PostInstructionAction::Next {}
}

pub fn f2d<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let f = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    current_frame.push(InterpreterJavaValue::Double(f as f64));
    PostInstructionAction::Next {}
}

pub fn d2i<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let f = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Int(f as i32));
    PostInstructionAction::Next {}
}

pub fn d2l<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let f = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Long(f as i64));
    PostInstructionAction::Next {}
}

pub fn d2f<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let f = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    current_frame.push(InterpreterJavaValue::Float(f as f32));
    PostInstructionAction::Next {}
}

