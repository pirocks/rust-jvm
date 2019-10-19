use std::intrinsics::uninit;
use std::prelude::v1::Vec;

use classfile::{ACC_NATIVE, Classfile};
use classfile::ACC_ABSTRACT;
use classfile::attribute_infos::VerificationTypeInfo;
use classfile::code::Instruction;
use verification::code_verification::ParseCodeAttribute;
use verification::code_verification::StackMap;
use verification::prolog_info_defs::{class_name, get_access_flags, get_super_class_name};

pub fn loaded_class(class: &PrologClass) -> bool {
    unimplemented!()
}

pub fn loaded_class_(class_name: String, loader_name: String) -> Option<PrologClass> {
    unimplemented!()
}


struct ClassLoaderState<'l> {
    //todo
}

struct PrologClass<'l> {
    loader: String,
    class: &'l Classfile,
}

struct PrologClassMethod<'l> {
    prolog_class: PrologClass<'l>,
    method_index: usize,
}

//todo how to handle arrays
pub fn is_java_assignable(from: &PrologClass, to: &PrologClass) -> bool {
    if loaded_class(to) {
        return class_is_interface(to)
    }
    return is_java_sub_class_of(from, to);
    unimplemented!();
}

pub fn is_array_interface(class: PrologClass) -> bool {
    class_name(class.class) == "java/lang/Cloneable" ||
        class_name(class.class) == "java/io/Serializable"
}

pub fn is_java_subclass_of(sub: &PrologClass, super_: &PrologClass) {
    unimplemented!()
}

pub fn super_class_chain(chain_start: &PrologClass) -> Vec<&PrologClass> {
    unimplemented!()
}

struct Frame {
    locals: Vec<VerificationTypeInfo>,
    stack_map: Vec<VerificationTypeInfo>,
    flag_this_uninit: bool,
}

/**
Because of the confusing many types of types, this is a type enum to rule them all.
*/
pub enum UnifiedType {}

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

pub fn valid_type_transition(environment: Environment, expected_types_on_stack: Vec<UnifiedType>, result_type: UnifiedType, input_frame: Frame, next_frame: Frame) -> bool {
    unimplemented!()
}

pub fn pop_matching_list(pop_from: Vec<UnifiedType>, pop: Vec<UnifiedType>) -> Vec<UnifiedType> {
    unimplemented!()
}

