use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::instructions::{InstructionIsTypeSafeResult, AfterGotoFrames, exception_stack_frame, target_is_type_safe, ResultFrames};
use crate::verifier::codecorrectness::{Environment, can_pop};
use crate::verifier::{Frame, get_class};
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classfile::ConstantKind;
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::utils::name_and_type_extractor;
use crate::types::MethodDescriptor;
use crate::verifier::codecorrectness::stackmapframes::copy_recurse;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::passes_protected_check;
use rust_jvm_common::utils::extract_class_from_constant_pool;
use crate::types::parse_method_descriptor;
use crate::verifier::codecorrectness::valid_type_transition;
use rust_jvm_common::classnames::ClassName;
use crate::verifier::filecorrectness::is_assignable;
use classfile_parser::code::InstructionTypeNum::return_;

pub fn instruction_is_type_safe_return(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    match env.return_type {
        UnifiedType::VoidType => {}
        _ => { return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string())); }
    };
    if stack_frame.flag_this_uninit {
        return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string()));
    }
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::AfterGoto(AfterGotoFrames {
        exception_frame
    }))
}


pub fn instruction_is_type_safe_if_acmpeq(target: isize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let next_frame = can_pop(stack_frame, vec![UnifiedType::Reference, UnifiedType::Reference])?;
    assert!(target >= 0);//todo shouldn't be an assert
    target_is_type_safe(env, &next_frame, target as usize)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_goto(target: isize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    assert!(target >= 0);//todo shouldn't be an assert
    target_is_type_safe(env, stack_frame, target as usize)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::AfterGoto(AfterGotoFrames { exception_frame }))
}


pub fn instruction_is_type_safe_ireturn(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    match env.return_type {
        UnifiedType::IntType => {}
        _ => return Result::Err(TypeSafetyError::NotSafe("Tried to return not an int with ireturn".to_string()))
    }
    can_pop(stack_frame, vec![UnifiedType::IntType])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::AfterGoto(AfterGotoFrames { exception_frame }))
}


//#[allow(unused)]
//fn instruction_is_type_safe_areturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}


//#[allow(unused)]
//fn instruction_is_type_safe_if_icmpeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ifeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_ifnonnull(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//


//#[allow(unused)]
//fn instruction_is_type_safe_invokedynamic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//fn instruction_is_type_safe_invokeinterface(cp: usize, count: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//
pub fn instruction_is_type_safe_invokespecial(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let (method_class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name == "<init>" {} else {
        if method_name == "<clinit>" {
            return Result::Err(TypeSafetyError::NotSafe("invoke special on clinit is not allowed".to_string()));
        }
        let current_class_name = env.method.prolog_class.class_name.clone();
        let current_loader = env.method.prolog_class.loader.clone();
        if !is_assignable(&UnifiedType::Class(ClassWithLoader {
            class_name: current_class_name,
            loader: current_loader.clone(),
        }), &UnifiedType::Class(ClassWithLoader {
            class_name: ClassName::Str(method_class_name),
            loader: current_loader.clone(),
        })){
            return Result::Err(TypeSafetyError::NotSafe("not assignable".to_string()));
        }
        let mut operand_arg_list_copy: Vec<_> = parsed_descriptor.parameter_types.iter().map(|x| copy_recurse(x)).collect();
        operand_arg_list_copy.push(UnifiedType::Class(ClassWithLoader {
            class_name: current_class_name,
            loader: current_loader.clone(),
        }));
        operand_arg_list_copy.reverse();
        let next_stack_frame = valid_type_transition(env,operand_arg_list_copy,&parsed_descriptor.return_type,stack_frame)?;
        let mut operand_arg_list_copy2: Vec<_> = parsed_descriptor.parameter_types.iter().map(|x| copy_recurse(x)).collect();
        operand_arg_list_copy2.push(UnifiedType::Class(ClassWithLoader {
            class_name: ClassName::Str(method_class_name),
            loader: current_loader.clone(),
        }));
        operand_arg_list_copy2.reverse();
        valid_type_transition(env,operand_arg_list_copy2,&parsed_descriptor.return_type,stack_frame)?;
        let exception_frame = exception_stack_frame(stack_frame);
        return Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame, next_frame }))
    }
}

pub fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let (_class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    let operand_arg_list = parsed_descriptor.parameter_types;
    let stack_arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| copy_recurse(x))
        .collect();
    let next_frame = match valid_type_transition(env, stack_arg_list, &parsed_descriptor.return_type, stack_frame) {
        Ok(nf) => nf,
        Err(_) => unimplemented!(),
    };
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame, next_frame }))
}


pub fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    let (class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    let operand_arg_list = &parsed_descriptor.parameter_types;
    let arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| copy_recurse(x))
        .collect();
    let current_loader = &env.class_loader;
//todo deal with loaders in class names/types
    let mut stack_arg_list: Vec<UnifiedType> = arg_list.iter().map(|x| copy_recurse(x)).collect();
    let class_type = ClassWithLoader { class_name: ClassName::Str(class_name.clone()), loader: current_loader.clone() };//todo better name
    stack_arg_list.push(UnifiedType::Class(class_type));
    stack_arg_list.reverse();
    let nf = valid_type_transition(env, stack_arg_list, &parsed_descriptor.return_type, stack_frame)?;
    let popped_frame = can_pop(stack_frame, arg_list)?;
    passes_protected_check(env, class_name.clone(), method_name, &parsed_descriptor, &popped_frame)?;
    let exception_stack_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionIsTypeSafeResult::Safe(ResultFrames { exception_frame: exception_stack_frame, next_frame: nf }))
}

fn get_method_descriptor(cp: usize, env: &Environment) -> (String, String, MethodDescriptor) {
    let classfile = &get_class(env.method.prolog_class);
    let c = &classfile.constant_pool[cp].kind;
    let (class_name, method_name, parsed_descriptor) = match c {
        ConstantKind::Methodref(m) => {
            let c = extract_class_from_constant_pool(m.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            let (method_name, descriptor) = name_and_type_extractor(m.name_and_type_index, classfile);
            let parsed_descriptor = match parse_method_descriptor(&env.class_loader, descriptor.as_str()) {
                None => { unimplemented!() }
                Some(pd) => { pd }
            };
            (class_name, method_name, parsed_descriptor)
        }
        _ => unimplemented!()
    };
    (class_name, method_name, parsed_descriptor)
}

//#[allow(unused)]
//fn instruction_is_type_safe_lreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
//#[allow(unused)]
//fn instruction_is_type_safe_dreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}

//#[allow(unused)]
//fn instruction_is_type_safe_freturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
//    unimplemented!()
//}
