use crate::verifier::{Frame, ClassWithLoaderMethod, get_class};
use crate::verifier::filecorrectness::{does_not_override_final_method, is_assignable, super_class_chain};
use crate::verifier::stackmapframes::get_stack_map_frames;
use crate::verifier::instructions::{handlers_are_legal, FrameResult};
use crate::verifier::instructions::merged_code_is_type_safe;

use std::option::Option::Some;
use rust_jvm_common::classfile::{InstructionInfo, Instruction, Code};
use crate::verifier::TypeSafetyError;
use crate::{StackMap, VerifierContext};
use rust_jvm_common::classnames::ClassName;
use crate::OperandStack;
use std::ops::Deref;
use classfile_view::vtype::VType;
use classfile_view::view::HasAccessFlags;
use classfile_view::loading::*;
use classfile_view::view::ptype_view::PTypeView;
use classfile_view::view::constant_info_view::ConstantInfoView;
use descriptor_parser::MethodDescriptor;
use std::rc::Rc;

pub fn valid_type_transition(env: &Environment, expected_types_on_stack: Vec<VType>, result_type: &VType, input_frame: Frame) -> Result<Frame, TypeSafetyError> {
    let Frame { locals,  stack_map:input_operand_stack, flag_this_uninit } = input_frame;
    let interim_operand_stack = pop_matching_list(&env.vf, input_operand_stack, expected_types_on_stack)?;
    let next_operand_stack = push_operand_stack(&env.vf, interim_operand_stack, &result_type);
    if operand_stack_has_legal_length(env, &next_operand_stack) {
        Result::Ok(Frame { locals, stack_map: next_operand_stack, flag_this_uninit })
    } else {
        Result::Err(TypeSafetyError::NotSafe("Operand stack did not have legal length".to_string()))
    }
}

pub fn pop_matching_list(vf: &VerifierContext, pop_from: OperandStack, pop: Vec<VType>) -> Result<OperandStack, TypeSafetyError> {
    let result = pop_matching_list_impl(vf, pop_from, pop.as_slice());
    return result;
}

pub fn pop_matching_list_impl(vf: &VerifierContext, mut pop_from: OperandStack, pop: &[VType]) -> Result<OperandStack, TypeSafetyError> {
    if pop.is_empty() {
        Result::Ok(pop_from)//todo inefficent copying
    } else {
        let to_pop = pop.first().unwrap();
        pop_matching_type(vf, &mut pop_from, to_pop)?;
        return pop_matching_list_impl(vf, pop_from, &pop[1..]);
    }
}

pub fn pop_matching_type<'l>(vf: &VerifierContext, operand_stack: &'l mut OperandStack, type_: &VType) -> Result<VType, TypeSafetyError> {
    if size_of(vf, type_) == 1 {
        let actual_type = operand_stack.peek();
        is_assignable(vf, &actual_type, type_)?;
        operand_stack.operand_pop();
        return Result::Ok(actual_type.clone());
    } else if size_of(vf, type_) == 2 {
        assert!(match &operand_stack.peek() {
            VType::TopType => true,
            _ => false
        });
        operand_stack.operand_pop();
        let actual_type = &operand_stack.peek();
        //todo if not assignable we need to roll back top pop
        is_assignable(vf, actual_type, type_).unwrap();
        operand_stack.operand_pop();
        return Result::Ok(actual_type.clone());
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
                unimplemented!()
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
    if left.stack_map.len() == right.stack_map.len() && locals_assignable && stack_assignable /*&&
        if left.flag_this_uninit {
            right.flag_this_uninit
        } else {
            true//todo realisitically I shouldn't check this b/c no way of knowing from stackmapframes.
        }*/ {
        Result::Ok(())
    } else {
        dbg!(locals_assignable);
        dbg!(stack_assignable);
        dbg!(left);
        dbg!(right);
        panic!();
//        Result::Err(unknown_error_verifying!())
    }
}

pub fn method_is_type_safe(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let method_class = get_class(vf, method.class);
    let method_view = method_class.method_view_i(method.method_index as usize);
    // dbg!(method_view.name());
    does_not_override_final_method(vf, class, method)?;
    if method_view.is_native() {
        Result::Ok(())
    } else if method_view.is_abstract() {
        Result::Ok(())
    } else {
        //will have a code attribute. or else method_with_code_is_type_safe will crash todo
        /*let attributes = get_attributes(class, method);
        attributes.iter().any(|_| {
            unimplemented!()
        }) && */method_with_code_is_type_safe(vf, class, method)
    }
}

pub struct ParsedCodeAttribute<'l> {
    //    pub class_name: NameReference,
//    pub frame_size: u16,
//    pub max_stack: u16,
//    pub code: &'l Vec<Instruction>,
//    pub exception_table: Vec<Handler>,
//    todo
//    pub stackmap_frames: Vec<&'l StackMap<'l>>,//todo
    pub method: &'l ClassWithLoaderMethod<'l>
}

