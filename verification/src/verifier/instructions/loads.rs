use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::instructions::{InstructionTypeSafe, nth0};
use crate::verifier::{Frame, standard_exception_frame};
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
use crate::verifier::instructions::stores::is_small_array;


pub fn instruction_is_type_safe_aaload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, stack_frame)?;
    let component_type = array_component_type(array_type)?;
    let bl = env.vf.bootstrap_loader.clone();
    let object = ClassWithLoader { class_name: ClassName::object(), loader: bl };
    let object_array = VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::from(ParsedType::Class(object)) });
    let next_frame = valid_type_transition(env, vec![VerificationType::IntType, object_array], &component_type.to_verification_type(), stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &VerificationType, frame: &Frame) -> Result<Frame, TypeSafetyError> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame = valid_type_transition(env, vec![], &actual_type, frame)?;
    is_assignable(&env.vf, &actual_type, type_)?;
    Result::Ok(next_frame)
}

pub fn instruction_is_type_safe_lload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::LongType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::Reference, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_baload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, stack_frame)?;
    is_small_array(array_type)?;
    type_transition(env, stack_frame, vec![VerificationType::IntType, VerificationType::TopType], VerificationType::IntType)
}

pub fn instruction_is_type_safe_caload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VerificationType::IntType, VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::CharType) })], VerificationType::IntType)
}

pub fn instruction_is_type_safe_daload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::DoubleType) });
    type_transition(env, stack_frame, vec![VerificationType::IntType, array_type], VerificationType::DoubleType)
}

pub fn instruction_is_type_safe_dload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::DoubleType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_faload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::FloatType) });
    type_transition(env, stack_frame, vec![VerificationType::IntType, array_type], VerificationType::FloatType)
}

pub fn instruction_is_type_safe_fload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::FloatType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_iaload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VerificationType::IntType, VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::IntType) })], VerificationType::IntType)
}


pub fn instruction_is_type_safe_iload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VerificationType::IntType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

//#[allow(unused)]
pub fn instruction_is_type_safe_laload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::LongType) });
    type_transition(env, stack_frame, vec![VerificationType::IntType, array_type], VerificationType::LongType)
}

pub fn instruction_is_type_safe_saload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VerificationType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::ShortType) });
    type_transition(env, stack_frame, vec![VerificationType::IntType, array_type], VerificationType::IntType)
}
