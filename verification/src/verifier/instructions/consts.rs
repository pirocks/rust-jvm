use crate::verifier::instructions::{InstructionTypeSafe, type_transition};
use crate::verifier::codecorrectness::Environment;
use crate::verifier::TypeSafetyError;
use crate::verifier::Frame;
use rust_jvm_common::unified_types::VType;

pub fn instruction_is_type_safe_iconst_m1(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![],VType::IntType)
}


pub fn instruction_is_type_safe_lconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![],VType::LongType)
}


pub fn instruction_is_type_safe_aconst_null(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![],VType::NullType)
}


pub fn instruction_is_type_safe_dconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![],VType::DoubleType)
}

pub fn instruction_is_type_safe_fconst_0(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![],VType::FloatType)
}