pub fn pop_matching_type(operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Option<(Vec<UnifiedType>, UnifiedType)> {
    unimplemented!()
}

pub fn size_of(unified_type: UnifiedType) -> u64 {
    unimplemented!()
}

pub fn push_operand_stack(operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Vec<UnifiedType> {
    unimplemented!()
}

pub fn operand_stack_has_legal_length(environment: Environment) -> bool {
    unimplemented!()
}

pub fn pop_category_1(types: Vec<UnifiedType>) -> Option<(UnifiedType, Vec<UnifiedType>)> {
    unimplemented!()
}

pub fn can_safely_push(environment: Environment, input_operand_stack: Vec<UnifiedType>, type_: UnifiedType) -> Option<Vec<UnifiedType>> {
    unimplemented!();
}

pub fn can_safely_push_list(environment: Environment, input_operand_stack: Vec<UnifiedType>, type_list: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
    unimplemented!()
}

pub fn can_push_list(input_operand_stack: Vec<UnifiedType>, type_list: Vec<UnifiedType>) -> Option<Vec<UnifiedType>> {
    unimplemented!()
}

pub fn can_pop(input_frame: Frame, types: Vec<UnifiedType>) -> Option<Frame> {
    unimplemented!()
}

//pub fn nth1OperandStackIs



pub fn is_bootstrap_loader(loader: &String) -> bool {
    return loader == &"bl".to_string();//todo  what if someone defines a Loader class called bl
}

pub fn get_class_methods(class: &PrologClass) -> Vec<PrologClassMethod> {
    let mut res = vec![];
    for method_index in 0..class.methods.len() {
        res.push(PrologClassMethod { prolog_class: class, method_index })
    }
    res
}

pub fn class_is_type_safe(class: &PrologClass) -> bool {
    if class.name == "java/lang/Object" {
        if !is_bootstrap_loader(&class.loader) {
            return false;
        }
    } else {
        //class must have a superclass or be 'java/lang/Object'
        let chain = super_class_chain(class);
        if chain.is_empty() {
            return false;
        }
        let super_class_name = get_super_class_name(class.class);
        let super_class = loaded_class_(super_class_name, "bl".to_string())?;//todo magic string//todo double check this returns false
        if class_is_final(super_class) {
            return false;
        }

        unimplemented!();
    }
    let mut method = get_class_methods(class);
    method.iter().all(|m| {
        method_is_type_safe(class, m)
    })
}

pub fn does_not_override_final_method(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}

pub fn final_method_not_overridden(method: &PrologClassMethod, super_class: &PrologClass, method_list: &Vec<PrologClassMethod>) -> bool {
    unimplemented!()
}


pub fn does_not_override_final_method_of_superclass(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}


pub fn method_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> bool {
    let access_flags = get_access_flags(class, method);
    return does_not_override_final_method(class, method) &&
        if access_flags & ACC_NATIVE {
            true
        } else if access_flags & ACC_ABSTRACT {
            true
        } else {
            let attributes = get_attributes(class, method);
            attributes.iter().any(|_| {
                unimplemented!()
            }) && method_with_code_is_type_safe(class, method)
        };
}

pub fn method_with_code_is_type_safe(class: &PrologClass, method: &PrologClassMethod) -> bool {
    let parsed_code: ParseCodeAttribute = get_parsed_code_attribute(class, method);
    let frame_size = parsed_code.frame_size;
    let max_stack = parsed_code.max_stack;
    let code = parsed_code.code;
    let handlers = parsed_code.exception_table;
    let stack_map = parsed_code.stackmap_frames;
    let merged = merge_stack_map_and_code(code, stack_map);
    let (frame, frame_size, return_type) = method_initial_stack_frame(class, method);
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code };
    handers_are_legal(&env) && merged_code_is_type_safe(&env, merged)
}

pub fn handers_are_legal(env: &Environment) -> bool {
    unimplemented!()
}

pub fn instructions_include_end(instructs: Vec<UnifiedInstruction>, end: u64) -> bool {
    unimplemented!()
}

pub struct Handler {
    pub start: usize,
    pub end: usize,
    //todo
}

pub fn handler_exception_class(handler: &Handler) {
    unimplemented!()
}

pub fn init_handler_is_legal(env: &Environment, handler: &Handler) -> bool {
    unimplemented!()
}

pub fn not_init_handler(env: &Environment, handler: &Handler) -> bool {
    unimplemented!()
}

pub fn is_init_handler(env: &Environment, handler: &Handler) -> bool {
    unimplemented!()
}

pub enum UnifiedInstruction {}

pub fn is_applicable_instruction(handler_start: u64, instruction: &UnifiedInstruction) -> bool {
    unimplemented!()
}

pub fn no_attempt_to_return_normally(instruction: &UnifiedInstruction) -> bool {
    unimplemented!()
}


struct Environment<'l> {
    method: &PrologClassMethod<'l>,
    frame_size: u16,
    max_stack: u16,
    merged_code: Option<Vec<MergedCodeInstruction<'l>>>,

}

