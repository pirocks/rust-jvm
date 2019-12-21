use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::instructions::{InstructionIsTypeSafeResult, AfterGotoFrames, exception_stack_frame, target_is_type_safe, ResultFrames};
use crate::verifier::codecorrectness::{Environment, can_pop};
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;

pub fn instruction_is_type_safe_return(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    match env.return_type {
        UnifiedType::VoidType => {}
        _ => { return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string())); }
    };
    if stack_frame.flag_this_uninit {
        return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string()));
    }
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::AfterGoto(AfterGotoFrames {
        exception_frame
    }))
}


pub fn instruction_is_type_safe_if_acmpeq(target: isize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let next_frame = can_pop(stack_frame, vec![UnifiedType::Reference, UnifiedType::Reference])?;
    assert!(target >= 0);//todo shouldn't be an assert
    target_is_type_safe(env, &next_frame, target as usize)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_goto(target: isize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    assert!(target  >= 0);//todo shouldn't be an assert
    target_is_type_safe(env, stack_frame, target as usize)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::AfterGoto(AfterGotoFrames { exception_frame }))

}
