use std::option::Option::Some;
use std::rc::Rc;

use itertools::Itertools;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::*;
use rust_jvm_common::vtype::VType;

use crate::{StackMap, VerifierContext};
use crate::OperandStack;
use crate::verifier::{ClassWithLoaderMethod, Frame, get_class};
use crate::verifier::filecorrectness::{does_not_override_final_method, is_assignable, super_class_chain};
use crate::verifier::instructions::{FrameResult, handlers_are_legal};
use crate::verifier::instructions::merged_code_is_type_safe;
use crate::verifier::stackmapframes::get_stack_map_frames;
use crate::verifier::TypeSafetyError;

pub fn valid_type_transition(env: &Environment, expected_types_on_stack: Vec<VType>, result_type: VType, input_frame: Frame) -> Result<Frame, TypeSafetyError> {
    let Frame { locals, stack_map: input_operand_stack, flag_this_uninit } = input_frame;
    let interim_operand_stack = pop_matching_list(&env.vf, input_operand_stack, expected_types_on_stack)?;
    let next_operand_stack = push_operand_stack(&env.vf, interim_operand_stack, &result_type);
    if operand_stack_has_legal_length(env, &next_operand_stack) {
        Result::Ok(Frame { locals, stack_map: next_operand_stack, flag_this_uninit })
    } else {
        Result::Err(TypeSafetyError::NotSafe("Operand stack did not have legal length".to_string()))
    }
}

pub fn pop_matching_list(vf: &VerifierContext, pop_from: OperandStack, pop: Vec<VType>) -> Result<OperandStack, TypeSafetyError> {
    pop_matching_list_impl(vf, pop_from, pop.as_slice())
}

pub fn pop_matching_list_impl(vf: &VerifierContext, mut pop_from: OperandStack, pop: &[VType]) -> Result<OperandStack, TypeSafetyError> {
    if pop.is_empty() {
        Result::Ok(pop_from)
    } else {
        let to_pop = pop.first().unwrap();
        pop_matching_type(vf, &mut pop_from, to_pop)?;
        pop_matching_list_impl(vf, pop_from, &pop[1..])
    }
}

pub fn pop_matching_type<'l>(vf: &VerifierContext, operand_stack: &'l mut OperandStack, type_: &VType) -> Result<VType, TypeSafetyError> {
    if size_of(vf, type_) == 1 {
        let actual_type = operand_stack.peek();
        is_assignable(vf, &actual_type, type_)?;
        operand_stack.operand_pop();
        Result::Ok(actual_type)
    } else if size_of(vf, type_) == 2 {
        assert!(matches!(&operand_stack.peek(), VType::TopType));
        let top = operand_stack.operand_pop();
        let actual_type = &operand_stack.peek();
        if let Err(err) = is_assignable(vf, actual_type, type_) {
            operand_stack.operand_push(top);
            return Err(err);
        };
        operand_stack.operand_pop();
        Result::Ok(actual_type.clone())
    } else {
        panic!()
    }
}


pub fn size_of(vf: &VerifierContext, unified_type: &VType) -> u64 {
    match unified_type {
        VType::TopType => { 1 }
        _ => {
            if is_assignable(vf, unified_type, &VType::TwoWord).is_ok() {
                2
            } else if is_assignable(vf, unified_type, &VType::OneWord).is_ok() {
                1
            } else {
                panic!("This is a bug")
            }
        }
    }
}

pub fn push_operand_stack(vf: &VerifierContext, mut operand_stack: OperandStack, type_: &VType) -> OperandStack {
    match type_ {
        VType::VoidType => {
            operand_stack
        }
        _ => {
            if size_of(vf, type_) == 2 {
                operand_stack.operand_push(type_.clone());
                operand_stack.operand_push(VType::TopType);
            } else if size_of(vf, type_) == 1 {
                operand_stack.operand_push(type_.clone());
            } else {
                panic!("It's impossible to have something which isn't size 1 or 2")
            }
            operand_stack
        }
    }
}


pub fn operand_stack_has_legal_length(environment: &Environment, operand_stack: &OperandStack) -> bool {
    operand_stack.len() <= environment.max_stack as usize
}

pub fn can_pop(vf: &VerifierContext, input_frame: Frame, types: Vec<VType>) -> Result<Frame, TypeSafetyError> {
    let Frame { locals, stack_map, flag_this_uninit } = input_frame;
    let poped_stack = pop_matching_list(vf, stack_map, types)?;
    Result::Ok(Frame {
        locals,
        stack_map: poped_stack,
        flag_this_uninit,
    })
}

