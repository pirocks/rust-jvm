use crate::verifier::codecorrectness::{Environment, valid_type_transition};
use crate::verifier::instructions::{InstructionTypeSafe, nth0};
use crate::verifier::{Frame, standard_exception_frame};
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use crate::verifier::instructions::special::array_component_type;
use rust_jvm_common::classnames::ClassName;
use crate::verifier::instructions::type_transition;
use crate::verifier::instructions::stores::is_small_array;
use rust_jvm_common::vtype::VType;
use rust_jvm_common::view::ptype_view::{PTypeView, ReferenceTypeView};


pub fn instruction_is_type_safe_aaload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, stack_frame)?;
    let component_type = array_component_type(array_type)?;
    let object_array = VType::ArrayReferenceType(PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())));
    let next_frame = valid_type_transition(env, vec![VType::IntType, object_array], &component_type.to_verification_type(&env.class_loader), stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &VType, frame: &Frame) -> Result<Frame, TypeSafetyError> {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let next_frame = valid_type_transition(env, vec![], &actual_type, frame)?;
    is_assignable(&env.vf, &actual_type, type_)?;
    Result::Ok(next_frame)
}

pub fn instruction_is_type_safe_lload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VType::LongType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_aload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VType::Reference, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_baload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(2, stack_frame)?;
    is_small_array(array_type)?;
    type_transition(env, stack_frame, vec![VType::IntType, VType::TopType], VType::IntType)
}

pub fn instruction_is_type_safe_caload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::ArrayReferenceType(PTypeView::CharType)], VType::IntType)
}

pub fn instruction_is_type_safe_daload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::DoubleType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::DoubleType)
}

pub fn instruction_is_type_safe_dload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VType::DoubleType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_faload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::FloatType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::FloatType)
}

pub fn instruction_is_type_safe_fload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VType::FloatType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_iaload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_transition(env, stack_frame, vec![VType::IntType, VType::ArrayReferenceType(PTypeView::IntType)], VType::IntType)
}


pub fn instruction_is_type_safe_iload(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = load_is_type_safe(env, index, &VType::IntType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_laload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::LongType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::LongType)
}

pub fn instruction_is_type_safe_saload(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::ShortType);
    type_transition(env, stack_frame, vec![VType::IntType, array_type], VType::IntType)
}
