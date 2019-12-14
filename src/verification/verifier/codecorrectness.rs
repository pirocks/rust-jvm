use verification::verifier::{Frame, PrologClass, PrologClassMethod, TypeSafetyResult, merge_type_safety_results};
use verification::unified_type::{UnifiedType, NameReference, ClassNameReference};
use std::rc::Rc;
use verification::code_writer::{ParseCodeAttribute, StackMap};
use classfile::code::Instruction;
use verification::verifier::filecorrectness::{is_assignable, does_not_override_final_method};
use verification::prolog_info_writer::get_access_flags;
use classfile::{ACC_NATIVE, ACC_ABSTRACT, stack_map_table_attribute, code_attribute};
use classfile::attribute_infos::StackMapTable;
use verification::verifier::TypeSafetyResult::Safe;
use verification::verifier::instructions::instruction_is_type_safe;

#[allow(unused)]
fn exception_stack_frame(frame1: Frame, excpetion_stack_frame: Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn valid_type_transition<'l>(environment: &Environment, expected_types_on_stack: Vec<UnifiedType>, result_type: &UnifiedType, input_frame: &Frame<'l>) -> Frame<'l> {
    unimplemented!()
}

#[allow(unused)]
pub fn pop_matching_list(pop_from: Vec<UnifiedType>, pop: Vec<UnifiedType>) -> Vec<UnifiedType> {
    unimplemented!()
}

#[allow(unused)]
pub fn pop_matching_type(operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Option<(Vec<UnifiedType>, UnifiedType)> {
    unimplemented!()
}

#[allow(unused)]
pub fn size_of(unified_type: UnifiedType) -> u64 {
    unimplemented!()
}

#[allow(unused)]
pub fn push_operand_stack(operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Vec<UnifiedType> {
    unimplemented!()
}

#[allow(unused)]
pub fn operand_stack_has_legal_length(environment: &Environment, operand_stack: &Vec<UnifiedType>) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn pop_category_1(types: Vec<UnifiedType>) -> Option<(UnifiedType, Vec<UnifiedType>)> {
    unimplemented!()
}

#[allow(unused)]
pub fn can_safely_push(environment: Environment, input_operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Option<Vec<UnifiedType>> {
    unimplemented!();
}

#[allow(unused)]
pub fn can_safely_push_list(environment: Environment, input_operand_stack: Vec<UnifiedType>, type_list: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
    unimplemented!()
}

#[allow(unused)]
pub fn can_push_list(input_operand_stack: Vec<UnifiedType>, type_list: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
    unimplemented!()
}

#[allow(unused)]
pub fn can_pop(input_frame: Frame, types: Vec<UnifiedType>) -> Option<Frame> {
    unimplemented!()
}



/**
Because of the confusing many types of types, this is a type enum to rule them all.
*/
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
    merge_type_safety_results(vec![does_not_override_final_method(class, method),
                                   if access_flags & ACC_NATIVE != 0 {
                                       TypeSafetyResult::Safe()
                                   } else if access_flags & ACC_ABSTRACT != 0 {
                                       TypeSafetyResult::Safe()
                                   } else {
                                       //will have a code attribute.
                                       /*let attributes = get_attributes(class, method);
                                       attributes.iter().any(|_| {
                                           unimplemented!()
                                       }) && */method_with_code_is_type_safe(class, method)
                                   }].into_boxed_slice())
}

pub fn get_parsed_code_attribute<'l>(class: &'l PrologClass, method: &'l PrologClassMethod) -> ParseCodeAttribute<'l> {
    //todo check method in class
    let method_info = &class.class.methods.borrow_mut()[method.method_index];
    let code = code_attribute(method_info).unwrap();
    let empty_stack_map = StackMapTable { entries: Vec::new() };
    let stack_map = stack_map_table_attribute(code).get_or_insert(&empty_stack_map);
    ParseCodeAttribute {
        class_name: NameReference {
            class_file: Rc::downgrade(&class.class),
            index: class.class.this_class,
        },
        frame_size: code.max_locals,
        max_stack: code.max_stack,
        code: &code.code,
        exception_table: code.exception_table.iter().map(|f| {
            Handler {
                start: f.start_pc as usize,
                end: f.end_pc as usize,
                target: f.handler_pc as usize,
                class_name: if f.catch_type == 0 { None } else {
                    Some(NameReference {//todo NameReference v ClassReference
                        index: f.catch_type,
                        class_file: Rc::downgrade(&class.class),
                    })
                },
            }
        }).collect(),
        stackmap_frames: unimplemented!(),
    }
}

pub fn method_with_code_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> TypeSafetyResult {
    let parsed_code: ParseCodeAttribute = get_parsed_code_attribute(class, &method);
    let frame_size = parsed_code.frame_size;
    let max_stack = parsed_code.max_stack;
    let code: Vec<&Instruction> = parsed_code.code.iter().map(|x| { x }).collect();
    let handlers = parsed_code.exception_table;
    let stack_map = parsed_code.stackmap_frames;
    let merged = merge_stack_map_and_code(code, stack_map);
    let (frame, frame_size, return_type) = method_initial_stack_frame(class, method);
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(&merged), class_loader: class.loader.as_str(), handlers };
    if handers_are_legal(&env) && merged_code_is_type_safe(&env, merged.as_slice(), &frame, false) {
        Safe()
    } else {
        unimplemented!()
    }
}

#[allow(unused)]
pub fn handers_are_legal(env: &Environment) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn instructions_include_end(instructs: Vec<UnifiedInstruction>, end: u64) -> bool {
    unimplemented!()
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

#[allow(unused)]
pub fn init_handler_is_legal(env: &Environment, handler: &Handler) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn not_init_handler(env: &Environment, handler: &Handler) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn is_init_handler(env: &Environment, handler: &Handler) -> bool {
    unimplemented!()
}

pub enum UnifiedInstruction {}

#[allow(unused)]
pub fn is_applicable_instruction(handler_start: u64, instruction: &UnifiedInstruction) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn no_attempt_to_return_normally(instruction: &UnifiedInstruction) -> bool {
    unimplemented!()
}


#[allow(dead_code)]
pub struct Environment<'l> {
    method: &'l PrologClassMethod<'l>,
    frame_size: u16,
    max_stack: u16,
    merged_code: Option<&'l Vec<MergedCodeInstruction<'l>>>,
    class_loader: &'l str,
    handlers: Vec<Handler>,
}

enum MergedCodeInstruction<'l> {
    Instruction(&'l Instruction),
    StackMap(&'l StackMap<'l>),
}

/**
assumes that stackmaps and instructions are ordered
*/
fn merge_stack_map_and_code<'l>(instruction: Vec<&'l Instruction>, stack_maps: Vec<&'l StackMap>) -> Vec<MergedCodeInstruction<'l>> {
    let mut res = vec![];

    loop {
        let (instruction, instruction_offset) = match instruction.first() {
            None => { (None, -1) }//todo hacky
            Some(i) => { (Some(i), i.offset as i32) }
        };
        let (stack_map, stack_map_offset) = match stack_maps.first() {
            None => { (None, -1) }
            Some(s) => { (Some(s), s.offset as i32) }
        };
        if stack_map_offset >= instruction_offset {
            res.push(MergedCodeInstruction::StackMap(stack_map.unwrap()))//todo
        } else {
            let instr = match instruction {
                None => { break; }
                Some(i) => { i }
            };
            res.push(MergedCodeInstruction::Instruction(instr))//todo
        }
    }
    return res;
}

#[allow(unused)]
fn method_initial_stack_frame<'l>(class: &'l PrologClass, method: &'l PrologClassMethod) -> (Frame<'l>, u64, UnifiedType) {
    unimplemented!()
}

