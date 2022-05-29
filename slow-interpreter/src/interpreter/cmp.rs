use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};

use crate::jvm_state::JVMState;

//Floating-point comparison is performed in accordance with IEEE754
// this is the same as regular rust floats

pub fn fcmpl<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>)  -> PostInstructionAction<'gc> {
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(InterpreterJavaValue::Int(-1));
        return PostInstructionAction::Next {};
    }
    fcmp_common(jvm, current_frame, value2, value1);
    PostInstructionAction::Next {}
}

pub fn fcmpg<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>)  -> PostInstructionAction<'gc>{
    let value2 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    let value1 = current_frame.pop(RuntimeType::FloatType).unwrap_float();
    if value1.is_nan() || value2.is_nan() {
        current_frame.push(InterpreterJavaValue::Int(1));
        return PostInstructionAction::Next {};
    }
    fcmp_common(jvm, current_frame, value2, value1);
    PostInstructionAction::Next {}
}

fn fcmp_common<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, value2: f32, value1: f32) {
    if value1.to_bits() == value2.to_bits() {
        current_frame.push(InterpreterJavaValue::Int(0))
    } else if value1 > value2 {
        current_frame.push(InterpreterJavaValue::Int(1))
    } else if value1 < value2 {
        current_frame.push(InterpreterJavaValue::Int(-1))
    } else {
        panic!()
    }
}
