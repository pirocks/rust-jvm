use rust_jvm_common::unified_types::UnifiedType;
use crate::verifier::instructions::{InstructionTypeSafe, AfterGotoFrames, exception_stack_frame, target_is_type_safe, ResultFrames};
use crate::verifier::codecorrectness::{Environment, can_pop, MergedCodeInstruction};
use crate::verifier::{Frame, get_class};
use crate::verifier::TypeSafetyError;
use rust_jvm_common::classfile::{ConstantKind, InstructionInfo, UninitializedVariableInfo};
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::utils::name_and_type_extractor;

use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::passes_protected_check;
use rust_jvm_common::utils::extract_class_from_constant_pool;
use crate::verifier::codecorrectness::valid_type_transition;
use rust_jvm_common::classnames::{ClassName, NameReference};
use crate::verifier::filecorrectness::is_assignable;
use std::sync::Arc;
use crate::verifier::codecorrectness::translate_types_to_vm_types;
use crate::OperandStack;
use classfile_parser::types::Descriptor;
use classfile_parser::types::parse_method_descriptor;
use classfile_parser::types::MethodDescriptor;
use classfile_parser::types::parse_field_descriptor;
use rust_jvm_common::classnames::get_referred_name;

pub fn instruction_is_type_safe_return(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    match env.return_type {
        UnifiedType::VoidType => {}
        _ => { return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string())); }
    };
    if stack_frame.flag_this_uninit {
        return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string()));
    }
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames {
        exception_frame
    }))
}


pub fn instruction_is_type_safe_if_acmpeq(target: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![UnifiedType::Reference, UnifiedType::Reference])?;
    target_is_type_safe(env, &next_frame, target as usize)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


pub fn instruction_is_type_safe_goto(target: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    target_is_type_safe(env, stack_frame, target as usize)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}


pub fn instruction_is_type_safe_ireturn(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    match env.return_type {
        UnifiedType::IntType => {}
        _ => return Result::Err(TypeSafetyError::NotSafe("Tried to return not an int with ireturn".to_string()))
    }
    can_pop(&env.vf, stack_frame, vec![UnifiedType::IntType])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}


pub fn instruction_is_type_safe_areturn(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let return_type = &env.return_type;
    is_assignable(&env.vf, return_type, &UnifiedType::Reference)?;
    can_pop(&env.vf, stack_frame, vec![return_type.clone()])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}