#[allow(unused)]
fn expand_type_list(list: Vec<UnifiedType>) -> Vec<UnifiedType> {
    unimplemented!()
}

//fn flags()

#[allow(unused)]
fn expand_to_length(list: Vec<UnifiedType>, size: usize, filler: UnifiedType) -> Vec<UnifiedType> {
    unimplemented!()
}

#[allow(unused)]
fn method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> Option<UnifiedType> {
    unimplemented!()
}

#[allow(unused)]
fn instance_method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}

//todo how to handle other values here
fn merged_code_is_type_safe(env: &Environment, merged_code: &[MergedCodeInstruction], after_frame: &Frame, after_goto: bool) -> bool {
    let first = &merged_code[0];
    let rest = &merged_code[1..merged_code.len()];
    match first {
        MergedCodeInstruction::Instruction(i) => {
            let instruction_res = instruction_is_type_safe(&i.instruction, env, i.offset, after_frame).unwrap();//todo unwrap
            let exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, &instruction_res.exception_frame);
            merged_code_is_type_safe(env, rest, &instruction_res.next_frame, false)
        }
        MergedCodeInstruction::StackMap(s) => {
            if after_goto {
                merged_code_is_type_safe(env, rest, &s.map_frame, false)
            } else {
                frame_is_assignable(after_frame, &s.map_frame) &&
                    merged_code_is_type_safe(env, rest, &s.map_frame, false)
            }
        }
    }
}

#[allow(unused)]
fn offset_stack_frame<'l>(env: &Environment, target: usize) -> Frame<'l> {
    unimplemented!()
}

fn target_is_type_safe(env: &Environment, stack_frame: &Frame, target: usize) -> bool {
    let frame = offset_stack_frame(env, target);
    frame_is_assignable(stack_frame, &frame)
}

fn instruction_satisfies_handlers(env: &Environment, offset: usize, exception_stack_frame: &Frame) -> bool {
    let handlers = &env.handlers;
    let mut applicable_handler = handlers.iter().filter(|h| {
        is_applicable_handler(offset as usize, h)
    });
    applicable_handler.all(|h| {
        instruction_satisfies_handler(env, exception_stack_frame, h)
    })
}

fn is_applicable_handler(offset: usize, handler: &Handler) -> bool {
    offset <= handler.start && offset < handler.end
}

fn class_to_type(class: &PrologClass) -> UnifiedType {
    UnifiedType::ReferenceType(ClassNameReference::Ref(NameReference {
        index: class.class.this_class,
        class_file: Rc::downgrade(&class.class),
    }))
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> bool {
    let target = handler.target;
    let class_loader = &env.class_loader;
    let exception_class = handler_exception_class(handler);
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let true_exc_stack_frame = Frame { locals, stack_map: vec![class_to_type(&exception_class)], flag_this_uninit: flags };
    operand_stack_has_legal_length(env, &vec![class_to_type(&exception_class)]) &&
        target_is_type_safe(env, &true_exc_stack_frame, target)
}

#[allow(unused)]
pub(crate) fn nth0(index: usize, locals: &Vec<UnifiedType>) -> UnifiedType {
    unimplemented!()
}

