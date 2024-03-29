use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::runtime_type::RuntimeType;

use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::InterpreterFrame;
use crate::jvm_state::JVMState;

pub fn dup<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let val = current_frame.pop(RuntimeType::LongType); //type doesn't currently matter so do whatever(well it has to be 64 bit).//todo fix for when type does matter
    current_frame.push(val.clone());
    current_frame.push(val);
    PostInstructionAction::Next {}
}

pub fn dup_x1<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    let value1 = current_frame.pop(RuntimeType::LongType); //type doesn't matter
    let value2 = current_frame.pop(RuntimeType::LongType); //type doesn't matter
    current_frame.push(value1.clone());
    current_frame.push(value2);
    current_frame.push(value1);
    PostInstructionAction::Next {}
}

pub fn dup_x2<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, current_pc: ByteCodeOffset) -> PostInstructionAction<'gc> {
    let stack_frames = &jvm.function_frame_type_data.read().unwrap().no_tops[&method_id];
    let category2 = &stack_frames[&current_pc].is_category_2_no_tops();
    let value2_vtype = if category2[1] {
        RuntimeType::LongType
    } else {
        RuntimeType::IntType
    };
    let value1 = current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter
    let value2 = current_frame.pop(RuntimeType::LongType);
    match value2_vtype {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
        _ => {
            let value3 = current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter todo pass it anyway
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    };
    PostInstructionAction::Next {}
}

pub fn dup2<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, current_pc: ByteCodeOffset) -> PostInstructionAction<'gc> {
    let stack_frames = &jvm.function_frame_type_data.read().unwrap().no_tops[&method_id];
    let category2 = &stack_frames[&current_pc].is_category_2_no_tops();
    let value1_vtype = if category2[0] {
        RuntimeType::LongType
    } else {
        RuntimeType::IntType
    };
    let value1 = current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter but needs to be u64
    match value1_vtype {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            current_frame.push(value1.clone());
            current_frame.push(value1);
        }
        _ => {
            let value2 = current_frame.pop(RuntimeType::LongType);
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
    };
    PostInstructionAction::Next {}
}

pub fn dup2_x1<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, current_pc: ByteCodeOffset) -> PostInstructionAction<'gc> {
    let stack_frames = &jvm.function_frame_type_data.read().unwrap().no_tops[&method_id];
    let category2 = &stack_frames[&current_pc].is_category_2_no_tops();
    let value1_vtype = if category2[0] {
        RuntimeType::LongType
    } else {
        RuntimeType::IntType
    };
    let value1 = current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter todo pass it anyway
    match value1_vtype {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            let value2 = current_frame.pop(RuntimeType::LongType);
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        }
        _ => {
            let value2 = current_frame.pop(RuntimeType::LongType);
            let value3 = current_frame.pop(RuntimeType::LongType);
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    };
    PostInstructionAction::Next {}
}

pub fn dup2_x2<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, current_pc: ByteCodeOffset) -> PostInstructionAction<'gc> {
    let stack_frames = &jvm.function_frame_type_data.read().unwrap().no_tops[&method_id];
    let category2 = &stack_frames[&current_pc].is_category_2_no_tops();
    let value1_vtype = if category2[0] {
        RuntimeType::LongType
    } else {
        RuntimeType::IntType
    };
    let value2_vtype = if category2[1] {
        RuntimeType::LongType
    } else {
        RuntimeType::IntType
    };
    match value1_vtype {
        RuntimeType::LongType | RuntimeType::DoubleType => {
            match value2_vtype {
                RuntimeType::LongType | RuntimeType::DoubleType => {
                    //form 4:
                    let value1 = current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter todo pass it anyway
                    let value2 = current_frame.pop(RuntimeType::LongType);
                    current_frame.push(value1.clone());
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
                _ => {
                    //form 2:
                    let value1 = current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter todo pass it anyway
                    let value2 = current_frame.pop(RuntimeType::LongType);
                    let value3 = current_frame.pop(RuntimeType::LongType);
                    assert!(!category2[2]);
                    current_frame.push(value1.clone());
                    current_frame.push(value3);
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
            }
        }
        _ => {
            let value3_vtype = if category2[2] {
                RuntimeType::LongType
            } else {
                RuntimeType::IntType
            };
            match value3_vtype {
                RuntimeType::LongType | RuntimeType::DoubleType => {
                    //form 3:
                    let value1 = current_frame.pop(RuntimeType::LongType);
                    let value2 = current_frame.pop(RuntimeType::LongType);
                    let value3 = current_frame.pop(RuntimeType::LongType);
                    current_frame.push(value2.clone());
                    current_frame.push(value1.clone());
                    current_frame.push(value3);
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
                _ => {
                    //form 1
                    let value1 = current_frame.pop(RuntimeType::LongType);
                    let value2 = current_frame.pop(RuntimeType::LongType);
                    let value3 = current_frame.pop(RuntimeType::LongType);
                    let value4 = current_frame.pop(RuntimeType::LongType);
                    current_frame.push(value2.clone());
                    current_frame.push(value1.clone());
                    current_frame.push(value4);
                    current_frame.push(value3);
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
            }
        }
    };
    PostInstructionAction::Next {}
}


pub fn swap<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>) -> PostInstructionAction<'gc> {
    // let stack_frames = &jvm.function_frame_type_data.read().unwrap().no_tops[&method_id];
    // let category2 = &stack_frames[&current_pc].is_category_2_no_tops();
    // assert!(!category2[0]);
    // assert!(!category2[1]);
    let value1 = current_frame.pop(RuntimeType::LongType);
    let value2 = current_frame.pop(RuntimeType::LongType);
    current_frame.push(value1);
    current_frame.push(value2);
    PostInstructionAction::Next {}
}