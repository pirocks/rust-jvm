use std::rc::Rc;

use classfile_view::loading::ClassWithLoader;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use classfile_view::vtype::VType;
use rust_jvm_common::classnames::ClassName;

use crate::verifier::{Frame, standard_exception_frame};
use crate::verifier::codecorrectness::can_pop;
use crate::verifier::codecorrectness::Environment;
use crate::verifier::codecorrectness::pop_matching_type;
use crate::verifier::codecorrectness::size_of;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use crate::verifier::TypeSafetyError;
use crate::VerifierContext;

fn store_is_type_safe(env: &Environment, index: usize, type_: &VType, frame: Frame) -> Result<Frame, TypeSafetyError> {
    let mut next_stack = frame.stack_map.clone();
    let actual_type = pop_matching_type(&env.vf, &mut next_stack, &type_)?;
    let new_locals = modify_local_variable(&env.vf, index, actual_type, &frame.locals)?;
    Result::Ok(Frame {
        locals: Rc::new(new_locals),
        stack_map: next_stack,
        flag_this_uninit: frame.flag_this_uninit,
    })
}

pub fn modify_local_variable(vf: &VerifierContext, index: usize, type_: VType, locals: &[VType]) -> Result<Vec<VType>, TypeSafetyError> {
    let mut locals_copy = locals.to_vec();
    if size_of(vf, &locals[index]) == 1 {
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    } else if size_of(vf, &locals[index]) == 2 {
        assert_eq!(&locals[index + 1], &VType::TopType);//todo this isn't completely correct. Ideally this function should fail, instead of returning a assertion error
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_aastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let object = ClassWithLoader { class_name: ClassName::object(), loader: env.vf.bootstrap_loader.clone() };
    let object_type = VType::Class(object);
    let object_array = VType::ArrayReferenceType(PTypeView::Ref(ReferenceTypeView::Class(ClassName::object())));
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![object_type, VType::IntType, object_array])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_astore(index: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::Reference, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_bastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(3, &stack_frame)?;
    is_small_array(array_type)?;
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, VType::TopType])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn is_small_array(array_type: VType) -> Result<(), TypeSafetyError> {
    match array_type {
        VType::NullType => Result::Ok(()),
        VType::ArrayReferenceType(a) => match a {
            PTypeView::ByteType => Result::Ok(()),
            PTypeView::BooleanType => Result::Ok(()),
            _ => Result::Err(unknown_error_verifying!())
        },
        _ => Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_castore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, VType::ArrayReferenceType(PTypeView::CharType)])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_dastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::DoubleType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::DoubleType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_dstore(index: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::DoubleType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_fastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::FloatType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::FloatType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_fstore(index: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::FloatType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_iastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::IntType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_istore(index: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::IntType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_lastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::LongType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::LongType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}


pub fn instruction_is_type_safe_lstore(index: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::LongType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_sastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(PTypeView::ShortType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}
