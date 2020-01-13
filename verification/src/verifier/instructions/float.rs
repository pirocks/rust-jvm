use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::type_transition;


pub fn instruction_is_type_safe_d2f(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![UnifiedType::DoubleType],UnifiedType::FloatType)
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_d2i(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_d2l(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_dadd(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![UnifiedType::DoubleType, UnifiedType::DoubleType],UnifiedType::DoubleType)
}


pub fn instruction_is_type_safe_dcmpg(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env,stack_frame,vec![UnifiedType::DoubleType, UnifiedType::DoubleType],UnifiedType::IntType)
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_dneg(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}

pub fn instruction_is_type_safe_f2d(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![UnifiedType::FloatType],UnifiedType::DoubleType)
}

pub fn instruction_is_type_safe_f2i(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![UnifiedType::FloatType],UnifiedType::IntType)
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_f2l(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}

pub fn instruction_is_type_safe_fadd(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![UnifiedType::FloatType,UnifiedType::FloatType],UnifiedType::FloatType)
}

pub fn instruction_is_type_safe_fcmpg(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    type_transition(env,stack_frame,vec![UnifiedType::FloatType,UnifiedType::FloatType],UnifiedType::IntType)
}

//
//#[allow(unused)]
//pub fn instruction_is_type_safe_fneg(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
