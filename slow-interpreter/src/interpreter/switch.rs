use rust_jvm_common::classfile::{LookupSwitch, TableSwitch};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::InterpreterFrame;
use crate::jvm_state::JVMState;

pub fn invoke_lookupswitch<'gc, 'j, 'k, 'l>(ls: &LookupSwitch, jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let key = current_frame.pop(RuntimeType::IntType).unwrap_int();
    for (candidate_key, o) in &ls.pairs {
        if *candidate_key == key {
            return PostInstructionAction::NextOffset { offset_change: *o as i32 };
        }
    }
    PostInstructionAction::NextOffset { offset_change: ls.default as i32 }
}

pub fn tableswitch<'gc, 'j, 'k, 'l>(ls: &TableSwitch, jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let index = current_frame.pop(RuntimeType::IntType).unwrap_int();
    if index < ls.low || index > ls.high {
        PostInstructionAction::NextOffset { offset_change: ls.default as i32 }
    } else {
        PostInstructionAction::NextOffset { offset_change: ls.offsets[(index - ls.low) as usize] as i32 }
    }
}
