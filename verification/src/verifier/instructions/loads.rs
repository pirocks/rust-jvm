use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::instructions::{InstructionTypeSafe, exception_stack_frame, ResultFrames, nth0};
use crate::verifier::Frame;
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use crate::verifier::instructions::special::array_component_type;
use rust_jvm_common::unified_types::ArrayType;
use rust_jvm_common::unified_types::ClassWithLoader;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::unified_types::VerificationType;
use rust_jvm_common::unified_types::ParsedType;
use crate::verifier::instructions::type_transition;


pub fn instruction_is_type_safe_aaload(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, stack_frame)?;
    let component_type = array_component_type(array_type)?;
    let bl = env.vf.bootstrap_loader.clone();
    let object = ClassWithLoader { class_name: ClassName::Str("java/lang/Object".to_string()), loader: bl };
    let object_array = VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::from(ParsedType::Class(object)) });
    let next_frame = valid_type_transition(env, vec![VerificationType::IntType, object_array], &component_type.to_verification_type(), stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &VerificationType, frame: &Frame) -> Result<Frame, TypeSafetyError> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame = valid_type_transition(env, vec![], &actual_type, frame)?;
    is_assignable(&env.vf, &actual_type, type_)?;
    Result::Ok(next_frame)
}

pub fn instruction_is_type_safe_lload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::LongType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame }))
}


pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::Reference, stack_frame)?;
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

pub fn instruction_is_type_safe_caload(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
    type_transition(env,stack_frame,vec![VerificationType::IntType,VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::CharType) })],VerificationType::IntType)
}

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

pub fn instruction_is_type_safe_fload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::FloatType, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_iaload(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>{
//    unimplemented!()
//}


pub fn instruction_is_type_safe_iload(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::IntType, stack_frame)?;
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