enum MergedCodeInstruction<'l> {
    Instruction(&'l Instruction),
    StackMap(&'l StackMap),
}

/**
assumes that stackmaps and instructions are ordered
*/
fn merge_stack_map_and_code<'l>(instruction: Vec<Instruction>, stack_maps: Vec<StackMap>) -> Vec<MergedCodeInstruction<'l>> {
    let mut res = vec![];

    loop {
        let (instruction, instruction_offset) = match instruction.first() {
            None => { (None, -1) },//todo hacky
            Some(i) => { (Some(i), i.offset as i32) },
        };
        let (stack_map, stack_map_offset) = match stack_maps.first() {
            None => { (None, -1) },
            Some(s) => { (Some(s), s.offset as i32) },
        };
        if stack_map_offset >= instruction_offset {
            res.push(MergedCodeInstruction::StackMap(stack_map.unwrap()))//todo
        } else {
            let instr = match instruction {
                None => { break },
                Some(i) => { i },
            };
            res.push(MergedCodeInstruction::Instruction(instr))//todo
        }
    }
    return res;
}

fn method_initial_stack_frame(class: &PrologClass, method: &PrologClassMethod) -> (Frame, u64, UnifiedType) {
    unimplemented!()
}

fn expand_type_list(list: Vec<UnifiedType>) -> Vec<UnifiedType> {
    unimplemented!()
}

//fn flags()

fn expand_to_length(list: Vec<UnifiedType>, size: usize, filler: UnifiedType) -> Vec<UnifiedType> {
    unimplemented!()
}

fn method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> Option<UnifiedType> {
    unimplemented!()
}

fn instance_method_initial_this_type(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}

fn merged_code_is_type_safe(env: &Environment, merged_code: Vec<MergedCodeInstruction>) -> bool {
    unimplemented!()
}

fn target_is_type_safe(env: &Environment, stack_frame: Frame, target: u64) {
    unimplemented!()
}

fn instruction_satisfies_handlers(env: &Environment, offset: u64) -> bool {
    unimplemented!()
}

fn is_applicable_handler(offset: usize, handler: Handler) -> bool {
    unimplemented!()
}

fn load_is_type_safe(env: &Environment, index: usize, type_: &UnifiedType, frame: &Frame, next_frame: &Frame) -> bool {
    unimplemented!()
}

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

fn passes_protected_check(env: &Environment, member_class_name: String, member_name: String, member_descriptor: &Descriptor, stack_frame: &Frame) -> bool {
    unimplemented!()
}

//fn classesInOtherPkgWithProtectedMember(, ) //todo

fn same_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

fn different_runtime_package(class1: PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

fn exception_stack_frame(frame1: Frame, excpetion_stack_frame: Frame) -> bool {
    unimplemented!()
}


fn instruction_is_type_safe_aaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_aastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_aconst_null(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_aload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_anewarray(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_areturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_arraylength(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_astore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_athrow(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_baload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_bastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_caload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_castore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_checkcast(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_d2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_d2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_d2l(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_daload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dcmpg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dup(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dup_x1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dup_x2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dup2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dup2_x1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_dup2_x2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_f2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_f2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_f2l(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_faload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fcmpg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_freturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_fstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_getfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_getstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_goto(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_i2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_i2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_iadd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_iaload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_iastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_iconst_m1(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_if_acmpeq(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_if_icmpeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ifeq(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ifnonnull(target: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_iinc(index: usize, value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_iload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ineg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_instanceof(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_invokedynamic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_invokeinterface(cp: usize, count: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_invokespecial(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_invokestatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_invokevirtual(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ireturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_istore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_l2d(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_l2f(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_l2i(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ladd(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_laload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lcmp(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lconst_0(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ldc(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_ldc2_w(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lload(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lneg(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lookupswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lreturn(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lshl(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_lstore(index: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_monitorenter(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_multianewarray(cp: usize, dim: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

//todo start using CPIndex instead of usize

fn instruction_is_type_safe_new(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_newarray(type_code:usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_nop(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_pop(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_pop2(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_putfield(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_putstatic(cp: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_return(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_saload(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_sastore(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_sipush(value:usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_swap(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, keys: Vec<usize> ,env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn different_package_name(class1: &PrologClass, class2:&PrologClass ) -> bool{
    unimplemented!()
}

fn same_package_name(class1: &PrologClass, class2:&PrologClass ) -> bool{
    unimplemented!()
}