pub fn instruction_is_type_safe_if_icmpeq(target: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![UnifiedType::IntType, UnifiedType::IntType])?;
    target_is_type_safe(env, &next_frame, target)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_ifeq(target: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![UnifiedType::IntType])?;
    target_is_type_safe(env, &next_frame, target)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_ifnonnull(target: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let next_frame = can_pop(&env.vf, stack_frame, vec![UnifiedType::Reference])?;
    target_is_type_safe(env, &next_frame, target)?;
    //todo dup with above
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_invokedynamic(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let method_class = get_class(&env.vf, env.method.class);
    let constant_pool = &method_class.constant_pool;
    let (name_index, descriptor_index) = match &constant_pool[cp].kind {
        ConstantKind::InvokeDynamic(i) => {
            match &constant_pool[i.name_and_type_index as usize].kind {
                ConstantKind::NameAndType(nt) => (nt.name_index as usize, nt.descriptor_index as usize),
                _ => panic!()
            }
        }
        _ => panic!()
    };
    let call_site_name = extract_string_from_utf8(&constant_pool[name_index]);
    let descriptor_string = extract_string_from_utf8(&constant_pool[descriptor_index]);
    let descriptor = parse_method_descriptor(&env.class_loader, descriptor_string.as_str()).unwrap();
    if call_site_name == "<init>" || call_site_name == "<clinit>" {
        return Result::Err(TypeSafetyError::NotSafe("Tried to invoke dynamic in constructor".to_string()));
    }
    let mut operand_arg_list: Vec<UnifiedType> = descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    let return_type = translate_types_to_vm_types(&descriptor.return_type);
    operand_arg_list.reverse();
    let stack_arg_list = operand_arg_list;
    let next_frame = valid_type_transition(env, stack_arg_list, &return_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

pub fn instruction_is_type_safe_invokeinterface(cp: usize, count: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let method_class = get_class(&env.vf, env.method.class);
    let constant_pool = &method_class.constant_pool;
    let ((name_index, descriptor_index), class_index) = match &constant_pool[cp].kind {
        ConstantKind::InterfaceMethodref(i) => {
            (match &constant_pool[i.nt_index as usize].kind {
                ConstantKind::NameAndType(nt) => {
                    (nt.name_index as usize, nt.descriptor_index as usize)
                }
                _ => panic!()
            }, i.class_index as usize)
        }
        _ => panic!()
    };
    let method_intf_name = extract_string_from_utf8(match &constant_pool[class_index].kind {
        ConstantKind::Class(c) => { &constant_pool[c.name_index as usize] }
        _ => panic!()
    });
    let method_name = extract_string_from_utf8(&constant_pool[name_index]);
    let descriptor_string = extract_string_from_utf8(&constant_pool[descriptor_index]);
    let descriptor = parse_method_descriptor(&env.class_loader, descriptor_string.as_str()).unwrap();
    if method_name == "<init>" || method_name == "<clinit>" {
        return Result::Err(TypeSafetyError::NotSafe("Tried to invoke interface on constructor".to_string()));
    }
    let mut operand_arg_list: Vec<UnifiedType> = descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    let return_type = translate_types_to_vm_types(&descriptor.return_type);
    let current_loader = env.class_loader.clone();
    //todo this is almost certainly wrong.
    operand_arg_list.push(UnifiedType::Class(ClassWithLoader { class_name: ClassName::Str(method_intf_name), loader: current_loader }));
//    operand_arg_list.reverse();
    let stack_arg_list = operand_arg_list;
    let temp_frame = can_pop(&env.vf, stack_frame, stack_arg_list)?;
    let next_frame = valid_type_transition(env, vec![], &return_type, &temp_frame)?;
    count_is_valid(count, stack_frame, &temp_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}

fn count_is_valid(count: usize, input_frame: &Frame, output_frame: &Frame) -> Result<(), TypeSafetyError> {
    let len1 = input_frame.stack_map.len();
    let len2 = output_frame.stack_map.len();
    if count == len1 - len2 {
        Result::Ok(())
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_invokespecial(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    let method_class_name = match method_class_type {
        UnifiedType::Class(c) => c.class_name,
        _ => panic!()
    };
    if method_name == "<init>" {
        invoke_special_init(&env, stack_frame, &method_class_name, &parsed_descriptor)
    } else {
        invoke_special_not_init(env, stack_frame, get_referred_name(&method_class_name), method_name, &parsed_descriptor)
    }
}

fn invoke_special_init(env: &Environment, stack_frame: &Frame, method_class_name: &ClassName, parsed_descriptor: &MethodDescriptor) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let mut stack_arg_list: Vec<_> = parsed_descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    stack_arg_list.reverse();
    dbg!(&stack_arg_list);
    let temp_frame = can_pop(&env.vf, stack_frame, stack_arg_list)?;
    let locals = temp_frame.locals;
    let mut operand_stack = temp_frame.stack_map.clone();
    let first = operand_stack.operand_pop();
    let flags = temp_frame.flag_this_uninit;
    let current_class_loader = env.class_loader.clone();
//    match first {
//        None => unimplemented!(),
//        Some(u) => {
    dbg!(&operand_stack);
    match first {
        UnifiedType::Uninitialized(address) => {
            let uninit_address = UnifiedType::Uninitialized(UninitializedVariableInfo { offset: address.offset });
            let this = rewritten_uninitialized_type(&uninit_address, env, &ClassWithLoader { class_name: method_class_name.clone(), loader: current_class_loader })?;
            let next_flags = rewritten_initialization_flags(&uninit_address, flags);
            let this_class = UnifiedType::Class(this);
            let next_operand_stack = substitute_operand_stack(&uninit_address, &this_class, &operand_stack);
            let next_locals = substitute(&uninit_address, &this_class, locals.as_slice());
            let next_stack_frame = Frame {
                locals: next_locals,
                stack_map: next_operand_stack,
                flag_this_uninit: next_flags,
            };
            let exception_stack_frame = Frame {
                locals,
                stack_map: OperandStack::empty(),
                flag_this_uninit: flags,
            };
            passes_protected_check(env, &method_class_name.clone(), "<init>".to_string(), Descriptor::Method(&parsed_descriptor), &next_stack_frame)?;
            Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame: next_stack_frame, exception_frame: exception_stack_frame }))
        }
        UnifiedType::UninitializedThis => {
            let this = rewritten_uninitialized_type(&UnifiedType::UninitializedThis, env, &ClassWithLoader { class_name: method_class_name.clone(), loader: current_class_loader })?;
            let flag_this_uninit = rewritten_initialization_flags(&UnifiedType::UninitializedThis, flags);
            let this_class = UnifiedType::Class(this);
            let next_operand_stack = substitute_operand_stack(&UnifiedType::UninitializedThis, &this_class, &operand_stack);
            let next_locals = substitute(&UnifiedType::UninitializedThis, &this_class, locals.as_slice());
            //todo duplication with above
            let next_stack_frame = Frame {
                locals: next_locals,
                stack_map: next_operand_stack,
                flag_this_uninit,
            };
            let exception_stack_frame = Frame {
                locals,
                stack_map: OperandStack::empty(),
                flag_this_uninit: flags,
            };
            Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame: next_stack_frame, exception_frame: exception_stack_frame }))
        }
        _ => panic!(),
    }
