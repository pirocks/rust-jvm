use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::runtime_type::RuntimeType;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::InterpreterFrame;
use crate::JVMState;

pub fn pop2<'gc, 'l, 'k, 'j, 'h>(jvm: &'gc JVMState<'gc>, method_id: MethodId, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j, 'h>, current_pc: ByteCodeOffset) -> PostInstructionAction<'gc> {
    let stack_frames = &jvm.function_frame_type_data.read().unwrap().no_tops[&method_id];
    let category2 = &stack_frames[&current_pc].is_category_2_no_tops();
    let value1_vtype = if category2[0] {
        RuntimeType::LongType
    } else {
        RuntimeType::IntType
    };
    current_frame.pop(RuntimeType::LongType); //in principle type doesn't matter todo pass it anyway
    match value1_vtype {
        RuntimeType::LongType | RuntimeType::DoubleType => {

        }
        _ => {
            current_frame.pop(RuntimeType::LongType);
        }
    };
    PostInstructionAction::Next {}
}
