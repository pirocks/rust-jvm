use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::type_transition;
use rust_jvm_common::unified_types::VerificationType;


pub fn instruction_is_type_safe_d2f(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::DoubleType],VerificationType::FloatType)
}

pub fn instruction_is_type_safe_d2i(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::DoubleType],VerificationType::IntType)
}

pub fn instruction_is_type_safe_d2l(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::DoubleType],VerificationType::LongType)
}


pub fn instruction_is_type_safe_dadd(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![VerificationType::DoubleType, VerificationType::DoubleType],VerificationType::DoubleType)
}


pub fn instruction_is_type_safe_dcmpg(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![VerificationType::DoubleType, VerificationType::DoubleType],VerificationType::IntType)
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_dneg(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}

pub fn instruction_is_type_safe_f2d(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::FloatType],VerificationType::DoubleType)
}

pub fn instruction_is_type_safe_f2i(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::FloatType],VerificationType::IntType)
}

pub fn instruction_is_type_safe_f2l(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::FloatType],VerificationType::LongType)
}

pub fn instruction_is_type_safe_fadd(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::FloatType,VerificationType::FloatType],VerificationType::FloatType)
}

pub fn instruction_is_type_safe_fcmpg(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::FloatType,VerificationType::FloatType],VerificationType::IntType)
}

pub fn instruction_is_type_safe_fneg(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::FloatType],VerificationType::FloatType)
}