//        }
//    }
}

pub fn substitute(old: &UnifiedType, new: &UnifiedType, list: &[UnifiedType]) -> Vec<UnifiedType> {
    list.iter().map(|x| (if old == x {
        new
    } else {
        x
    }).clone()).collect()
}

pub fn substitute_operand_stack(old: &UnifiedType, new: &UnifiedType, list: &OperandStack) -> OperandStack {
    let mut o = list.clone();
    o.substitute(old, new);
    o
}

fn rewritten_initialization_flags(type_: &UnifiedType, flag_this_uninit: bool) -> bool {
    match type_ {
        UnifiedType::Uninitialized(_) => flag_this_uninit,
        UnifiedType::UninitializedThis => false,
        _ => panic!()
    }
}

fn rewritten_uninitialized_type(type_: &UnifiedType, env: &Environment, _class: &ClassWithLoader) -> Result<ClassWithLoader, TypeSafetyError> {
    match type_ {
        UnifiedType::Uninitialized(address) => {
            match env.merged_code {
                None => unimplemented!(),
                Some(code) => {
                    let found_new = code.iter().find(|x| {
                        match x {
                            MergedCodeInstruction::Instruction(i) => {
                                i.offset == address.offset as usize && match i.instruction {
                                    InstructionInfo::new(_this) => true,
                                    _ => { unimplemented!() }
                                }
                            }
                            MergedCodeInstruction::StackMap(_) => false,
                        }
                    });
                    match found_new {
                        None => unimplemented!(),
                        Some(new_this) => match new_this {
                            MergedCodeInstruction::Instruction(instr) => match instr.instruction {
                                InstructionInfo::new(this) => {
                                    let method_class = get_class(&env.vf, env.method.class);
                                    match &method_class.constant_pool[this as usize].kind {
                                        ConstantKind::Class(c) => {
                                            let class_name = ClassName::Ref(NameReference {
                                                class_file: Arc::downgrade(&method_class),
                                                index: c.name_index,
                                            });
                                            return Result::Ok(ClassWithLoader { class_name, loader: env.class_loader.clone() });
                                        }
                                        _ => { unimplemented!() }
                                    }
                                }
                                _ => panic!()
                            },
                            MergedCodeInstruction::StackMap(_) => panic!(),
                        },
                    }
                }
            }
        }
        UnifiedType::UninitializedThis => {
            //todo there needs to be some weird retry logic here/in invoke_special b/c This is not strictly a return value in the prolog class, and there is a more complex
            // version of this branch which would be triggered by verificaion failure for this invoke special.
            Result::Ok(ClassWithLoader { class_name: env.method.class.class_name.clone(), loader: env.method.class.loader.clone() })
        }
        _ => { panic!() }
    }
}

fn invoke_special_not_init(env: &Environment, stack_frame: &Frame, method_class_name: String, method_name: String, parsed_descriptor: &MethodDescriptor) -> Result<InstructionTypeSafe, TypeSafetyError> {
    if method_name == "<clinit>" {
        return Result::Err(TypeSafetyError::NotSafe("invoke special on clinit is not allowed".to_string()));
    }
    let current_class_name = env.method.class.class_name.clone();
    let current_loader = env.method.class.loader.clone();
    let current_class = UnifiedType::Class(ClassWithLoader {
        class_name: current_class_name,
        loader: current_loader.clone(),
    });
    let method_class = UnifiedType::Class(ClassWithLoader {
        class_name: ClassName::Str(method_class_name),
        loader: current_loader.clone(),
    });
    is_assignable(&env.vf, &current_class, &method_class)?;
    let mut operand_arg_list_copy: Vec<_> = parsed_descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    operand_arg_list_copy.push(current_class);
    operand_arg_list_copy.reverse();
    let return_type = translate_types_to_vm_types(&parsed_descriptor.return_type);
    let next_frame = valid_type_transition(env, operand_arg_list_copy, &return_type, stack_frame)?;
    let mut operand_arg_list_copy2: Vec<_> = parsed_descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    operand_arg_list_copy2.push(method_class);
    operand_arg_list_copy2.reverse();
    valid_type_transition(env, operand_arg_list_copy2, &return_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    return Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame }));
}

