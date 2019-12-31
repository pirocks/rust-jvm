use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::codecorrectness::valid_type_transition;
use crate::verifier::codecorrectness::Environment;
use crate::verifier::instructions::ResultFrames;
use crate::verifier::TypeSafetyError;
use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::Frame;
use crate::verifier::instructions::exception_stack_frame;

pub fn instruction_is_type_safe_iconst_m1(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = valid_type_transition(env,vec![],&UnifiedType::IntType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames {exception_frame, next_frame}))
}


pub fn instruction_is_type_safe_lconst_0(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = valid_type_transition(env, vec![], &UnifiedType::LongType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


//
//
//#[allow(unused)]
//fn instruction_is_type_safe_aconst_null(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//

//#[allow(unused)]
//fn instruction_is_type_safe_dconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_fconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
