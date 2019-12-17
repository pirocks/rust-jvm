use log::trace;
use classfile::{ACC_ABSTRACT, ACC_NATIVE, code_attribute};
use classfile::attribute_infos::Code;
use classfile::code::Instruction;
use verification::classnames::NameReference;
use verification::code_writer::StackMap;
use verification::prolog_info_writer::get_access_flags;
use verification::unified_type::UnifiedType;
use verification::verifier::{Frame, merge_type_safety_results, PrologClass, PrologClassMethod, TypeSafetyResult};
use verification::verifier::filecorrectness::{does_not_override_final_method, is_assignable};
use verification::verifier::TypeSafetyResult::Safe;
use verification::verifier::codecorrectness::stackmapframes::get_stack_map_frames;
use class_loading::Loader;
use std::sync::Arc;
use verification::verifier::instructions::handers_are_legal;
use verification::verifier::instructions::merged_code_is_type_safe;
use verification::prolog_info_writer::extract_string_from_utf8;
use verification::types::parse_method_descriptor;
use verification::verifier::codecorrectness::stackmapframes::copy_recurse;
use classfile::ACC_STATIC;

pub mod stackmapframes;
//
//#[allow(unused)]
//fn exception_stack_frame(frame1: Frame, excpetion_stack_frame: Frame) -> bool {
//    unimplemented!()
//}
//

pub fn valid_type_transition(environment: &Environment, expected_types_on_stack: Vec<UnifiedType>, result_type: &UnifiedType, input_frame: &Frame) -> Frame {
    unimplemented!()
}
//
//#[allow(unused)]
//pub fn pop_matching_list(pop_from: Vec<UnifiedType>, pop: Vec<UnifiedType>) -> Vec<UnifiedType> {
//    unimplemented!()
//}
//
//#[allow(unused)]
//pub fn pop_matching_type(operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Option<(Vec<UnifiedType>, UnifiedType)> {
//    unimplemented!()
//}
//

pub fn size_of(unified_type: &UnifiedType) -> u64 {
    unimplemented!()
}
//
//#[allow(unused)]
//pub fn push_operand_stack(operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Vec<UnifiedType> {
//    unimplemented!()
//}
//

pub fn operand_stack_has_legal_length(environment: &Environment, operand_stack: &Vec<UnifiedType>) -> bool {
    unimplemented!()
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
//#[allow(unused)]
//pub fn can_pop(input_frame: Frame, types: Vec<UnifiedType>) -> Option<Frame> {
//    unimplemented!()
//}

pub fn frame_is_assignable(left: &Frame, right: &Frame) -> bool {
    left.stack_map.len() == right.stack_map.len()
        && left.locals.iter().zip(right.locals.iter()).all(|(left_, right_)| {
        is_assignable(left_, right_)
    }) && left.stack_map.iter().zip(right.stack_map.iter()).all(|(left_, right_)| {
        is_assignable(left_, right_)
    }) && if left.flag_this_uninit {
        right.flag_this_uninit
    } else {
        true
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
    let instructs: Vec<&Instruction> = code.code.iter().map(|x| { x }).collect();
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
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.clone(), handlers };
    if handers_are_legal(&env) && merged_code_is_type_safe(&env, merged.as_slice(), &frame, false) {
        Safe()
    } else {
        unimplemented!()
    }
}

pub struct Handler {
    pub start: usize,
    pub end: usize,
    pub target: usize,
    pub class_name: Option<NameReference>,
    //todo
}

pub fn handler_exception_class(handler: &Handler) -> PrologClass {
    match &handler.class_name {
        None => { unimplemented!("Return java/lang/Throwable") }
        Some(s) => { unimplemented!("Need to get class from state") }
    }
}
//
//#[allow(unused)]
//pub fn init_handler_is_legal(env: &Environment, handler: &Handler) -> bool {
//    unimplemented!()
//}
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

/**
assumes that stackmaps and instructions are ordered
*/
fn merge_stack_map_and_code<'l>(instruction: Vec<&'l Instruction>, stack_maps: Vec<&'l StackMap>) -> Vec<MergedCodeInstruction<'l>> {
    trace!("Starting instruction and stackmap merge");
    let mut res = vec![];
    let mut current_instruction_i = 0;
    let mut current_stackmap_i = 0;

    loop {
        let (instruction, instruction_offset) = match instruction.get(current_instruction_i) {
            None => { (None, -1) }//todo very hacky
            Some(i) => { (Some(i), i.offset as i32) }
        };
        let (stack_map, stack_map_offset) = match stack_maps.get(current_stackmap_i) {
            None => { (None, -2) }
            Some(s) => { (Some(s), s.offset as i32) }
        };
        if stack_map_offset >= instruction_offset {
            res.push(MergedCodeInstruction::StackMap(stack_map.unwrap()));
            current_stackmap_i += 1;
        } else {
            let instr = match instruction {
                None => { break; }
                Some(i) => { i }
            };
            res.push(MergedCodeInstruction::Instruction(instr));
            current_instruction_i += 1;
        }
    }
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
            vec![copy_recurse(x),UnifiedType::TopType]
        }
    }).collect();
}

fn flags(this_list: &Option<UnifiedType>) -> bool {
    match this_list {
        None => { return false; }
        Some(s) => {
//            assert!(s is unitializedThis )//todo
            return true;
        }
    }
}


fn expand_to_length(list: Vec<UnifiedType>, size: usize, filler: UnifiedType) -> Vec<UnifiedType> {
    unimplemented!()
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
        }
    }
    return Some(UnifiedType::UninitializedThis);
}

#[allow(unused)]
fn instance_method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}
