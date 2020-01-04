use log::trace;
use crate::verifier::{Frame, ClassWithLoaderMethod, get_class};
use crate::verifier::filecorrectness::{does_not_override_final_method, is_assignable, super_class_chain};
use crate::verifier::codecorrectness::stackmapframes::get_stack_map_frames;
use std::sync::Arc;
use crate::verifier::instructions::{handers_are_legal, FrameResult};
use crate::verifier::instructions::merged_code_is_type_safe;
use crate::types::{parse_method_descriptor, MethodDescriptor};

use std::option::Option::Some;
use rust_jvm_common::unified_types::{UnifiedType, ArrayType, ClassWithLoader};
use rust_jvm_common::classfile::{InstructionInfo, Instruction, ACC_NATIVE, ACC_ABSTRACT, Code, ACC_STATIC};
use rust_jvm_common::classnames::{NameReference, class_name, get_referred_name};
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::loading::Loader;
use classfile_parser::code_attribute;
use crate::verifier::TypeSafetyError;
use crate::verifier::filecorrectness::get_access_flags;
use rust_jvm_common::utils::method_name;
use crate::{StackMap, VerifierContext};
use rust_jvm_common::classnames::ClassName;
use crate::OperandStack;

pub mod stackmapframes;


pub fn valid_type_transition(env: &Environment, expected_types_on_stack: Vec<UnifiedType>, result_type: &UnifiedType, input_frame: &Frame) -> Result<Frame, TypeSafetyError> {
//    dbg!(&expected_types_on_stack);
//    dbg!(&result_type);
//    dbg!(&input_frame);
    let input_operand_stack = &input_frame.stack_map;
    let interim_operand_stack = pop_matching_list(&env.vf,input_operand_stack, expected_types_on_stack)?;
    let next_operand_stack = push_operand_stack(&env.vf,&interim_operand_stack, &result_type);
    if operand_stack_has_legal_length(env, &next_operand_stack) {
        Result::Ok(Frame { locals: input_frame.locals.iter().map(|x| x.clone()).collect(), stack_map: next_operand_stack, flag_this_uninit: input_frame.flag_this_uninit })
    } else {
        Result::Err(TypeSafetyError::NotSafe("Operand stack did not have legal length".to_string()))
    }
}

//IMPORTANT NOTE:
// lists are stored in same order as prolog, e.g. 0 is first elem, n-1 is last.
// This is problematic for adding to the beginning of a Vec. as a result linked lists may be used in impl.
// alternatively results can be reversed at the end.
pub fn pop_matching_list(vf:&VerifierContext,pop_from: &OperandStack, pop: Vec<UnifiedType>) -> Result<OperandStack, TypeSafetyError> {
    let result = pop_matching_list_impl(vf,&mut pop_from.clone(), pop.as_slice());
    if pop_from.len() > 1 && pop.len() > 1{
        dbg!("Attempt to pop matching:");
        dbg!(&pop_from);
        dbg!(&pop);
        dbg!(&result);
    }
    return result;
}

pub fn pop_matching_list_impl(vf:&VerifierContext,pop_from: &mut OperandStack, pop: &[UnifiedType]) -> Result<OperandStack, TypeSafetyError> {
    if pop.is_empty() {
        Result::Ok(pop_from.clone())//todo inefficent copying
    } else {
        let to_pop = pop.first().unwrap();
        pop_matching_type(vf,pop_from, to_pop)?;
        return pop_matching_list_impl(vf,pop_from, &pop[1..]);
    }
}

pub fn pop_matching_type<'l>(vf:&VerifierContext,operand_stack: &'l mut  OperandStack, type_: &UnifiedType) -> Result<UnifiedType, TypeSafetyError> {
    if size_of(vf,type_) == 1 {
        let actual_type = operand_stack.peek();
        is_assignable(vf,&actual_type, type_)?;
        operand_stack.operand_pop();
        return Result::Ok(actual_type.clone());
    } else if size_of(vf,type_) == 2 {
        assert!(match &operand_stack.peek() {
            UnifiedType::TopType => true,
            _ => false
        });
        operand_stack.operand_pop();
        let actual_type = &operand_stack.peek();
        //todo if not assignable we need to roll back top pop
        is_assignable(vf,actual_type, type_).unwrap();
        operand_stack.operand_pop();
        return Result::Ok(actual_type.clone());
    } else {
        panic!()
    }
}


