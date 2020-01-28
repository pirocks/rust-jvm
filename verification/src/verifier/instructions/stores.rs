use crate::verifier::codecorrectness::Environment;
use crate::verifier::{Frame, standard_exception_frame};
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::codecorrectness::pop_matching_type;
use crate::verifier::codecorrectness::size_of;
use crate::VerifierContext;
use crate::verifier::codecorrectness::can_pop;
use rust_jvm_common::unified_types::ClassWithLoader;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::unified_types::ArrayType;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use std::ops::Deref;
use rust_jvm_common::unified_types::VType;
use rust_jvm_common::unified_types::ParsedType;

fn store_is_type_safe(env: &Environment, index: usize, type_: &VType, frame: &Frame) -> Result<Frame, TypeSafetyError> {
    let mut next_stack = frame.stack_map.clone();
    let actual_type = pop_matching_type(&env.vf, &mut next_stack, &type_)?;
    let new_locals = modify_local_variable(&env.vf, index, actual_type, &frame.locals)?;
    Result::Ok(Frame {
        locals: new_locals,
        stack_map: next_stack,
        flag_this_uninit: frame.flag_this_uninit,
    })
}

pub fn modify_local_variable(vf: &VerifierContext, index: usize, type_: VType, locals: &Vec<VType>) -> Result<Vec<VType>, TypeSafetyError> {
    let mut locals_copy = locals.clone();
    if size_of(vf, &locals[index]) == 1 {
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    } else if size_of(vf, &locals[index]) == 2 {
        assert!(&locals[index + 1] == &VType::TopType);//todo this isn't completely correct. Ideally this function should fail, instead of returning a assertion error
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_aastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let object = ClassWithLoader { class_name: ClassName::object(), loader: env.vf.bootstrap_loader.clone() };
    let object_type = VType::Class(object.clone());
    let object_array = VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::Class(object)) });
    let next_frame = can_pop(&env.vf, stack_frame, vec![object_type, VType::IntType, object_array])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_astore(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = store_is_type_safe(env, index, &VType::Reference, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_bastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = nth1_operand_stack_is(3, stack_frame)?;
    is_small_array(array_type)?;
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, VType::TopType])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn is_small_array(array_type: VType) -> Result<(), TypeSafetyError> {
    match array_type {
        VType::NullType => Result::Ok(()),
        VType::ArrayReferenceType(a) => match &a.sub_type.deref() {
            ParsedType::ByteType => Result::Ok(()),
            ParsedType::BooleanType => Result::Ok(()),
            _ => Result::Err(unknown_error_verifying!())
        },
        _ => Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_castore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::CharType) })])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_dastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::DoubleType) });
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::DoubleType, VType::IntType, array_type])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_dstore(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = store_is_type_safe(env, index, &VType::DoubleType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_fastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::FloatType) });
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::FloatType, VType::IntType, array_type])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_fstore(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = store_is_type_safe(env, index, &VType::FloatType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_iastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::IntType) });
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, array_type])?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_istore(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = store_is_type_safe(env, index, &VType::IntType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_lastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::LongType) });
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::LongType, VType::IntType, array_type])?;
    standard_exception_frame(stack_frame, next_frame)
}


pub fn instruction_is_type_safe_lstore(index: usize, env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = store_is_type_safe(env, index, &VType::LongType, stack_frame)?;
    standard_exception_frame(stack_frame, next_frame)
}

pub fn instruction_is_type_safe_sastore(env: &Environment, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let array_type = VType::ArrayReferenceType(ArrayType { sub_type: Box::new(ParsedType::ShortType) });
    let next_frame = can_pop(&env.vf, stack_frame, vec![VType::IntType, VType::IntType, array_type])?;
    standard_exception_frame(stack_frame, next_frame)
}
