use log::trace;
use classfile::{ACC_ABSTRACT, ACC_NATIVE, code_attribute};
use classfile::attribute_infos::Code;
use classfile::code::{Instruction, InstructionInfo};
use verification::classnames::{NameReference, get_referred_name};
use verification::code_writer::StackMap;
use verification::prolog_info_writer::{get_access_flags, method_name, class_name};
use verification::unified_type::UnifiedType;
use verification::verifier::{Frame, merge_type_safety_results, PrologClass, PrologClassMethod, TypeSafetyResult};
use verification::verifier::filecorrectness::{does_not_override_final_method, is_assignable, super_class_chain};
use verification::verifier::codecorrectness::stackmapframes::get_stack_map_frames;
use class_loading::Loader;
use std::sync::Arc;
use verification::verifier::instructions::{handers_are_legal, FrameResult};
use verification::verifier::instructions::merged_code_is_type_safe;
use verification::prolog_info_writer::extract_string_from_utf8;
use verification::types::parse_method_descriptor;
use verification::verifier::codecorrectness::stackmapframes::copy_recurse;
use classfile::ACC_STATIC;
use verification::verifier::and;
use std::option::Option::Some;

pub mod stackmapframes;


pub fn valid_type_transition(environment: &Environment, expected_types_on_stack: Vec<UnifiedType>, result_type: &UnifiedType, input_frame: &Frame) -> Result<Frame, TypeSafetyResult> {
    let input_operand_stack = &input_frame.stack_map;
    let interim_operand_stack = match pop_matching_list(input_operand_stack, expected_types_on_stack) {
        None => return Result::Err(TypeSafetyResult::NotSafe("could not pop matching list".to_string())),
        Some(s) => s,
    };
    let next_operand_stack = push_operand_stack(&input_operand_stack, &result_type);
    if operand_stack_has_legal_length(environment, &next_operand_stack) {
        Result::Ok(Frame { locals: input_frame.locals.iter().map(|x| copy_recurse(x)).collect(), stack_map: next_operand_stack, flag_this_uninit: input_frame.flag_this_uninit })
    } else {
        Result::Err(TypeSafetyResult::NotSafe("Operand stack did not have legal length".to_string()))
    }
}


pub fn pop_matching_list(pop_from: &Vec<UnifiedType>, pop: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
    return pop_matching_list_impl(pop_from.as_slice(), pop.as_slice());
}

pub fn pop_matching_list_impl(pop_from: &[UnifiedType], pop: &[UnifiedType]) -> Option<Vec<UnifiedType>> {
    if pop.is_empty() {
        Some(pop_from.iter().map(|x| copy_recurse(x)).collect())//todo inefficent copying
    } else {
        let (pop_result, _) = match pop_matching_type(pop_from, pop.first().unwrap()) {
            None => return None,
            Some(s) => s,
        };
        return pop_matching_list_impl(&pop_result, &pop[1..]);
    }
}

pub fn pop_matching_type<'l>(operand_stack: &'l [UnifiedType], type_: &UnifiedType) -> Option<(&'l [UnifiedType], UnifiedType)> {
    if size_of(type_) == 1 {
        let actual_type = &operand_stack[0];
        if is_assignable(actual_type, type_) {
            return Some((&operand_stack[1..], copy_recurse(actual_type)));
        } else {
            return None;
        }
    } else if size_of(type_) == 2 {
        &operand_stack[0];//should be top todo
        let actual_type = &operand_stack[1];
        if is_assignable(actual_type, type_) {
            return Some((&operand_stack[2..], copy_recurse(actual_type)));
        } else {
            unimplemented!()
        }
    } else {
        panic!()
    }
}


pub fn size_of(unified_type: &UnifiedType) -> u64 {
    match unified_type {
        UnifiedType::TopType => { 1 }
        _ => {
            if is_assignable(unified_type, &UnifiedType::TwoWord) {
                2
            } else if is_assignable(unified_type, &UnifiedType::OneWord) {
                1
            } else {
                panic!("This is a bug")
            }
        }
    }
}

pub fn push_operand_stack(operand_stack: &Vec<UnifiedType>, type_: &UnifiedType) -> Vec<UnifiedType> {
    let mut operand_stack_copy = operand_stack.iter().map(|x| copy_recurse(x)).collect();
    match type_ {
        UnifiedType::VoidType => {
            operand_stack_copy
        }
        _ => {
            if size_of(type_) == 2 {
                operand_stack_copy.push(UnifiedType::TopType);
                operand_stack_copy.push(copy_recurse(type_));
            } else if size_of(type_) == 1 {
                operand_stack_copy.push(copy_recurse(type_));
            } else {
                unimplemented!()
            }
            operand_stack_copy
        }
    }
}


