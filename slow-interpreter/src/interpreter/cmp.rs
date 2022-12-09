use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
use crate::jvm_state::JVMState;

//Floating-point comparison is performed in accordance with IEEE754
// this is the same as regular rust floats

pub fn fcmpl<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(InterpreterJavaValue::Int(-1));
        return PostInstructionAction::Next {};
    }
    fcmp_common(current_frame, value2, value1);
    PostInstructionAction::Next {}
}

pub fn fcmpg<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(InterpreterJavaValue::Int(1));
        return PostInstructionAction::Next {};
    }
    fcmp_common(current_frame, value2, value1);
    PostInstructionAction::Next {}
}

fn fcmp_common<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, value2: f32, value1: f32) {
    if value1 > value2 {
        current_frame.push(InterpreterJavaValue::Int(1))
    } else if value1 == value2 {
        current_frame.push(InterpreterJavaValue::Int(0))
    } else if value1 < value2 {
        current_frame.push(InterpreterJavaValue::Int(-1))
    } else {
        dbg!(value1);
        dbg!(value2);
        panic!()
    }
}


pub fn dcmpl<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let val2 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    let val1 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(InterpreterJavaValue::Int(-1));
        return PostInstructionAction::Next {};
    }
    dcmp_common(jvm, current_frame, val2, val1);
    PostInstructionAction::Next {}
}

pub fn dcmpg<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let val2 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    let val1 = current_frame.pop(RuntimeType::DoubleType).unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(InterpreterJavaValue::Int(1));
        return PostInstructionAction::Next {};
    }
    dcmp_common(jvm, current_frame, val2, val1);
    PostInstructionAction::Next {}
}


fn dcmp_common<'gc, 'j, 'k, 'l>(_jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, val2: f64, val1: f64) {
    assert!(!val2.is_nan());
    assert!(!val1.is_nan());
    let res = if val1 > val2 {
        1
    } else if val1 == val2 {
        0
    } else if val1 < val2 {
        -1
    } else {
        dbg!(val1);
        dbg!(val2);
        unreachable!()
    };
    current_frame.push(InterpreterJavaValue::Int(res));
}

