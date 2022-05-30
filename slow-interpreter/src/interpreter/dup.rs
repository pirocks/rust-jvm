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
//
// pub fn dup_x2(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: StackEntryMut<'gc, 'l>) {
//     let current_pc = current_frame.to_ref().pc(jvm);
//     let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
//     let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[todo!()];
//     /*let value2_vtype = data[1].clone();*/
//     let value1 = current_frame.pop(None); //in principle type doesn't matter
//     let value2 = current_frame.pop(None);
//     match value1.to_type() {
//         RuntimeType::LongType | RuntimeType::DoubleType => {
//             current_frame.push(value1.clone());
//             current_frame.push(value2);
//             current_frame.push(value1);
//         }
//         _ => {
//             let value3 = current_frame.pop(None); //in principle type doesn't matter todo pass it anyway
//             current_frame.push(value1.clone());
//             current_frame.push(value3);
//             current_frame.push(value2);
//             current_frame.push(value1);
//         }
//     }
// }

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
//
pub fn dup2_x1<'gc, 'l, 'k, 'j>(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, current_pc: ByteCodeOffset)-> PostInstructionAction<'gc> {
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
