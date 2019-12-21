use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::instructions::{InstructionIsTypeSafeResult, exception_stack_frame, ResultFrames, nth0};
use crate::verifier::Frame;
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::TypeSafetyError;

#[allow(unused)]
fn instruction_is_type_safe_aaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame) -> Result<Frame,TypeSafetyError> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame = valid_type_transition(env, vec![], &actual_type, frame)?;
    is_assignable(&actual_type, type_)?;
    Result::Ok(next_frame)

}

pub fn instruction_is_type_safe_lload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult,TypeSafetyError>{
    let next_frame = load_is_type_safe(env,index,&UnifiedType::LongType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames {exception_frame,next_frame}))
}


pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult,TypeSafetyError>{
    let next_frame = load_is_type_safe(env, index, &UnifiedType::Reference, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames {
        exception_frame,
        next_frame,
    }))
}