pub fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (_class_name, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    let operand_arg_list: Vec<UnifiedType> = parsed_descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    let stack_arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| x.clone())
        .collect();
    let return_type = translate_types_to_vm_types(&parsed_descriptor.return_type);
    let next_frame = valid_type_transition(env, stack_arg_list, &return_type, stack_frame)?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame }))
}

pub fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (class_type, method_name, parsed_descriptor) = get_method_descriptor(cp, env);
    let (class_name, method_class) = match class_type {
        UnifiedType::Class(c) => (Some(c.class_name.clone()), UnifiedType::Class(c.clone())),
        UnifiedType::ArrayReferenceType(a) => {
            (None, UnifiedType::ArrayReferenceType(a))
        }
        _ => panic!()
    };

    if method_name.contains("arrayOf") || method_name.contains("[") || method_name == "<init>" || method_name == "<clinit>" {
        unimplemented!();
    }
    // the operand_arg list is in the order displayed by the spec, e.g first elem a 0.
    let operand_arg_list: &Vec<UnifiedType> = &parsed_descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect();
    // arg list is the reversed verison of operand_arg_list
    let arg_list: Vec<UnifiedType> = operand_arg_list.iter()
        .rev()
        .map(|x| x.clone())
        .collect();
    let mut stack_arg_list: Vec<UnifiedType> = arg_list.clone();
    stack_arg_list.push(method_class);
    let return_type = translate_types_to_vm_types(&parsed_descriptor.return_type);
    let nf = valid_type_transition(env, stack_arg_list.clone(), &return_type, stack_frame)?;
    let popped_frame = can_pop(&env.vf, stack_frame, arg_list)?;
    if class_name.is_some() {
        dbg!(stack_arg_list);
        dbg!(&popped_frame);
        dbg!(&nf);
        dbg!(&class_name.clone().unwrap());
        dbg!(Descriptor::Method(&parsed_descriptor));
        passes_protected_check(env, &class_name.unwrap(), method_name, Descriptor::Method(&parsed_descriptor), &popped_frame)?;
    }
    let exception_stack_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame: exception_stack_frame, next_frame: nf }))
}

fn get_method_descriptor(cp: usize, env: &Environment) -> (UnifiedType, String, MethodDescriptor) {
    let classfile = &get_class(&env.vf, env.method.class);
    let c = &classfile.constant_pool[cp].kind;
    let (class_name, method_name, parsed_descriptor) = match c {
        ConstantKind::Methodref(m) => {
            let c = extract_class_from_constant_pool(m.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            //todo ideally for name we would return weak ref.
            let (method_name, descriptor) = name_and_type_extractor(m.name_and_type_index, classfile);
            let parsed_descriptor = match parse_method_descriptor(&env.class_loader, descriptor.as_str()) {
                None => { unimplemented!() }
                Some(pd) => { pd }
            };
            (class_name, method_name, parsed_descriptor)
        }
        ConstantKind::InterfaceMethodref(m) => {
            //todo dup?
            let c = extract_class_from_constant_pool(m.class_index, &classfile);
            let class_name = extract_string_from_utf8(&classfile.constant_pool[c.name_index as usize]);
            let (method_name, descriptor) = name_and_type_extractor(m.nt_index, classfile);
            let parsed_descriptor = match parse_method_descriptor(&env.class_loader, descriptor.as_str()) {
                None => { unimplemented!() }
                Some(pd) => { pd }
            };
            (class_name, method_name, parsed_descriptor)
        }
        _ => unimplemented!("{:?}",c)
    };
    (possibly_array_to_type(env, class_name), method_name, parsed_descriptor)
}

pub fn possibly_array_to_type(env: &Environment, class_name: String) -> UnifiedType {
    if class_name.contains("[") {
        let class_type = match parse_field_descriptor(&env.class_loader, class_name.as_str()) {
            None => panic!(),
            Some(s) => s.field_type,
        };
        class_type
    } else {
        UnifiedType::Class(ClassWithLoader {
            class_name: ClassName::Str(class_name),
            loader: env.class_loader.clone(),
        })
    }
}

//#[allow(unused)]
//pub fn instruction_is_type_safe_lreturn(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>  {
//    unimplemented!()
//}
//#[allow(unused)]
//pub fn instruction_is_type_safe_dreturn(env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>  {
//    unimplemented!()
//}

pub fn instruction_is_type_safe_freturn(env: &Environment, _offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError>  {
    if env.return_type != UnifiedType::FloatType{
        return Result::Err(unknown_error_verifying!());
    }
    can_pop(&env.vf,stack_frame,vec![UnifiedType::FloatType])?;
    let exception_frame = exception_stack_frame(stack_frame);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}
