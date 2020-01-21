use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::codecorrectness::valid_type_transition;
use crate::verifier::codecorrectness::Environment;
use crate::verifier::instructions::ResultFrames;
use crate::verifier::TypeSafetyError;
use crate::verifier::Frame;
use crate::verifier::instructions::exception_stack_frame;
use rust_jvm_common::unified_types::VerificationType;

pub fn instruction_is_type_safe_iconst_m1(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = valid_type_transition(env,vec![],&VerificationType::IntType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames {exception_frame, next_frame}))
}


pub fn instruction_is_type_safe_lconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &VerificationType::LongType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}



pub fn instruction_is_type_safe_aconst_null(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    let next_frame = valid_type_transition(env,vec![], &VerificationType::NullType,stack_frame)?;
    //todo dup with above
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_dconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    let next_frame = valid_type_transition(env,vec![], &VerificationType::DoubleType,stack_frame)?;
    //todo dup with above
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_fconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    let next_frame = valid_type_transition(env,vec![],&VerificationType::FloatType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}
