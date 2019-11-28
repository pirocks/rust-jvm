use std::rc::Rc;

use classfile::{ACC_NATIVE, Classfile, ACC_PRIVATE, ACC_STATIC, stack_map_table_attribute};
use classfile::ACC_ABSTRACT;
use classfile::ACC_FINAL;
use classfile::ACC_INTERFACE;
use classfile::code::Instruction;
use classfile::code::InstructionInfo;
use classfile::code_attribute;
use verification::code_writer::ParseCodeAttribute;
use verification::code_writer::StackMap;
use verification::prolog_info_writer::{class_name_legacy, get_access_flags, get_super_class_name};
use verification::unified_type::ClassNameReference;
use verification::unified_type::NameReference;
use verification::unified_type::UnifiedType;
use verification::verifier::TypeSafetyResult::{NeedToLoad, NotSafe, Safe};
use classfile::attribute_infos::StackMapTable;
use class_loading::Loader;
use class_loading::class_entry;

pub struct InternalFrame {
    pub locals: Vec<UnifiedType>,
    pub stack: Vec<UnifiedType>,
    pub max_locals: u16,
    pub current_offset: u16,
}

//todo have an actual loader type. instead of refering to loader name
pub fn loaded_class(class: &PrologClass, loader: Loader) -> TypeSafetyResult {
    let class_entry = class_entry(&class.class);
    if loader.loading.borrow().contains_key(&class_entry) || loader.loaded.borrow().contains_key(&class_entry) {
        return Safe();
    } else {
        return NeedToLoad(vec![unimplemented!()]);
    }
}

pub fn loaded_class_(class_name: String, loader_name: String) -> Option<PrologClass> {
    unimplemented!()
}


#[allow(dead_code)]
struct ClassLoaderState {
    //todo
}

pub struct PrologClass {
    pub loader: String,
    pub class: Rc<Classfile>,
}

pub struct PrologClassMethod<'l> {
    pub prolog_class: &'l PrologClass,
    pub method_index: usize,
}

pub fn class_is_interface(class: &PrologClass) -> bool {
    return class.class.access_flags & ACC_INTERFACE != 0;
}

#[allow(unused)]
pub fn is_java_sub_class_of(from: &PrologClass, to: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn is_assignable(from: &UnifiedType, to: &UnifiedType) -> bool {
    unimplemented!()
}

//todo how to handle arrays
pub fn is_java_assignable(from: &PrologClass, to: &PrologClass) -> bool {
    match loaded_class(to, unimplemented!()) {
        TypeSafetyResult::Safe() => { return class_is_interface(to); }
    }
    unimplemented!();
    return is_java_sub_class_of(from, to);
}

pub fn is_array_interface(class: PrologClass) -> bool {
    class_name_legacy(&class.class) == "java/lang/Cloneable" ||
        class_name_legacy(&class.class) == "java/io/Serializable"
}

pub fn is_java_subclass_of(sub: &PrologClass, super_: &PrologClass) {
    unimplemented!()
}

pub fn class_super_class_name(class: &PrologClass) -> String {
    unimplemented!()
}

pub fn super_class_chain(chain_start: &PrologClass, loader: String) -> Vec<PrologClass> {
    let loaded = loaded_class(chain_start, unimplemented!());
    unimplemented!()
}

#[derive(Eq, PartialEq)]
pub struct Frame<'l> {
    pub locals: &'l Vec<UnifiedType>,
    pub stack_map: Vec<UnifiedType>,
    pub flag_this_uninit: bool,
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

//pub fn nth1OperandStackIs


pub fn is_bootstrap_loader(loader: &String) -> bool {
    return loader == &"bl".to_string();//todo  what if someone defines a Loader class called bl
}

pub fn get_class_methods(class: &PrologClass) -> Vec<PrologClassMethod> {
    let mut res = vec![];
    for method_index in 0..class.class.methods.borrow_mut().len() {
        res.push(PrologClassMethod { prolog_class: class, method_index })
    }
    res
}

pub fn class_is_final(class: &PrologClass) -> bool {
    class.class.access_flags & ACC_FINAL != 0
}

#[derive(Debug)]
pub enum TypeSafetyResult {
    NotSafe(String),
    //reason is a String
    Safe(),
    NeedToLoad(Vec<ClassNameReference>),
}

pub fn class_is_type_safe(class: &PrologClass) -> TypeSafetyResult {
    if class_name_legacy(&class.class) == "java/lang/Object" {
        if !is_bootstrap_loader(&class.loader) {
            return TypeSafetyResult::NotSafe("Loading object with something other than bootstrap loader".to_string());
        }
    } else {
        //class must have a superclass or be 'java/lang/Object'
        let chain = super_class_chain(class, unimplemented!());
        if chain.is_empty() {
            return TypeSafetyResult::NotSafe("No superclass but object is not Object".to_string());
        }
        let super_class_name = get_super_class_name(&class.class);
        let super_class = loaded_class_(super_class_name, "bl".to_string()).unwrap();//todo magic string
        if class_is_final(&super_class) {
            return TypeSafetyResult::NotSafe("Superclass is final".to_string());
        }
    }
    let method = get_class_methods(class);
    let method_type_safety: Vec<TypeSafetyResult> = method.iter().map(|m| {
        method_is_type_safe(class, m)
    }).collect();
    merge_type_safety_results(method_type_safety.into_boxed_slice())
}

pub(crate) fn merge_type_safety_results(method_type_safety: Box<[TypeSafetyResult]>) -> TypeSafetyResult {
    method_type_safety.iter().fold(TypeSafetyResult::Safe(), |a: TypeSafetyResult, b: &TypeSafetyResult| {
        match a {
            TypeSafetyResult::NotSafe(r) => { TypeSafetyResult::NotSafe(r) }
            TypeSafetyResult::Safe() => {
                match b {
                    TypeSafetyResult::NotSafe(r) => { TypeSafetyResult::NotSafe(r.clone()) }
                    TypeSafetyResult::Safe() => { TypeSafetyResult::Safe() }
                    TypeSafetyResult::NeedToLoad(to_load) => { TypeSafetyResult::NeedToLoad(to_load.clone()) }
                }
            }
            TypeSafetyResult::NeedToLoad(to_load) => {
                match b {
                    TypeSafetyResult::NotSafe(r) => { TypeSafetyResult::NotSafe(r.clone()) }
                    TypeSafetyResult::Safe() => { NeedToLoad(to_load) }
                    TypeSafetyResult::NeedToLoad(to_load_) => {
                        let mut new_to_load = vec![];
                        for c in to_load.iter() {
                            new_to_load.push(c.clone());
                        }
                        for c in to_load_.iter() {
                            new_to_load.push(c.clone());
                        }
                        NeedToLoad(new_to_load)
                    }
                }
            }
        }
    })
}

pub fn is_static(method: &PrologClassMethod, class: &PrologClass) -> bool {
    //todo check if same
    (get_access_flags(class, method) & ACC_STATIC) > 0
}

pub fn is_private(method: &PrologClassMethod, class: &PrologClass) -> bool {
    //todo check if method class and class same
    (get_access_flags(class, method) & ACC_PRIVATE) > 0
}

pub fn does_not_override_final_method(class: &PrologClass, method: &PrologClassMethod) -> TypeSafetyResult {
    dbg!(class_name_legacy(&class.class));
    if class_name_legacy(&class.class) == "java/lang/Object" {
        if is_bootstrap_loader(&class.loader) {
            Safe()
        } else {
            NotSafe("Loading Object w/o bootstrap loader".to_string())
        }
    } else if is_private(method, class) {
        Safe()
    } else if is_static(method, class) {
        Safe()
    } else if does_not_override_final_method_of_superclass(class, method) {
        Safe()
    } else {
        NotSafe("Failed does_not_override_final_method".to_string())
    }
}

#[allow(unused)]
pub fn final_method_not_overridden(method: &PrologClassMethod, super_class: &PrologClass, method_list: &Vec<PrologClassMethod>) -> bool {
    unimplemented!()
}

#[allow(unused)]
pub fn does_not_override_final_method_of_superclass(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
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
fn nth0(index: usize, locals: &Vec<UnifiedType>) -> UnifiedType {
    unimplemented!()
}

#[allow(unused)]
fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame, next_frame: &Frame) -> bool {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let type_transition = valid_type_transition(env, vec![], &actual_type, frame);

    is_assignable(&actual_type, type_) &&
        &type_transition == next_frame
}

#[allow(unused)]
fn store_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame, next_frame: &Frame) {
    unimplemented!()
}

