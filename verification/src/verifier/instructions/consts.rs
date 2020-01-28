use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::codecorrectness::valid_type_transition;
use crate::verifier::codecorrectness::Environment;
use crate::verifier::{TypeSafetyError, standard_exception_frame};
use crate::verifier::Frame;
use rust_jvm_common::unified_types::VType;

pub fn instruction_is_type_safe_iconst_m1(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &VType::IntType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_lconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &VType::LongType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_aconst_null(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &VType::NullType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_dconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &VType::DoubleType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_fconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &VType::FloatType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}