pub fn operand_stack_has_legal_length(environment: &Environment, operand_stack: &Vec<UnifiedType>) -> bool {
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

pub fn can_pop(input_frame: &Frame, types: Vec<UnifiedType>) -> Option<Frame> {
    let poped_stack = match pop_matching_list(&input_frame.stack_map, types){
        None => return None,
        Some(s) => s,
    };
    Some(Frame {
        locals: input_frame
            .locals
            .iter()
            .map(|x|copy_recurse(x))
            .collect(),
        stack_map: poped_stack,
        flag_this_uninit: input_frame.flag_this_uninit
    })
}

pub fn frame_is_assignable(left: &Frame, right: &Frame) -> TypeSafetyResult {
    if left.stack_map.len() == right.stack_map.len()
        && left.locals.iter().zip(right.locals.iter()).all(|(left_, right_)| {
        is_assignable(left_, right_)
    }) && left.stack_map.iter().zip(right.stack_map.iter()).all(|(left_, right_)| {
        is_assignable(left_, right_)
    }) && if left.flag_this_uninit {
        right.flag_this_uninit
    } else {
        true
    } {
        TypeSafetyResult::Safe()
    } else {
        TypeSafetyResult::NotSafe("todo message".to_string())//todo message
    }
}

pub fn method_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> TypeSafetyResult {
    let access_flags = get_access_flags(class, method);
    trace!("got access_flags:{}", access_flags);
    let does_not_override_final_method = does_not_override_final_method(class, method);
    trace!("does not override final method:");
    dbg!(&does_not_override_final_method);
    let results = vec![does_not_override_final_method,
                       if access_flags & ACC_NATIVE != 0 {
                           trace!("method is native");
                           TypeSafetyResult::Safe()
                       } else if access_flags & ACC_ABSTRACT != 0 {
                           trace!("method is abstract");
                           TypeSafetyResult::Safe()
                       } else {
                           //will have a code attribute. or else method_with_code_is_type_safe will crash todo
                           /*let attributes = get_attributes(class, method);
                           attributes.iter().any(|_| {
                               unimplemented!()
                           }) && */method_with_code_is_type_safe(class, method)
                       }].into_boxed_slice();
    merge_type_safety_results(results)
}

pub struct ParsedCodeAttribute<'l> {
    //    pub class_name: NameReference,
//    pub frame_size: u16,
//    pub max_stack: u16,
//    pub code: &'l Vec<Instruction>,
//    pub exception_table: Vec<Handler>,
//    todo
//    pub stackmap_frames: Vec<&'l StackMap<'l>>,//todo
    pub method: &'l PrologClassMethod<'l>
}

pub fn get_handlers(class: &PrologClass, code: &Code) -> Vec<Handler> {
    code.exception_table.iter().map(|f| {
        Handler {
            start: f.start_pc as usize,
            end: f.end_pc as usize,
            target: f.handler_pc as usize,
            class_name: if f.catch_type == 0 { None } else {
                Some(NameReference {//todo NameReference v ClassReference
                    index: f.catch_type,
                    class_file: Arc::downgrade(&class.class),
                })
            },
        }
    }).collect()
}

pub fn method_with_code_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> TypeSafetyResult {
    let method_info = &class.class.methods[method.method_index];
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
    let handlers = get_handlers(class, code);
    let stack_map: Vec<StackMap> = get_stack_map_frames(class, method_info);
    trace!("stack map frames generated:");
    dbg!(&stack_map);
    let merged = merge_stack_map_and_code(instructs, stack_map.iter().map(|x| { x }).collect());
    trace!("stack map frames merged:");
    dbg!(&merged);
    let (frame, return_type) = method_initial_stack_frame(class, method, frame_size);
    trace!("Initial stack frame:");
    dbg!(&frame);
    dbg!(&frame_size);
    dbg!(&return_type);
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.clone(), handlers, return_type };

    and(handers_are_legal(&env),
        merged_code_is_type_safe(&env, merged.as_slice(), FrameResult::Regular(&frame)))
}

pub struct Handler {
    pub start: usize,
    pub end: usize,
    pub target: usize,
    pub class_name: Option<NameReference>,
    //todo
}

pub fn handler_exception_class(handler: &Handler) -> PrologClass {
    //may want to return a unifiedType instead
    match &handler.class_name {
        None => { unimplemented!("Return java/lang/Throwable") }
        Some(s) => { unimplemented!("Need to get class from state") }
    }
}
//

