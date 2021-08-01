use std::ops::Deref;
use std::rc::Rc;

use classfile_view::view::ClassView;
use classfile_view::view::constant_info_view::ConstantInfoView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classfile::UninitializedVariableInfo;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedClassfileStringPool, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::CInstructionInfo;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::descriptor_parser::{Descriptor, parse_field_descriptor};
use rust_jvm_common::loading::ClassWithLoader;
use rust_jvm_common::vtype::VType;

use crate::OperandStack;
use crate::verifier::{Frame, get_class, standard_exception_frame};
use crate::verifier::codecorrectness::{can_pop, Environment, MergedCodeInstruction, push_operand_stack};
use crate::verifier::codecorrectness::valid_type_transition;
use crate::verifier::filecorrectness::is_assignable;
use crate::verifier::instructions::{AfterGotoFrames, exception_stack_frame, InstructionTypeSafe, ResultFrames, target_is_type_safe};
use crate::verifier::passes_protected_check;
use crate::verifier::TypeSafetyError;

pub fn instruction_is_type_safe_return(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    match env.return_type {
        VType::VoidType => {}
        _ => { return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string())); }
    };
    if stack_frame.flag_this_uninit {
        return Result::Err(TypeSafetyError::NotSafe("todo messsage".to_string()));
    }
    let exception_frame = exception_stack_frame(stack_frame.locals.clone(), stack_frame.flag_this_uninit);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames {
        exception_frame
    }))
}

pub fn type_safe_if_cmp(target: u16, env: &Environment, stack_frame: Frame, comparison_types: Vec<VType>) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = can_pop(&env.vf, stack_frame, comparison_types)?;
    target_is_type_safe(env, &next_frame, target)?;
    standard_exception_frame(locals, flag, next_frame)
}


pub fn instruction_is_type_safe_if_acmpeq(target: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_safe_if_cmp(target, env, stack_frame, vec![VType::Reference, VType::Reference])
}


pub fn instruction_is_type_safe_goto(target: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    target_is_type_safe(env, &stack_frame, target)?;
    let exception_frame = exception_stack_frame(stack_frame.locals.clone(), stack_frame.flag_this_uninit);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}

pub fn instruction_is_type_safe_ireturn(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    //todo is ireturn used for shorts etc?
    //what should a method return type be?
    match env.return_type {
        VType::IntType => {}
        _ => return Result::Err(TypeSafetyError::NotSafe("Tried to return not an int with ireturn".to_string()))
    }
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    can_pop(&env.vf, stack_frame, vec![VType::IntType])?;
    let exception_frame = exception_stack_frame(locals, flag);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}

pub fn instruction_is_type_safe_areturn(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let return_type = &env.return_type;
    is_assignable(&env.vf, return_type, &VType::Reference)?;
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    can_pop(&env.vf, stack_frame, vec![return_type.clone()])?;
    let exception_frame = exception_stack_frame(locals, flag);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}


pub fn instruction_is_type_safe_if_icmpeq(target: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_safe_if_cmp(target, env, stack_frame, vec![VType::IntType, VType::IntType])
}

pub fn instruction_is_type_safe_ifeq(target: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_safe_if_cmp(target, env, stack_frame, vec![VType::IntType])
}

pub fn instruction_is_type_safe_ifnonnull(target: u16, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    type_safe_if_cmp(target, env, stack_frame, vec![VType::Reference])
}