pub fn get_handlers(vf: &VerifierContext, class: &ClassWithLoader, code: &Code) -> Vec<Handler> {
    code.exception_table.iter().map(|f| Handler {
        start: f.start_pc as usize,
        end: f.end_pc as usize,
        target: f.handler_pc as usize,
        class_name: if f.catch_type == 0 { None } else {
            let classfile = get_class(vf, class);
            let catch_type_name = match &classfile.constant_pool_view(f.catch_type as usize) {
                ConstantInfoView::Class(c) => {
                    c.class_name()
                }
                _ => panic!()
            };
            Some(catch_type_name.unwrap_name())
        },
    }).collect()
}

pub fn method_with_code_is_type_safe(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let method_class = get_class(vf, class);
    let method_info = &method_class.method_view_i(method.method_index);
    let code = method_info.code_attribute().unwrap();//todo add CodeView
    let frame_size = code.max_locals;
    let max_stack = code.max_stack;
    let mut final_offset = 0;
    let mut instructs: Vec<&Instruction> = code.code
        .iter()
        .map(|x| {
            *(&mut final_offset) = x.offset;
            x
        })
        .collect();
    let end_of_code = Instruction { offset: final_offset, instruction: InstructionInfo::EndOfCode };
    instructs.push(&end_of_code);
    let handlers = get_handlers(vf, class, code);
    let stack_map: Vec<StackMap> = get_stack_map_frames(vf, class, method_info);
    let merged = merge_stack_map_and_code(instructs, stack_map.iter().map(|x| { x }).collect());
    let (frame, return_type) = method_initial_stack_frame(vf, class, method, frame_size);
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.clone(), handlers, return_type, vf: vf.clone() };
    handlers_are_legal(&env)?;
    merged_code_is_type_safe(&env, merged.as_slice(), FrameResult::Regular(frame))?;/*{
        Ok(_) => Result::Ok(()),
        Err(_) => {
            //then maybe we need to try alternate initial_this_type
            let (frame, return_type) = method_initial_stack_frame(vf, class, method, frame_size,true);
            let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.clone(), handlers, return_type, vf: vf.clone() };
            handlers_are_legal(&env)?;
            merged_code_is_type_safe(&env, merged.as_slice(), FrameResult::Regular(&frame))
        },
    }*/
    Result::Ok(())
}

#[derive(Debug)]
pub struct Handler {
    pub start: usize,
    pub end: usize,
    pub target: usize,
    pub class_name: Option<ClassName>,
}

pub fn handler_exception_class(vf: &VerifierContext, handler: &Handler, loader: LoaderArc) -> ClassWithLoader {
    //may want to return a unifiedType instead
    match &handler.class_name {
        None => { ClassWithLoader { class_name: ClassName::throwable(), loader: vf.bootstrap_loader.clone() } }
        Some(s) => {
//            let _classfile = loader.pre_load(loader.clone(),s).unwrap();
            // then class in question exists
            // todo compare against throwable , but not here ?
            ClassWithLoader { class_name: s.clone(), loader: loader.clone() }
        }
    }
}
//

pub fn init_handler_is_legal(_env: &Environment, _handler: &Handler) -> Result<(), TypeSafetyError> {
    unimplemented!()
}
//
//#[allow(unused)]
//pub fn not_init_handler(vf:&VerifierContext,env: &Environment, handler: &Handler) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn is_init_handler(vf:&VerifierContext,env: &Environment, handler: &Handler) -> bool {
//    unimplemented!()
//}

pub enum UnifiedInstruction {}

#[allow(dead_code)]
pub struct Environment<'l> {
    pub method: &'l ClassWithLoaderMethod<'l>,
    pub return_type: VType,
    pub frame_size: u16,
    pub max_stack: u16,
    pub merged_code: Option<&'l Vec<MergedCodeInstruction<'l>>>,
    pub class_loader: LoaderArc,
    pub handlers: Vec<Handler>,
    pub vf: VerifierContext,
}