pub fn size_of(vf:&VerifierContext,unified_type: &UnifiedType) -> u64 {
    match unified_type {
        UnifiedType::TopType => { 1 }
        _ => {
            if is_assignable(vf,unified_type, &UnifiedType::TwoWord).is_ok() {
                2
            } else if is_assignable(vf,unified_type, &UnifiedType::OneWord).is_ok() {
                1
            } else {
                panic!("This is a bug")
            }
        }
    }
}

pub fn push_operand_stack(vf:&VerifierContext,operand_stack: &OperandStack, type_: &UnifiedType) -> OperandStack {
    let mut operand_stack_copy = operand_stack.clone();
    match type_ {
        UnifiedType::VoidType => {
            operand_stack_copy
        }
        _ => {
            if size_of(vf,type_) == 2 {
                operand_stack_copy.operand_push(type_.clone());
                operand_stack_copy.operand_push(UnifiedType::TopType);
            } else if size_of(vf,type_) == 1 {
                operand_stack_copy.operand_push(type_.clone());
            } else {
                unimplemented!()
            }
            operand_stack_copy
        }
    }
}


pub fn operand_stack_has_legal_length(environment: &Environment, operand_stack: &OperandStack) -> bool {
    operand_stack.len() <= environment.max_stack as usize
}
//
//#[allow(unused)]
//pub fn pop_category_1(types: Vec<UnifiedType>) -> Option<(UnifiedType, Vec<UnifiedType>)> {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn can_safely_push(environment: Environment, input_operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Option<Vec<UnifiedType>> {
//    unimplemented!();
//}
//
//#[allow(unused)]
//pub fn can_safely_push_list(environment: Environment, input_operand_stack: Vec<UnifiedType>, type_list: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn can_push_list(input_operand_stack: Vec<UnifiedType>, type_list: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
//    unimplemented!()
//}
//

pub fn can_pop(vf:&VerifierContext,input_frame: &Frame, types: Vec<UnifiedType>) -> Result<Frame, TypeSafetyError> {
    let poped_stack = pop_matching_list(vf,&input_frame.stack_map, types)?;
    Result::Ok(Frame {
        locals: input_frame
            .locals
            .iter()
            .map(|x| x.clone())
            .collect(),
        stack_map: poped_stack,
        flag_this_uninit: input_frame.flag_this_uninit,
    })
}

pub fn frame_is_assignable(vf:&VerifierContext,left: &Frame, right: &Frame) -> Result<(), TypeSafetyError> {
    dbg!(left);
    dbg!(right);
    let locals_assignable_res: Result<Vec<_>, _> = left.locals.iter().zip(right.locals.iter()).map(|(left_, right_)| {
        is_assignable(vf,left_, right_)
    }).collect();
    let locals_assignable = locals_assignable_res.is_ok();
    let stack_assignable_res: Result<Vec<_>, _> = left.stack_map.iter().zip(right.stack_map.iter()).map(|(left_, right_)| {
        is_assignable(vf,left_, right_)
    }).collect();
    let stack_assignable = stack_assignable_res.is_ok();
    if left.stack_map.len() == right.stack_map.len() && locals_assignable && stack_assignable &&
        if left.flag_this_uninit {
            right.flag_this_uninit
        } else {
            true
        } {
        Result::Ok(())
    } else {
        panic!();
        Result::Err(unknown_error_verifying!())
    }
}