pub struct FieldDescriptor {
    //todo
}

pub struct MethodDescriptor {
    //todo
}

pub enum Descriptor {}

//fn modify_local_variable() //todo

#[allow(unused)]
fn passes_protected_check(env: &Environment, member_class_name: String, member_name: String, member_descriptor: &Descriptor, stack_frame: &Frame) -> bool {
    unimplemented!()
}

//fn classesInOtherPkgWithProtectedMember(, ) //todo

#[allow(unused)]
fn same_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn different_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn exception_stack_frame(frame1: Frame, excpetion_stack_frame: Frame) -> bool {
    unimplemented!()
}

pub struct InstructionIsTypeSafeResult<'l> {
    next_frame: Frame<'l>,
    exception_frame: Frame<'l>,
}

#[allow(unused)]
fn instruction_is_type_safe<'l>(instruction: &InstructionInfo, env: &Environment, offset: usize, stack_frame: &Frame<'l>) -> Option<InstructionIsTypeSafeResult<'l>> {
    unimplemented!()
}


#[allow(unused)]
fn instruction_is_type_safe_aaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_aastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_aconst_null(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_aload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_anewarray(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_areturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_arraylength(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_astore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_athrow(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_baload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_bastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_caload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_castore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_checkcast(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_d2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_d2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_d2l(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_daload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dcmpg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dup(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dup_x1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dup_x2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dup2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dup2_x1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_dup2_x2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_f2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_f2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_f2l(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_faload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fcmpg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_freturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_fstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_getfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_getstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_goto(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_i2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_i2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_iadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_iaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_iastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_iconst_m1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_if_acmpeq(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_if_icmpeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ifeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ifnonnull(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_iinc(index: usize, value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_iload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ineg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_instanceof(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_invokedynamic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_invokeinterface(cp: usize, count: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_invokespecial(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ireturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_istore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_l2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_l2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_l2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ladd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_laload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lcmp(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ldc(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_ldc2_w(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lookupswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lshl(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_lstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_monitorenter(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_multianewarray(cp: usize, dim: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

//todo start using CPIndex instead of usize

#[allow(unused)]
fn instruction_is_type_safe_new(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_newarray(type_code: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_nop(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_pop(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_pop2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_putfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_putstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_return(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_saload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_sastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_sipush(value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_swap(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn different_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

#[allow(unused)]
fn same_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}