pub fn frame_is_assignable(vf: &VerifierContext, left: &Frame, right: &Frame) -> Result<(), TypeSafetyError> {
    let locals_assignable_res: Result<Vec<_>, _> = left.locals.iter().zip(right.locals.iter()).map(|(left_, right_)| {
        is_assignable(vf, left_, right_)
    }).collect();
    let locals_assignable = locals_assignable_res.is_ok();
    let stack_assignable_res: Result<Vec<_>, _> = left.stack_map.iter().zip(right.stack_map.iter()).map(|(left_, right_)| {
        is_assignable(vf, left_, right_)
    }).collect();
    let stack_assignable = stack_assignable_res.is_ok();
    if left.stack_map.len() == right.stack_map.len() && locals_assignable && stack_assignable &&
        if left.flag_this_uninit {
            true
            // right.flag_this_uninit//todo wtf going on here
        } else {
            true
        } {
        Result::Ok(())
    } else {
        dbg!(left.stack_map.len() == right.stack_map.len());
        dbg!(locals_assignable);
        dbg!(stack_assignable);
        Result::Err(unknown_error_verifying!())
    }
}

pub fn method_is_type_safe(vf: &mut VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let method_class = get_class(vf, &method.class);
    let method_view = method_class.method_view_i(method.method_index as u16);
    does_not_override_final_method(vf, class, method)?;
    if method_view.is_native() || method_view.is_abstract() {
        Result::Ok(())
    } else {
        method_with_code_is_type_safe(vf, class.clone(), method.clone())
    }
}

pub struct ParsedCodeAttribute {
    pub method: ClassWithLoaderMethod,
}

pub fn get_handlers(_vf: &VerifierContext, _class: &ClassWithLoader, code: &CompressedCode) -> Vec<Handler> {
    code.exception_table.iter().map(|f| Handler {
        start: f.start_pc as u16,
        end: f.end_pc as u16,
        target: f.handler_pc as u16,
        class_name: f.catch_type,
    }).collect()
}

pub fn method_with_code_is_type_safe<'l, 'k>(vf: &'l mut VerifierContext<'k>, class: ClassWithLoader, method: ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let method_class = get_class(vf, &class);
    let method_info = &method_class.method_view_i(method.method_index as u16);
    let code = method_info.code_attribute().unwrap();
    let frame_size = code.max_locals;
    let max_stack = code.max_stack;
    let mut final_offset = 0;
    let mut instructs: Vec<&CInstruction> = code.instructions
        .iter()
        .sorted_by_key(|(offset, _)| *offset)
        .map(|(offset, x)| {
            assert_eq!(*offset, x.offset);
            final_offset = *offset;
            x
        })
        .collect();
    let end_of_code = CInstruction { offset: final_offset, instruction_size: 0, info: CompressedInstructionInfo::EndOfCode };
    instructs.push(&end_of_code);
    let handlers = get_handlers(vf, &class, code);
    let stack_map: Vec<StackMap> = get_stack_map_frames(vf, &class, method_info);
    let merged = merge_stack_map_and_code(instructs, stack_map.iter().collect());
    let (frame, return_type) = method_initial_stack_frame(vf, &class, &method, frame_size)?;
    let mut env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.clone(), handlers, return_type, vf };
    handlers_are_legal(&env)?;
    merged_code_is_type_safe(&mut env, merged.as_slice(), FrameResult::Regular(frame))?;
    Result::Ok(())
}

#[derive(Debug)]
pub struct Handler {
    pub start: u16,
    pub end: u16,
    pub target: u16,
    pub class_name: Option<CClassName>,
}

pub fn handler_exception_class(_vf: &VerifierContext, handler: &Handler, loader: LoaderName) -> ClassWithLoader {
    //may want to return a unifiedType instead
    match &handler.class_name {
        None => { ClassWithLoader { class_name: CClassName::throwable(), loader: LoaderName::BootstrapLoader } }
        Some(s) => {
            ClassWithLoader { class_name: *s, loader: loader.clone() }
        }
    }
}

pub fn init_handler_is_legal(_env: &Environment, _handler: &Handler) -> Result<(), TypeSafetyError> {
    return Result::Ok(());
    // if not_init_handler(&_env.vf, _env, _handler) {
    //     todo!()
    // } else {
    //     todo!()
    // }
}

pub fn not_init_handler(_vf: &VerifierContext, _env: &Environment, _handler: &Handler) -> bool {
    unimplemented!()
}

//#[allow(unused)]
//pub fn is_init_handler(vf:&VerifierContext,env: &Environment, handler: &Handler) -> bool {
//    unimplemented!()
//}

pub enum UnifiedInstruction {}

#[allow(dead_code)]
pub struct Environment<'l, 'k> {
    pub method: ClassWithLoaderMethod,
    pub return_type: VType,
    pub frame_size: u16,
    pub max_stack: u16,
    pub merged_code: Option<&'l Vec<MergedCodeInstruction<'l>>>,
    pub class_loader: LoaderName,
    pub handlers: Vec<Handler>,
    pub vf: &'l mut VerifierContext<'k>,
}