#[derive(Debug)]
pub enum MergedCodeInstruction<'l> {
    Instruction(&'l Instruction),
    StackMap(&'l StackMap),
}

fn merge_stack_map_and_code_impl<'l>(instructions: &[&'l Instruction], stack_maps: &[&'l StackMap], res: &mut Vec<MergedCodeInstruction<'l>>) {
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
        unimplemented!()
    }
}

/**
assumes that stackmaps and instructions are ordered
*/
pub fn merge_stack_map_and_code<'l>(instruction: Vec<&'l Instruction>, stack_maps: Vec<&'l StackMap>) -> Vec<MergedCodeInstruction<'l>> {
//    trace!("Starting instruction and stackmap merge");
    let mut res = vec![];
    merge_stack_map_and_code_impl(instruction.as_slice(), stack_maps.as_slice(), &mut res);
    return res;
}

fn method_initial_stack_frame(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod, frame_size: u16) -> (Frame, VType) {
    let classfile = get_class(vf, class);
    let method_view = &classfile.method_view_i(method.method_index as usize);
    let initial_parsed_descriptor = method_view.desc();
    let parsed_descriptor = MethodDescriptor {
        parameter_types: initial_parsed_descriptor.parameter_types.clone(),
        return_type: initial_parsed_descriptor.return_type.clone(),
    };
    let this_list = method_initial_this_type(vf, class, method);
    let flag_this_uninit = flags(&this_list);
    //todo this long and frequently duped
    let args = expand_type_list(vf, parsed_descriptor.parameter_types
        .iter()
        .map(|x| PTypeView::from_ptype(&x).to_verification_type(&vf.bootstrap_loader))
        .collect());//todo need to solve loader situation
    let mut this_args = vec![];
    this_list.iter().for_each(|x| {
        this_args.push(x.clone());
    });
    args.iter().for_each(|x| {
        this_args.push(x.clone())
    });
    let locals = Rc::new(expand_to_length_verification(this_args, frame_size as usize, VType::TopType));
    return (Frame { locals, flag_this_uninit, stack_map: OperandStack::empty() }, PTypeView::from_ptype(&parsed_descriptor.return_type).to_verification_type(&vf.bootstrap_loader));
}


fn expand_type_list(vf: &VerifierContext, list: Vec<VType>) -> Vec<VType> {
    return list.iter().flat_map(|x| {
        if size_of(vf, x) == 1 {
            vec![x.clone()]
        } else {
            assert!(size_of(vf, x) == 2);
            vec![x.clone(), VType::TopType]
        }
    }).collect();
}

fn flags(this_list: &Option<VType>) -> bool {
    match this_list {
        None => false,
        Some(s) => match s {
            VType::UninitializedThis => true,
            _ => false
        }
    }
}


pub fn expand_to_length(list: Vec<PTypeView>, size: usize, filler: PTypeView) -> Vec<PTypeView> {
    assert!(list.len() <= size);
    let mut res = vec![];
    for i in 0..size {
        res.push(match list.get(i) {
            None => { filler.clone() }
            Some(elem) => { elem.clone() }
        })
    }
    return res;
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
    return res;
}


fn method_initial_this_type(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Option<VType> {
    let method_class = get_class(vf, method.class);
    let method_view = method_class.method_view_i(method.method_index);
    if method_view.is_static() {
        //todo dup
        let classfile = &method_class;
        let method_info = &classfile.method_view_i(method.method_index);
        let method_name_ = method_info.name();
        let method_name = method_name_.deref();
        if method_name != "<init>" {
            return None;
        } else {
            unimplemented!()
        }
    } else {
        Some(instance_method_initial_this_type(vf, class, method).unwrap())
    }
}

fn instance_method_initial_this_type(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<VType, TypeSafetyError> {
    let classfile = get_class(vf, method.class);
    let method_name_ = classfile.method_view_i(method.method_index).name();
    let method_name = method_name_.deref();
    if method_name == "<init>" {
        if class.class_name == ClassName::object() {
            Result::Ok(VType::Class(ClassWithLoader { class_name: get_class(vf, class).name(), loader: class.loader.clone() }))
        } else {
            let mut chain = vec![];
            super_class_chain(vf, class, class.loader.clone(), &mut chain)?;
            if !chain.is_empty() {
                Result::Ok(VType::UninitializedThis)
            } else {
                unimplemented!()
            }
        }
    } else {
        Result::Ok(VType::Class(ClassWithLoader { class_name: get_class(vf, class).name(), loader: class.loader.clone() }))
    }
}
