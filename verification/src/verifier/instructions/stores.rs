use crate::verifier::codecorrectness::Environment;
use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::InstructionTypeSafe;
use crate::verifier::instructions::exception_stack_frame;
use crate::verifier::instructions::ResultFrames;
use crate::verifier::codecorrectness::pop_matching_type;
use crate::verifier::codecorrectness::size_of;
use crate::VerifierContext;
use crate::verifier::codecorrectness::can_pop;
use rust_jvm_common::unified_types::ClassWithLoader;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::unified_types::ArrayType;
use crate::verifier::instructions::special::nth1_operand_stack_is;
use std::ops::Deref;

fn store_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame) -> Result<Frame,TypeSafetyError>{
    let mut next_stack = frame.stack_map.clone();
    let actual_type = pop_matching_type(&env.vf, &mut next_stack, &type_)?;
    let new_locals = modify_local_variable(&env.vf,index,actual_type,&frame.locals)?;
    Result::Ok(Frame {
        locals: new_locals,
        stack_map: next_stack,
        flag_this_uninit: frame.flag_this_uninit
    })
}

pub fn modify_local_variable(vf:&VerifierContext, index:usize, type_: UnifiedType, locals: &Vec<UnifiedType>) -> Result<Vec<UnifiedType>,TypeSafetyError>{
    let mut locals_copy = locals.clone();
    if size_of(vf,&locals[index]) == 1{
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    }else if size_of(vf,&locals[index]) ==  2{
        assert!(&locals[index + 1] == &UnifiedType::TopType);//todo this isn't completely correct. Ideally this function should fail, instead of returning a assertion error
        locals_copy[index] = type_;
        Result::Ok(locals_copy)
    }else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_aastore(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let object = ClassWithLoader { class_name: ClassName::Str("java/lang/Object".to_string()), loader: env.vf.bootstrap_loader.clone() };
    let object_type = UnifiedType::Class(object);
    let object_array = UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::new(object_type.clone()) });
    let next_frame= can_pop(&env.vf, stack_frame, vec![object_type,UnifiedType::IntType, object_array])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))

}

pub fn instruction_is_type_safe_astore(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = store_is_type_safe(env,index,&UnifiedType::Reference,stack_frame)?;
    //todo dup
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_bastore(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let array_type = nth1_operand_stack_is(3,stack_frame)?;
    is_small_array(array_type)?;
    let next_frame = can_pop(&env.vf, stack_frame, vec![UnifiedType::IntType, UnifiedType::IntType, UnifiedType::TopType])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))
}

pub fn is_small_array(array_type: UnifiedType) -> Result<(),TypeSafetyError> {
    dbg!(&array_type);
    match array_type {
        UnifiedType::NullType => Result::Ok(()),
        UnifiedType::ArrayReferenceType(a) => match &a.sub_type.deref() {
            UnifiedType::ByteType => Result::Ok(()),
            UnifiedType::BooleanType => Result::Ok(()),
            _ => Result::Err(unknown_error_verifying!())
        },
        _ => Result::Err(unknown_error_verifying!())
    }
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_castore(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
//    unimplemented!()
//}

//#[allow(unused)]
//pub fn instruction_is_type_safe_dastore(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
//    unimplemented!()
//}

pub fn instruction_is_type_safe_dstore(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame= store_is_type_safe(env,index,&UnifiedType::DoubleType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_fastore(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
//    unimplemented!()
//}

pub fn instruction_is_type_safe_fstore(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = store_is_type_safe(env,index,&UnifiedType::FloatType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))
}
//
//#[allow(unused)]
//pub fn instruction_is_type_safe_iastore(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
//    unimplemented!()
//}

pub fn instruction_is_type_safe_istore(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = store_is_type_safe(env,index,&UnifiedType::IntType,stack_frame)?;
    //todo dup
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_lastore(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
//    unimplemented!()
//}


pub fn instruction_is_type_safe_lstore(index: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
    let next_frame = store_is_type_safe(env,index,&UnifiedType::LongType,stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames{ next_frame, exception_frame }))
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_sastore(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe,TypeSafetyError> {
//    unimplemented!()
//}
