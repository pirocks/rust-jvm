use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::vtype::VType;

use crate::verifier::{Frame, standard_exception_frame};
use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::instructions::{InstructionTypeSafe, nth0};
use crate::verifier::instructions::special::array_component_type;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use crate::verifier::instructions::stores::is_small_array;
use crate::verifier::instructions::type_transition;
use crate::verifier::TypeSafetyError;

pub fn instruction_is_type_safe_aaload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, &stack_frame)?;
    let component_type = array_component_type(array_type)?;
    let object_array = VType::ArrayReferenceType(CPDType::object());
    let locals = stack_frame.locals.clone();
    let flags = stack_frame.flag_this_uninit;
    let next_frame = valid_type_transition(env, vec![VType::IntType, object_array], component_type, stack_frame)?;
    standard_exception_frame(locals, flags, next_frame)
}

fn load_is_type_safe(env: &Environment, index: u16, type_: &VType, frame: Frame) -> Result<Frame, TypeSafetyError> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals)?;
    let next_frame = valid_type_transition(env, vec![], actual_type.clone(), frame)?;
    is_assignable(&env.vf, &actual_type, type_, true)?;
    Result::Ok(next_frame)
}

pub fn instruction_is_type_safe_lload(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = load_is_type_safe(env, index, &VType::LongType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_aload(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = load_is_type_safe(env, index, &VType::Reference, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_baload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, &stack_frame)?;
    is_small_array(array_type)?;
    type_transition(env, stack_frame, vec![VType::IntType, VType::TopType], VType::IntType)
}

pub fn instruction_is_type_safe_caload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::ArrayReferenceType(CPDType::CharType)], VType::IntType)
}

pub fn instruction_is_type_safe_daload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::DoubleType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::DoubleType)
}

pub fn instruction_is_type_safe_dload(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = load_is_type_safe(env, index, &VType::DoubleType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_faload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::FloatType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::FloatType)
}

pub fn instruction_is_type_safe_fload(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = load_is_type_safe(env, index, &VType::FloatType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_iaload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::ArrayReferenceType(CPDType::IntType)], VType::IntType)
}

pub fn instruction_is_type_safe_iload(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = load_is_type_safe(env, index, &VType::IntType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_laload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::LongType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::LongType)
}

pub fn instruction_is_type_safe_saload(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::ShortType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::IntType)
}