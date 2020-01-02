use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::instructions::{InstructionTypeSafe, exception_stack_frame, ResultFrames, nth0};
use crate::verifier::Frame;
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::TypeSafetyError;

#[allow(unused)]
fn instruction_is_type_safe_aaload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    unimplemented!()
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame) -> Result<Frame, TypeSafetyError> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame = valid_type_transition(env, vec![], &actual_type, frame)?;
    is_assignable(&env.vf,&actual_type, type_)?;
    Result::Ok(next_frame)
}

pub fn instruction_is_type_safe_lload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &UnifiedType::LongType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame }))
}


pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &UnifiedType::Reference, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames {
        exception_frame,
        next_frame,
    }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_baload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_caload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_daload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_dload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_faload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_fload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_iaload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}


pub fn instruction_is_type_safe_iload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env,index,&UnifiedType::IntType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_laload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_saload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}
