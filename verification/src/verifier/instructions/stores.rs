use std::rc::Rc;

use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::ClassWithLoader;
use rust_jvm_common::vtype::VType;

use crate::verifier::{Frame, standard_exception_frame};
use crate::verifier::codecorrectness::can_pop;
use crate::verifier::codecorrectness::Environment;
use crate::verifier::codecorrectness::pop_matching_type;
use crate::verifier::codecorrectness::size_of;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use crate::verifier::TypeSafetyError;
use crate::VerifierContext;

fn store_is_type_safe(env: &Environment, index: u16, type_: &VType, frame: Frame) -> Result<Frame, TypeSafetyError> {
    let mut next_stack = frame.stack_map.clone();
    let actual_type = pop_matching_type(&env.vf, &mut next_stack, &type_)?;
    let new_locals = modify_local_variable(&env.vf, index, actual_type, &frame.locals)?;
    Result::Ok(Frame {
        locals: Rc::new(new_locals),
        stack_map: next_stack,
        flag_this_uninit: frame.flag_this_uninit,
    })
}

pub fn modify_local_variable(vf: &VerifierContext, index: u16, type_: VType, locals: &[VType]) -> Result<Vec<VType>, TypeSafetyError> {
    let index = index as usize;
    let mut locals_copy = locals.to_vec();
    if size_of(vf, &locals[index]) == 1 {
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    } else if size_of(vf, &locals[index]) == 2 {
        if &locals[index + 1] != &VType::TopType {
            return Result::Err(unknown_error_verifying!());
        }
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_aastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let object = ClassWithLoader { class_name: CClassName::object(), loader: env.vf.current_loader.clone() };
    let object_type = VType::Class(object);
    let object_array = VType::ArrayReferenceType(CPDType::object());
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![object_type, VType::IntType, object_array])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_astore(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
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
            CPDType::ByteType => Result::Ok(()),
            CPDType::BooleanType => Result::Ok(()),
            _ => Result::Err(unknown_error_verifying!())
        },
        _ => Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_castore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, VType::ArrayReferenceType(CPDType::CharType)])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_dastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::DoubleType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::DoubleType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_dstore(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::DoubleType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_fastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::FloatType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::FloatType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_fstore(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::FloatType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_iastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::IntType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_istore(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::IntType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_lastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::LongType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::LongType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}


pub fn instruction_is_type_safe_lstore(index: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = store_is_type_safe(env, index, &VType::LongType, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_sastore(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(CPDType::ShortType);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, array_type])?;
    standard_exception_frame(locals, flag, next_frame)
}
