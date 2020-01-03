use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::TypeSafetyError;
use crate::verifier::codecorrectness::valid_type_transition;
use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::instructions::exception_stack_frame;
use crate::verifier::instructions::ResultFrames;

//#[allow(unused)]
//pub fn instruction_is_type_safe_d2f(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
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
//#[allow(unused)]
//pub fn instruction_is_type_safe_dadd(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//

pub fn instruction_is_type_safe_dcmpg(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
    let next_frame = valid_type_transition(env, vec![UnifiedType::DoubleType, UnifiedType::DoubleType], &UnifiedType::IntType, stack_frame)?;
    //todo dup
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_dneg(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_f2d(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_f2i(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_f2l(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_fadd(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_fcmpg(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_fneg(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError>{
//    unimplemented!()
//}
//