pub fn init_handler_is_legal(env: &Environment, handler: &Handler) -> TypeSafetyResult {
    unimplemented!()
}
//
//#[allow(unused)]
//pub fn not_init_handler(env: &Environment, handler: &Handler) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn is_init_handler(env: &Environment, handler: &Handler) -> bool {
//    unimplemented!()
//}

pub enum UnifiedInstruction {}

//#[allow(unused)]
//pub fn is_applicable_instruction(handler_start: u64, instruction: &UnifiedInstruction) -> bool {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn no_attempt_to_return_normally(instruction: &UnifiedInstruction) -> bool {
//    unimplemented!()
//}

#[allow(dead_code)]
pub struct Environment<'l> {
    pub method: &'l PrologClassMethod<'l>,
    pub return_type: UnifiedType,
    pub frame_size: u16,
    pub max_stack: u16,
    pub merged_code: Option<&'l Vec<MergedCodeInstruction<'l>>>,
    pub class_loader: Arc<Loader>,
    pub handlers: Vec<Handler>,
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

fn method_initial_stack_frame(class: &PrologClass, method: &PrologClassMethod, frame_size: u16) -> (Frame, UnifiedType) {
    //methodInitialStackFrame(Class, Method, FrameSize, frame(Locals, [], Flags),ReturnType):-
    //    methodDescriptor(Method, Descriptor),
    //    parseMethodDescriptor(Descriptor, RawArgs, ReturnType),
    //    expandTypeList(RawArgs, Args),
    //    methodInitialThisType(Class, Method, ThisList),
    //    flags(ThisList, Flags),
    //    append(ThisList, Args, ThisArgs),
    //    expandToLength(ThisArgs, FrameSize, top, Locals).
    let method_descriptor = extract_string_from_utf8(&class.class.constant_pool[method.prolog_class.class.methods[method.method_index as usize].descriptor_index as usize]);
    let parsed_descriptor = parse_method_descriptor(method_descriptor.as_str()).unwrap();
    let this_list = method_initial_this_type(class, method);
    let flag_this_uninit = flags(&this_list);
    let mut args = expand_type_list(parsed_descriptor.parameter_types);
    let mut this_args = vec![];
    this_list.iter().for_each(|x| {
        this_args.push(copy_recurse(x));
    });
    args.iter().for_each(|x| {
        this_args.push(copy_recurse(x))
    });
    let locals = expand_to_length(this_args, frame_size as usize, UnifiedType::TopType);
    return (Frame { locals, flag_this_uninit, stack_map: vec![] }, parsed_descriptor.return_type);
}


fn expand_type_list(list: Vec<UnifiedType>) -> Vec<UnifiedType> {
    return list.iter().flat_map(|x| {
        if size_of(x) == 1 {
            vec![copy_recurse(x)]
        } else {
            assert!(size_of(x) == 2);
            vec![copy_recurse(x), UnifiedType::TopType]
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
    assert!(list.len() >= size);
    let mut res = vec![];
    for i in 0..size {
        res.push(match list.get(i) {
            None => { copy_recurse(&filler) }
            Some(elem) => { copy_recurse(&elem) }
        })
    }
    return res;
}


fn method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> Option<UnifiedType> {
    let method_access_flags = method.prolog_class.class.methods[method.method_index].access_flags;
    if method_access_flags & ACC_STATIC > 0 {
        //todo dup
        let classfile = &method.prolog_class.class;
        let method_name_info = &classfile.constant_pool[classfile.methods[method.method_index].name_index as usize];
        let method_name = extract_string_from_utf8(method_name_info);
        if method_name != "<init>" {
            return None;
        } else {
            unimplemented!()
        }
    } else {
        Some(instance_method_initial_this_type(class, method))
    }
//    return Some(UnifiedType::UninitializedThis);
}

#[allow(unused)]
fn instance_method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> UnifiedType {
    let method_name = method_name(&method.prolog_class.class, &method.prolog_class.class.methods[method.method_index]);
    if method_name == "<init>" {
        if get_referred_name(&class_name(&class.class)) == "java/lang/Object" {
            UnifiedType::Class(class_name(&class.class))
        } else {
            let mut chain = vec![];
            super_class_chain(class, class.loader.clone(), &mut chain);
            if !chain.is_empty() {
                UnifiedType::UninitializedThis
            } else {
                unimplemented!()
            }
        }
    } else {
        //todo what to do about loaders
        UnifiedType::Class(class_name(&class.class))
    }
}