pub fn method_is_type_safe(vf:&VerifierContext,class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let access_flags = get_access_flags(vf,class, method);
    trace!("got access_flags:{}", access_flags);
    does_not_override_final_method(vf,class, method)?;
    trace!("does not override final method");
//    dbg!(&does_not_override_final_method);

    if access_flags & ACC_NATIVE != 0 {
        trace!("method is native");
        Result::Ok(())
    } else if access_flags & ACC_ABSTRACT != 0 {
        trace!("method is abstract");
        Result::Ok(())
    } else {
        //will have a code attribute. or else method_with_code_is_type_safe will crash todo
        /*let attributes = get_attributes(class, method);
        attributes.iter().any(|_| {
            unimplemented!()
        }) && */method_with_code_is_type_safe(vf,class, method)
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

pub fn get_handlers(vf:&VerifierContext,class: &ClassWithLoader, code: &Code) -> Vec<Handler> {
    code.exception_table.iter().map(|f| {
        Handler {
            start: f.start_pc as usize,
            end: f.end_pc as usize,
            target: f.handler_pc as usize,
            class_name: if f.catch_type == 0 { None } else {
                Some(ClassName::Ref(NameReference {// should be a name as is currently b/c spec says so.
                    index: f.catch_type,
                    class_file: Arc::downgrade(&get_class(vf,class)),
                }))
            },
        }
    }).collect()
}

pub fn method_with_code_is_type_safe(vf:&VerifierContext,class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<(), TypeSafetyError> {
    let method_info = &get_class(vf,class).methods[method.method_index];
    let code = code_attribute(method_info).unwrap();
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
    let handlers = get_handlers(vf,class, code);
    let stack_map: Vec<StackMap> = get_stack_map_frames(vf,class, method_info);
    trace!("stack map frames generated:");
//    dbg!(&stack_map);
    let merged = merge_stack_map_and_code(instructs, stack_map.iter().map(|x| { x }).collect());
    trace!("stack map frames merged:");
//    dbg!(&merged);
    let (frame, return_type) = method_initial_stack_frame(vf,class, method, frame_size);
    trace!("Initial stack frame:");
//    dbg!(&frame);
    dbg!(&frame_size);
//    dbg!(&return_type);
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.clone(), handlers, return_type, vf: vf.clone() };
    handers_are_legal(&env)?;
    merged_code_is_type_safe(&env, merged.as_slice(), FrameResult::Regular(&frame))?;
    Result::Ok(())
}

#[derive(Debug)]
pub struct Handler {
    pub start: usize,
    pub end: usize,
    pub target: usize,
    pub class_name: Option<ClassName>
}

pub fn handler_exception_class(vf: &VerifierContext,handler: &Handler) -> ClassWithLoader {
    //may want to return a unifiedType instead
    match &handler.class_name {
        None => { ClassWithLoader{ class_name: ClassName::Str("java/lang/Throwable".to_string()), loader: vf.bootstrap_loader.clone() } }
        Some(_s) => { unimplemented!("Need to get class from state") }
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

//#[allow(unused)]
//pub fn is_applicable_instruction(vf:&VerifierContext,handler_start: u64, instruction: &UnifiedInstruction) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn no_attempt_to_return_normally(vf:&VerifierContext,instruction: &UnifiedInstruction) -> bool {
//    unimplemented!()
//}

#[allow(dead_code)]
pub struct Environment<'l> {
    pub method: &'l ClassWithLoaderMethod<'l>,
    pub return_type: UnifiedType,
    pub frame_size: u16,
    pub max_stack: u16,
    pub merged_code: Option<&'l Vec<MergedCodeInstruction<'l>>>,
    pub class_loader: Arc<dyn Loader + Send + Sync>,
    pub handlers: Vec<Handler>,
    pub vf : VerifierContext
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
    trace!("Starting instruction and stackmap merge");
    let mut res = vec![];
    merge_stack_map_and_code_impl(instruction.as_slice(), stack_maps.as_slice(), &mut res);
    return res;
}

pub fn translate_types_to_vm_types(type_: &UnifiedType) -> UnifiedType {
    match type_ {
        UnifiedType::ByteType => UnifiedType::IntType,
        UnifiedType::CharType => UnifiedType::IntType,
        UnifiedType::ShortType => UnifiedType::IntType,
        UnifiedType::DoubleType => UnifiedType::DoubleType,
        UnifiedType::FloatType => UnifiedType::FloatType,
        UnifiedType::IntType => UnifiedType::IntType,
        UnifiedType::BooleanType => UnifiedType::IntType,
        UnifiedType::LongType => UnifiedType::LongType,
        UnifiedType::Class(_) => type_.clone(),
        UnifiedType::ArrayReferenceType(a) => {
            let translated_subtype = translate_types_to_vm_types(&a.sub_type);
            UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::new(translated_subtype) })
        }
        UnifiedType::VoidType => UnifiedType::VoidType,
        UnifiedType::TopType => panic!(),
        UnifiedType::NullType => panic!(),
        UnifiedType::Uninitialized(_) => panic!(),
        UnifiedType::UninitializedThis => panic!(),
        UnifiedType::TwoWord => panic!(),
        UnifiedType::OneWord => panic!(),
        UnifiedType::Reference => panic!(),
        UnifiedType::UninitializedEmpty => panic!(),
    }
}

fn method_initial_stack_frame(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod, frame_size: u16) -> (Frame, UnifiedType) {
    //methodInitialStackFrame(Class, Method, FrameSize, frame(Locals, [], Flags),ReturnType):-
    //    methodDescriptor(Method, Descriptor),
    //    parseMethodDescriptor(Descriptor, RawArgs, ReturnType),
    //    expandTypeList(RawArgs, Args),
    //    methodInitialThisType(Class, Method, ThisList),
    //    flags(ThisList, Flags),
    //    append(ThisList, Args, ThisArgs),
    //    expandToLength(ThisArgs, FrameSize, top, Locals).
    let method_descriptor = extract_string_from_utf8(&get_class(vf,class).constant_pool[get_class(vf,method.prolog_class).methods[method.method_index as usize].descriptor_index as usize]);
    let initial_parsed_descriptor = parse_method_descriptor(&class.loader, method_descriptor.as_str()).unwrap();
    let parsed_descriptor = MethodDescriptor {
        parameter_types: initial_parsed_descriptor.parameter_types
            .iter()
            .map(|x| translate_types_to_vm_types(x))
            .collect(),
        return_type: translate_types_to_vm_types(&initial_parsed_descriptor.return_type),
    };
    let this_list = method_initial_this_type(vf,class, method);
    let flag_this_uninit = flags(&this_list);
    //todo this long and frequently duped
    let args = expand_type_list(vf,parsed_descriptor.parameter_types.iter().map(|x| translate_types_to_vm_types(x)).collect());
    let mut this_args = vec![];
    this_list.iter().for_each(|x| {
        this_args.push(x.clone());
    });
    args.iter().for_each(|x| {
        this_args.push(x.clone())
    });
    let locals = expand_to_length(this_args, frame_size as usize, UnifiedType::TopType);
    return (Frame { locals, flag_this_uninit, stack_map: OperandStack::empty() }, parsed_descriptor.return_type);
}


fn expand_type_list(vf: &VerifierContext, list: Vec<UnifiedType>) -> Vec<UnifiedType> {
    return list.iter().flat_map(|x| {
        if size_of(vf,x) == 1 {
            vec![x.clone()]
        } else {
            assert!(size_of(vf,x) == 2);
            vec![x.clone(), UnifiedType::TopType]
        }
    }).collect();
}

fn flags(this_list: &Option<UnifiedType>) -> bool {
    match this_list {
        None => false,
        Some(s) => match s {
            UnifiedType::UninitializedThis => true,
            _ => false
        }
    }
}


fn expand_to_length(list: Vec<UnifiedType>, size: usize, filler: UnifiedType) -> Vec<UnifiedType> {
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


fn method_initial_this_type(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Option<UnifiedType> {
    let method_access_flags = get_class(vf,method.prolog_class).methods[method.method_index].access_flags;
    if method_access_flags & ACC_STATIC > 0 {
        //todo dup
        let classfile = &get_class(vf,method.prolog_class);
        let method_name_info = &classfile.constant_pool[classfile.methods[method.method_index].name_index as usize];
        let method_name = extract_string_from_utf8(method_name_info);
        if method_name != "<init>" {
            return None;
        } else {
            unimplemented!()
        }
    } else {
        Some(instance_method_initial_this_type(vf,class, method).unwrap())
    }
//    return Some(UnifiedType::UninitializedThis);
}

fn instance_method_initial_this_type(vf: &VerifierContext, class: &ClassWithLoader, method: &ClassWithLoaderMethod) -> Result<UnifiedType, TypeSafetyError> {
    let method_name = method_name(&get_class(vf,method.prolog_class), &get_class(vf,method.prolog_class).methods[method.method_index]);
    if method_name == "<init>" {
        if get_referred_name(&class.class_name) == "java/lang/Object" {
            Result::Ok(UnifiedType::Class(ClassWithLoader { class_name: class_name(&get_class(vf,class)), loader: class.loader.clone() }))
        } else {
            let mut chain = vec![];
            super_class_chain(vf,class, class.loader.clone(), &mut chain)?;
            if !chain.is_empty() {
                Result::Ok(UnifiedType::UninitializedThis)
            } else {
                unimplemented!()
            }
        }
    } else {
        Result::Ok(UnifiedType::Class(ClassWithLoader { class_name: class_name(&get_class(vf,class)), loader: class.loader.clone() }))
    }
}