pub fn instruction_is_type_safe_invokedynamic(cp: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let method_class = get_class(&env.vf, &env.method.class)?;
    let (call_site_name, descriptor) = match &method_class.constant_pool_view(cp) {
        ConstantInfoView::InvokeDynamic(i) => {
            (MethodName(i.name_and_type().name(env.vf.string_pool)), i.name_and_type().desc_method(env.vf.string_pool))
        }
        _ => panic!()
    };
    if call_site_name == MethodName::constructor_init() || call_site_name == MethodName::constructor_clinit() {
        return Result::Err(TypeSafetyError::NotSafe("Tried to invoke dynamic in constructor".to_string()));
    }
    let operand_arg_list: Vec<VType> = descriptor.arg_types.iter().rev().map(|x| { x.to_verification_type(env.class_loader) }).collect();
    let return_type = descriptor.return_type.to_verification_type(env.class_loader);
    let stack_arg_list = operand_arg_list;
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let next_frame = valid_type_transition(env, stack_arg_list, return_type, stack_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

pub fn instruction_is_type_safe_invokeinterface(method_name: MethodName, descriptor: &CMethodDescriptor, ref_type: &CPRefType, count: usize, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    if method_name == MethodName::constructor_init() || method_name == MethodName::constructor_clinit() {
        return Result::Err(TypeSafetyError::NotSafe("Tried to invoke interface on constructor".to_string()));
    }
    let mut operand_arg_list: Vec<_> = descriptor.arg_types.iter().rev().map(|x| { x.to_verification_type(env.class_loader) }).collect();
    let return_type = descriptor.return_type.to_verification_type(env.class_loader);
    let current_loader = env.class_loader.clone();
    operand_arg_list.push(CPDType::Ref(ref_type.clone()).to_verification_type(current_loader));
    let stack_arg_list = operand_arg_list;
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let stack_frame_size = stack_frame.stack_map.len();
    let temp_frame = can_pop(&env.vf, stack_frame, stack_arg_list)?;
    count_is_valid(count, stack_frame_size, &temp_frame)?;
    let next_frame = valid_type_transition(env, vec![], return_type, temp_frame)?;
    standard_exception_frame(locals, flag, next_frame)
}

fn count_is_valid(count: usize, input_frame_stack_map_size: usize, output_frame: &Frame) -> Result<(), TypeSafetyError> {
    let len1 = input_frame_stack_map_size;
    let len2 = output_frame.stack_map.len();
    if count == len1 - len2 {
        Result::Ok(())
    } else {
        Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_invokespecial(method_class_type: &CPDType, method_name: MethodName, parsed_descriptor: &CMethodDescriptor, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let method_class_name = match method_class_type {
        CPDType::Ref(CPRefType::Class(c)) => c,
        _ => panic!()
    };
    if method_name == MethodName::constructor_init() {
        invoke_special_init(&env, stack_frame, *method_class_name, parsed_descriptor)
    } else {
        invoke_special_not_init(env, stack_frame, *method_class_name, method_name, parsed_descriptor)
    }
}

fn invoke_special_init(env: &Environment, stack_frame: Frame, method_class_name: CClassName, parsed_descriptor: &CMethodDescriptor) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let mut stack_arg_list: Vec<_> = parsed_descriptor.arg_types.iter().map(|x| { x.to_verification_type(env.class_loader) }).collect();
    stack_arg_list.reverse();
    let temp_frame = can_pop(&env.vf, stack_frame, stack_arg_list)?;
    let locals = temp_frame.locals;
    let mut operand_stack = temp_frame.stack_map.clone();
    let first = operand_stack.operand_pop();
    let flags = temp_frame.flag_this_uninit;
    let current_class_loader = env.class_loader.clone();
    let exception_frame = Frame {
        locals: locals.clone(),
        stack_map: OperandStack::empty(),
        flag_this_uninit: flags,
    };
    match first {
        VType::Uninitialized(address) => {
            let uninit_address = VType::Uninitialized(UninitializedVariableInfo { offset: address.offset });
            let this = rewritten_uninitialized_type(&uninit_address, env, &ClassWithLoader { class_name: method_class_name, loader: current_class_loader })?;
            let next_flags = rewritten_initialization_flags(&uninit_address, flags);
            let this_class = VType::Class(this);
            let next_operand_stack = substitute_operand_stack(&uninit_address, &this_class, &operand_stack);
            let next_locals = substitute(&uninit_address, &this_class, locals.as_slice());
            let next_frame = Frame {
                locals: Rc::new(next_locals),
                stack_map: next_operand_stack,
                flag_this_uninit: next_flags,
            };
            passes_protected_check(env, method_class_name, MethodName::constructor_init().0, Descriptor::Method(&parsed_descriptor), &next_frame)?;
            Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
        }
        VType::UninitializedThis => {
            let this = rewritten_uninitialized_type(&VType::UninitializedThis, env, &ClassWithLoader { class_name: method_class_name, loader: current_class_loader })?;
            let flag_this_uninit = rewritten_initialization_flags(&VType::UninitializedThis, flags);
            let this_class = VType::Class(this);
            let next_operand_stack = substitute_operand_stack(&VType::UninitializedThis, &this_class, &operand_stack);
            let next_locals = substitute(&VType::UninitializedThis, &this_class, locals.as_slice());
            let next_frame = Frame {
                locals: Rc::new(next_locals),
                stack_map: next_operand_stack,
                flag_this_uninit,
            };
            Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
        }
        _ => panic!(),
    }
}

pub fn substitute(old: &VType, new: &VType, list: &[VType]) -> Vec<VType> {
    list.iter().map(|x| (if old == x {
        new
    } else {
        x
    }).clone()).collect()
}

pub fn substitute_operand_stack(old: &VType, new: &VType, list: &OperandStack) -> OperandStack {
    let mut o = list.clone();
    o.substitute(old, new);
    o
}

fn rewritten_initialization_flags(type_: &VType, flag_this_uninit: bool) -> bool {
    match type_ {
        VType::Uninitialized(_) => flag_this_uninit,
        VType::UninitializedThis => false,
        _ => panic!()
    }
}

fn rewritten_uninitialized_type(type_: &VType, env: &Environment, _class: &ClassWithLoader) -> Result<ClassWithLoader, TypeSafetyError> {
    match type_ {
        VType::Uninitialized(address) => {
            match env.merged_code {
                None => unimplemented!(),
                Some(code) => {
                    let found_new = code.iter().find(|x| {
                        match x {
                            MergedCodeInstruction::Instruction(i) => {
                                i.offset == address.offset as u16 && match &i.info {
                                    CInstructionInfo::new(_this) => true,
                                    _ => { unimplemented!() }
                                }
                            }
                            MergedCodeInstruction::StackMap(_) => false,
                        }
                    });
                    match found_new {
                        None => unimplemented!(),
                        Some(new_this) => match new_this {
                            MergedCodeInstruction::Instruction(instr) => match &instr.info {
                                CInstructionInfo::new(class_name) => {
                                    Result::Ok(ClassWithLoader { class_name: *class_name, loader: env.class_loader.clone() })
                                }
                                _ => panic!()
                            },
                            MergedCodeInstruction::StackMap(_) => panic!(),
                        },
                    }
                }
            }
        }
        VType::UninitializedThis => {
            //todo there needs to be some weird retry logic here/in invoke_special b/c This is not strictly a return value in the prolog class, and there is a more complex
            // version of this branch which would be triggered by verificaion failure for this invoke special.
            Result::Ok(ClassWithLoader { class_name: env.method.class.class_name.clone(), loader: env.method.class.loader.clone() })
        }
        _ => { panic!() }
    }
}

fn invoke_special_not_init(env: &Environment, stack_frame: Frame, method_class_name: CClassName, method_name: MethodName, parsed_descriptor: &CMethodDescriptor) -> Result<InstructionTypeSafe, TypeSafetyError> {
    if method_name == MethodName::constructor_clinit() {
        return Result::Err(TypeSafetyError::NotSafe("invoke special on clinit is not allowed".to_string()));
    }
    let current_class_name = env.method.class.class_name.clone();
    let current_loader = env.method.class.loader.clone();
    let current_class = VType::Class(ClassWithLoader {
        class_name: current_class_name,
        loader: current_loader.clone(),
    });
    let method_class = VType::Class(ClassWithLoader {
        class_name: method_class_name,
        loader: current_loader.clone(),
    });
    // is_assignable(&env.vf, &current_class, &method_class)?;//the spec puts this here, but I think the spec is wrong
    let mut operand_arg_list_copy: Vec<_> = parsed_descriptor.arg_types.iter().rev().map(|x| {
        x.to_verification_type(env.class_loader)
    }).collect();
    operand_arg_list_copy.push(current_class);
    let return_type = parsed_descriptor.return_type.to_verification_type(env.class_loader);
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let mut operand_arg_list_copy2: Vec<_> = parsed_descriptor.arg_types
        .iter().rev()
        .map(|x| { x.to_verification_type(env.class_loader) })
        .collect();
    operand_arg_list_copy2.push(method_class);
    let next_frame = valid_type_transition(env, operand_arg_list_copy2, return_type, stack_frame)?;
    let exception_frame = exception_stack_frame(locals, flag);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { exception_frame, next_frame }))
}

pub fn instruction_is_type_safe_invokestatic(method_name: MethodName, parsed_descriptor: &CMethodDescriptor, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let method_name_str = method_name.0.to_str(env.vf.string_pool);
    if method_name_str.contains("arrayOf") || method_name_str.contains('[') || method_name == MethodName::constructor_init() || method_name == MethodName::constructor_clinit() {
        unimplemented!();
    }
    let operand_arg_list: Vec<_> = parsed_descriptor.arg_types.iter().map(|x| x.to_verification_type(env.class_loader)).collect();

    //todo redundant?
    let stack_arg_list: Vec<_> = operand_arg_list.iter()
        .rev()
        .cloned()
        .collect();
    let return_type = parsed_descriptor.return_type.to_verification_type(env.class_loader);
    if method_name == MethodName::method_linkToStatic() || method_name == MethodName::method_linkToVirtual() {
        //todo should handle polymorphism better
        let mut next_stack_frame = stack_frame.stack_map.clone();
        stack_arg_list.iter().for_each(|_| {
            next_stack_frame.operand_pop();//todo do check object
        });
        next_stack_frame = push_operand_stack(&env.vf, next_stack_frame, &return_type);
        standard_exception_frame(stack_frame.locals.clone(), stack_frame.flag_this_uninit, Frame {
            locals: stack_frame.locals.clone(),
            stack_map: next_stack_frame,
            flag_this_uninit: stack_frame.flag_this_uninit,
        })
    } else {
        let locals = stack_frame.locals.clone();
        let flag = stack_frame.flag_this_uninit;
        let next_frame = valid_type_transition(env, stack_arg_list, return_type, stack_frame)?;
        standard_exception_frame(locals, flag, next_frame)
    }
}

pub fn instruction_is_type_safe_invokevirtual(class_type: &CPDType, method_name: MethodName, parsed_descriptor: &CMethodDescriptor, env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let (class_name, method_class) = match class_type {
        CPDType::Ref(r) => {
            match r {
                CPRefType::Class(c) => (Some(c.clone()), VType::Class(ClassWithLoader { class_name: *c, loader: env.class_loader.clone() })),
                CPRefType::Array(a) => {
                    (None, VType::ArrayReferenceType(a.deref().clone()))
                }
            }
        }
        _ => panic!()
    };

    if method_name == MethodName::constructor_init() || method_name == MethodName::constructor_clinit() {
        unimplemented!();
    }
    // the operand_arg list is in the order displayed by the spec, e.g first elem a 0.
    let operand_arg_list: &Vec<_> = &parsed_descriptor.arg_types.iter().map(|x| x.to_verification_type(env.class_loader)).collect();
    // arg list is the reversed verison of operand_arg_list
    let arg_list: Vec<_> = operand_arg_list.iter()
        .rev()
        .cloned()
        .collect();
    let mut stack_arg_list: Vec<_> = arg_list.clone();
    stack_arg_list.push(method_class);
    let return_type = parsed_descriptor.return_type.to_verification_type(env.class_loader);//todo what should the loader here be?
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    let nf = valid_type_transition(env, stack_arg_list, return_type, stack_frame.clone())?;
    let popped_frame = can_pop(&env.vf, stack_frame, arg_list)?;//todo the above is an unneeded clone b/c this is equivalent/redundant?
    if class_name.is_some() {
        passes_protected_check(env, class_name.unwrap(), method_name.0, Descriptor::Method(&parsed_descriptor), &popped_frame)?;
    }
    standard_exception_frame(locals, flag, nf)
}

pub fn get_method_descriptor(pool: &CompressedClassfileStringPool, cp: usize, classfile: &dyn ClassView) -> (CPDType, MethodName, CMethodDescriptor) {
    let c = &classfile.constant_pool_view(cp);
    let (ref_type, method_name, parsed_descriptor) = match c {
        ConstantInfoView::Methodref(m) => {
            let ref_type = m.class(pool);
            let (method_name, descriptor) = (m.name_and_type().name(pool), m.name_and_type().desc_method(pool));
            (ref_type, MethodName(method_name), descriptor)
        }
        ConstantInfoView::InterfaceMethodref(m) => {
            let ref_type = m.class();
            let (method_name, descriptor) = (m.name_and_type().name(pool), m.name_and_type().desc_method(pool));
            (ref_type, MethodName(method_name), descriptor)
        }
        _ => todo!()
    };
    (CPDType::Ref(ref_type), method_name, parsed_descriptor)
}

pub fn possibly_array_to_type(class_name: &str) -> ReferenceTypeView {
    if class_name.contains('[') {
        let class_type = match parse_field_descriptor(class_name) {
            None => panic!(),
            Some(s) => s.field_type,
        };
        ReferenceTypeView::Array(Box::new(PTypeView::from_ptype(&class_type.unwrap_array_type())))
    } else {
        ReferenceTypeView::Class(ClassName::Str(class_name.to_string()))
    }
}

pub fn instruction_is_type_safe_lreturn(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    match env.return_type {
        VType::LongType => {
            let locals = stack_frame.locals.clone();
            let flag = stack_frame.flag_this_uninit;
            can_pop(&env.vf, stack_frame, vec![VType::LongType])?;
            let exception_frame = exception_stack_frame(locals, flag);
            Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
        }
        _ => Result::Err(unknown_error_verifying!())
    }
}

pub fn instruction_is_type_safe_dreturn(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    if env.return_type != VType::DoubleType {
        return Result::Err(unknown_error_verifying!());
    }
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    can_pop(&env.vf, stack_frame, vec![VType::DoubleType])?;
    let exception_frame = exception_stack_frame(locals, flag);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}

pub fn instruction_is_type_safe_freturn(env: &Environment, stack_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    if env.return_type != VType::FloatType {
        return Result::Err(unknown_error_verifying!());
    }
    let locals = stack_frame.locals.clone();
    let flag = stack_frame.flag_this_uninit;
    can_pop(&env.vf, stack_frame, vec![VType::FloatType])?;
    let exception_frame = exception_stack_frame(locals, flag);
    Result::Ok(InstructionTypeSafe::AfterGoto(AfterGotoFrames { exception_frame }))
}
