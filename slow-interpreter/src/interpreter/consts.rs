use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};

pub fn fconst_0<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Float(0.0));
    PostInstructionAction::Next {}
}

pub fn fconst_1<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Float(1.0));
    PostInstructionAction::Next {}
}

pub fn fconst_2<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Float(2.0));
    PostInstructionAction::Next {}
}

pub fn bipush<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, b: i8) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(b as i32));
    PostInstructionAction::Next {}
}

pub fn sipush<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, val: i16) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(val as i32));
    PostInstructionAction::Next {}
}

//
pub fn aconst_null<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Object(None));
    PostInstructionAction::Next {}
}

pub fn iconst_5<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(5));
    PostInstructionAction::Next {}
}

pub fn iconst_4<'gc, 'l, 'k, 'j>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(4));
    PostInstructionAction::Next {}
}

pub fn iconst_3<'gc, 'l, 'k, 'j>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(3));
    PostInstructionAction::Next {}
}

pub fn iconst_2<'gc, 'l, 'k, 'j>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(2));
    PostInstructionAction::Next {}
}

pub fn iconst_1<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(1));
    PostInstructionAction::Next {}
}

pub fn iconst_0<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(0));
    PostInstructionAction::Next {}
}

pub fn iconst_m1<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Int(-1));
    PostInstructionAction::Next {}
}

pub fn dconst_1<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Double(1.0));
    PostInstructionAction::Next {}
}

pub fn dconst_0<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Double(0.0));
    PostInstructionAction::Next {}
}

pub fn lconst<'gc, 'j, 'k, 'l>(mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, i: i64) -> PostInstructionAction<'gc> {
    current_frame.push(InterpreterJavaValue::Long(i));
    PostInstructionAction::Next {}
}