#[derive(Debug)]
pub enum MergedCodeInstruction<'l> {
    Instruction(&'l CInstruction),
    StackMap(&'l StackMap),
}

fn merge_stack_map_and_code_impl<'l>(instructions: &[&'l CInstruction], stack_maps: &[&'l StackMap], res: &mut Vec<MergedCodeInstruction<'l>>) {
    if stack_maps.is_empty() {
        res.append(&mut instructions.iter().map(|x| MergedCodeInstruction::Instruction(x)).collect());
        return;
    }
    let stack_map = stack_maps.first().unwrap();
    let instruction = instructions.first().unwrap_or_else(|| unimplemented!());
    if stack_map.offset == instruction.offset {
        res.push(MergedCodeInstruction::StackMap(stack_map));
        res.push(MergedCodeInstruction::Instruction(instruction));
        merge_stack_map_and_code_impl(&instructions[1..], &stack_maps[1..], res);
    } else if instruction.offset < stack_map.offset {
        res.push(MergedCodeInstruction::Instruction(instruction));
        merge_stack_map_and_code_impl(&instructions[1..], stack_maps, res);
    } else {
        dbg!(stack_map.offset);
        dbg!(instruction.offset);
        dbg!(instructions);
        dbg!(stack_maps);
        unimplemented!()
    }
}

/**
assumes that stackmaps and instructions are ordered
 */
pub fn merge_stack_map_and_code<'l>(instruction: Vec<&'l CInstruction>, stack_maps: Vec<&'l StackMap>) -> Vec<MergedCodeInstruction<'l>> {
    let mut res = vec![];
    merge_stack_map_and_code_impl(instruction.as_slice(), stack_maps.as_slice(), &mut res);
    res
}

fn method_initial_stack_frame(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod, frame_size: u16) -> Result<(Frame, VType), TypeSafetyError> {
    let classfile = get_class(vf, class);
    let method_view = &classfile.method_view_i(method.method_index as u16);
    let parsed_descriptor = method_view.desc().clone();
    let this_list = method_initial_this_type(vf, class, method)?;
    let flag_this_uninit = flags(&this_list);
    //todo this long and frequently duped
    let args = expand_type_list(vf, parsed_descriptor.arg_types
        .iter()
        .map(|x| x.to_verification_type(vf.current_loader))
        .collect());//todo need to solve loader situation
    let mut this_args = vec![];
    this_list.iter().for_each(|x| {
        this_args.push(x.clone());
    });
    args.iter().for_each(|x| {
        this_args.push(x.clone())
    });
    let locals = Rc::new(expand_to_length_verification(this_args, frame_size as usize, VType::TopType));
    Ok((Frame { locals, flag_this_uninit, stack_map: OperandStack::empty() }, parsed_descriptor.return_type.to_verification_type(vf.current_loader)))
}


fn expand_type_list(vf: &VerifierContext, list: Vec<VType>) -> Vec<VType> {
    return list.iter().flat_map(|x| {
        if size_of(vf, x) == 1 {
            vec![x.clone()]
        } else {
            assert_eq!(size_of(vf, x), 2);
            vec![x.clone(), VType::TopType]
        }
    }).collect();
}

fn flags(this_list: &Option<VType>) -> bool {
    match this_list {
        None => false,
        Some(s) => matches!(s, VType::UninitializedThis)
    }
}


pub fn expand_to_length(list: Vec<VType>, size: usize, filler: VType) -> Vec<VType> {
    assert!(list.len() <= size);
    let mut res = vec![];
    for i in 0..size {
        res.push(match list.get(i) {
            None => { filler.clone() }
            Some(elem) => { elem.clone() }
        })
    }
    res
}


fn expand_to_length_verification(list: Vec<VType>, size: usize, filler: VType) -> Vec<VType> {
    assert!(list.len() <= size);
    let mut res = vec![];
    for i in 0..size {
        res.push(match list.get(i) {
            None => { filler.clone() }
            Some(elem) => { elem.clone() }
        })
    }
    res
}


fn method_initial_this_type(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<Option<VType>, TypeSafetyError> {
    let method_class = get_class(vf, &method.class);
    let method_view = method_class.method_view_i(method.method_index as u16);
    if method_view.is_static() {
        let method_name = method_view.name();
        if method_name != MethodName::constructor_init() {//todo convert init to classname like string/id
            Ok(None)
        } else {
            Err(unknown_error_verifying!())
        }
    } else {
        Ok(Some(instance_method_initial_this_type(vf, class, method)?))
    }
}

fn instance_method_initial_this_type(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<VType, TypeSafetyError> {
    let classfile = get_class(vf, &method.class);
    let method_name = classfile.method_view_i(method.method_index as u16).name();
    if method_name == MethodName::constructor_init() {
        if class.class_name == CClassName::object() {
            Result::Ok(VType::Class(ClassWithLoader { class_name: class.class_name, loader: class.loader.clone() }))
        } else {
            let mut chain = vec![];
            super_class_chain(vf, class, class.loader.clone(), &mut chain)?;
            if !chain.is_empty() {
                Result::Ok(VType::UninitializedThis)
            } else {
                Result::Err(unknown_error_verifying!())
            }
        }
    } else {
        Result::Ok(get_class(vf, class).name().to_verification_type(class.loader))
    }
}
