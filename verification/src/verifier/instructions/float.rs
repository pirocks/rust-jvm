use classfile_view::vtype::VType;

use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::instructions::type_transition;
use crate::verifier::TypeSafetyError;

pub fn instruction_is_type_safe_d2f(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::DoubleType], VType::FloatType)
}

pub fn instruction_is_type_safe_d2i(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::DoubleType], VType::IntType)
}

pub fn instruction_is_type_safe_d2l(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::DoubleType], VType::LongType)
}

pub fn instruction_is_type_safe_dadd(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::DoubleType, VType::DoubleType], VType::DoubleType)
}

pub fn instruction_is_type_safe_dcmpg(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::DoubleType, VType::DoubleType], VType::IntType)
}

pub fn instruction_is_type_safe_dneg(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::DoubleType], VType::DoubleType)
}

pub fn instruction_is_type_safe_f2d(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::FloatType], VType::DoubleType)
}

pub fn instruction_is_type_safe_f2i(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::FloatType], VType::IntType)
}

pub fn instruction_is_type_safe_f2l(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::FloatType], VType::LongType)
}

pub fn instruction_is_type_safe_fadd(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::FloatType, VType::FloatType], VType::FloatType)
}

pub fn instruction_is_type_safe_fcmpg(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::FloatType, VType::FloatType], VType::IntType)
}

pub fn instruction_is_type_safe_fneg(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::FloatType], VType::FloatType)
}

