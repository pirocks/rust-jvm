use classfile::{ACC_NATIVE, Classfile};
use classfile::ACC_ABSTRACT;
use classfile::code::Instruction;
use verification::code_writer::ParseCodeAttribute;
use verification::code_writer::StackMap;
use verification::prolog_info_writer::{class_name, get_access_flags, get_super_class_name};
use classfile::ACC_INTERFACE;
use classfile::ACC_FINAL;
use classfile::code_attribute;
use classfile::code::InstructionInfo;
use verification::unified_type::UnifiedType;
use verification::unified_type::ClassNameReference;
use verification::unified_type::NameReference;

pub struct InternalFrame<'l> {
    pub locals: Vec<UnifiedType<'l>>,
    pub stack: Vec<UnifiedType<'l>>,
    pub max_locals: u16,
    pub current_offset: u16,
}

pub fn loaded_class(class: &PrologClass) -> bool {
    unimplemented!()
}

pub fn loaded_class_<'l>(class_name: String, loader_name: String) -> Option<PrologClass<'l>> {
    unimplemented!()
}


struct ClassLoaderState {
    //todo
}

pub struct PrologClass<'l> {
    pub loader: String,
    pub class: &'l Classfile<'l>,
}

pub struct PrologClassMethod<'l> {
    pub prolog_class: &'l PrologClass<'l>,
    pub method_index: usize,
}

pub fn class_is_interface(class: &PrologClass) -> bool {
    return class.class.access_flags & ACC_INTERFACE != 0;
}

pub fn is_java_sub_class_of(from: &PrologClass, to: &PrologClass) -> bool {
    unimplemented!()
}

pub fn is_assignable(from: &UnifiedType, to: &UnifiedType) -> bool {
    unimplemented!()
}

//todo how to handle arrays
pub fn is_java_assignable(from: &PrologClass, to: &PrologClass) -> bool {
    if loaded_class(to) {
        return class_is_interface(to);
    }
    unimplemented!();
    return is_java_sub_class_of(from, to);
}

pub fn is_array_interface(class: PrologClass) -> bool {
    class_name(class.class) == "java/lang/Cloneable" ||
        class_name(class.class) == "java/io/Serializable"
}

pub fn is_java_subclass_of(sub: &PrologClass, super_: &PrologClass) {
    unimplemented!()
}

pub fn super_class_chain<'l, 'k>(chain_start: &'k PrologClass) -> Vec<&'l PrologClass<'l>> {
    unimplemented!()
}

#[derive(Eq, PartialEq)]
pub struct Frame<'l> {
    pub locals: &'l Vec<UnifiedType<'l>>,
    pub stack_map: &'l Vec<UnifiedType<'l>>,
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

pub fn valid_type_transition(environment: &Environment, expected_types_on_stack: Vec<UnifiedType>, result_type: &UnifiedType, input_frame: &Frame) -> &'static Frame<'static> {
    unimplemented!()
}

pub fn pop_matching_list<'l>(pop_from: Vec<UnifiedType<'l>>, pop: Vec<UnifiedType<'l>>) -> Vec<UnifiedType<'l>> {
    unimplemented!()
}

pub fn pop_matching_type<'l>(operand_stack: Vec<UnifiedType<'l>>, type_: UnifiedType<'l>) -> Option<(Vec<UnifiedType<'l>>, UnifiedType<'l>)> {
    unimplemented!()
}

pub fn size_of<'l>(unified_type: UnifiedType<'l>) -> u64 {
    unimplemented!()
}

pub fn push_operand_stack<'l>(operand_stack: Vec<UnifiedType<'l>>, type_: UnifiedType<'l>) -> Vec<UnifiedType<'l>> {
    unimplemented!()
}

pub fn operand_stack_has_legal_length<'l>(environment: &Environment, operand_stack: &Vec<UnifiedType<'l>>) -> bool {
    unimplemented!()
}

pub fn pop_category_1<'l>(types: Vec<UnifiedType<'l>>) -> Option<(UnifiedType<'l>, Vec<UnifiedType<'l>>)> {
    unimplemented!()
}

pub fn can_safely_push<'l>(environment: Environment, input_operand_stack: Vec<UnifiedType<'l>>, type_: UnifiedType<'l>) -> Option<Vec<UnifiedType<'l>>> {
    unimplemented!();
}

pub fn can_safely_push_list<'l>(environment: Environment, input_operand_stack: Vec<UnifiedType<'l>>, type_list: Vec<UnifiedType<'l>>) -> Option<Vec<UnifiedType<'l>>> {
    unimplemented!()
}

pub fn can_push_list<'l>(input_operand_stack: Vec<UnifiedType<'l>>, type_list: Vec<UnifiedType<'l>>) -> Option<Vec<UnifiedType<'l>>> {
    unimplemented!()
}

pub fn can_pop<'l>(input_frame: Frame, types: Vec<UnifiedType<'l>>) -> Option<Frame<'l>> {
    unimplemented!()
}

//pub fn nth1OperandStackIs


pub fn is_bootstrap_loader(loader: &String) -> bool {
    return loader == &"bl".to_string();//todo  what if someone defines a Loader class called bl
}

pub fn get_class_methods<'l>(class: &'l PrologClass) -> Vec<PrologClassMethod<'l>> {
    let mut res = vec![];
    for method_index in 0..class.class.methods.len() {
        res.push(PrologClassMethod { prolog_class: class, method_index })
    }
    res
}

pub fn class_is_final(class: &PrologClass) -> bool {
    class.class.access_flags & ACC_FINAL != 0
}

pub fn class_is_type_safe(class: &PrologClass) -> bool {
    if class_name(class.class) == "java/lang/Object" {
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
        let super_class = loaded_class_(super_class_name, "bl".to_string()).unwrap();//todo magic string
        if class_is_final(&super_class) {
            return false;
        }
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
        if access_flags & ACC_NATIVE != 0 {
            true
        } else if access_flags & ACC_ABSTRACT != 0 {
            true
        } else {
            //will have a code attribute.
            /*let attributes = get_attributes(class, method);
            attributes.iter().any(|_| {
                unimplemented!()
            }) && */method_with_code_is_type_safe(class, method)
        };
}

pub fn get_parsed_code_attribute<'l>(class: &PrologClass<'l>, method: &PrologClassMethod<'l>) -> ParseCodeAttribute<'l> {
    let method_info = &class.class.methods[method.method_index];
    let code = code_attribute(method_info).unwrap();
    unimplemented!()
//    ParseCodeAttribute {
//
//    }
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
    let env = Environment { method, max_stack, frame_size: frame_size as u16, merged_code: Some(merged), class_loader: class.loader.as_str(), handlers };
    handers_are_legal(&env) && merged_code_is_type_safe(&env, merged.as_slice(), &frame, false)
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
    pub target: usize,
    pub class_name: Option<String>,
    //todo
}

pub fn handler_exception_class(handler: &Handler) -> PrologClass {
    match handler.class_name {
        None => { unimplemented!("Return java/lang/Throwable") }
        Some(s) => { unimplemented!("Need to get class from state") }
    }
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
    method: &'l PrologClassMethod<'l>,
    frame_size: u16,
    max_stack: u16,
    merged_code: Option<Vec<MergedCodeInstruction<'l>>>,
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
fn merge_stack_map_and_code<'l>(instruction: Vec<Instruction>, stack_maps: Vec<StackMap<'l>>) -> Vec<MergedCodeInstruction<'l>> {
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

fn method_initial_stack_frame<'l>(class: &PrologClass, method: &PrologClassMethod) -> (Frame<'l>, u64, UnifiedType<'l>) {
    unimplemented!()
}

fn expand_type_list<'l>(list: Vec<UnifiedType<'l>>) -> Vec<UnifiedType<'l>> {
    unimplemented!()
}

//fn flags()

fn expand_to_length<'l>(list: Vec<UnifiedType<'l>>, size: usize, filler: UnifiedType<'l>) -> Vec<UnifiedType<'l>> {
    unimplemented!()
}

fn method_initial_this_type<'l>(class: &PrologClass, method: &PrologClassMethod) -> Option<UnifiedType<'l>> {
    unimplemented!()
}

fn instance_method_initial_this_type<'l>(class: &PrologClass, method: &PrologClassMethod) -> bool {
    unimplemented!()
}

//todo how to handle other values here
fn merged_code_is_type_safe(env: &Environment, merged_code: &[MergedCodeInstruction], after_frame: &Frame, after_goto: bool) -> bool {
    let first = &merged_code[0];
    let rest = &merged_code[1..merged_code.len()];
    match first {
        MergedCodeInstruction::Instruction(i) => {
            let instruction_res = instruction_is_type_safe(&i.instruction, env, i.offset, after_frame).unwrap();//todo unwrap
            let exception_stack_frame1 = instruction_satisfies_handlers(env, i.offset, instruction_res.exception_frame);
            merged_code_is_type_safe(env, rest, instruction_res.next_frame, false)
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

fn class_to_type<'l>(class: &'l PrologClass<'l>) -> UnifiedType<'l> {
    UnifiedType::ReferenceType(&ClassNameReference::Ref(NameReference {
        index: class.class.this_class,
        class_file: class.class,
    }))
}

fn instruction_satisfies_handler(env: &Environment, exc_stack_frame: &Frame, handler: &Handler) -> bool {
    let target = handler.target;
    let class_loader = env.class_loader;
    let exception_class = handler_exception_class(handler);
    let locals = &exc_stack_frame.locals;
    let flags = exc_stack_frame.flag_this_uninit;
    let true_exc_stack_frame = Frame { locals, stack_map: &vec![class_to_type(&exception_class)], flag_this_uninit: flags };
    operand_stack_has_legal_length(env, &vec![class_to_type(&exception_class)]) &&
        target_is_type_safe(env, &true_exc_stack_frame, target)
}

fn nth0<'l>(index: usize, locals: &Vec<UnifiedType<'l>>) -> UnifiedType<'l> {
    unimplemented!()
}

fn load_is_type_safe<'l>(env: &Environment, index: usize, type_: &UnifiedType<'l>, frame: &Frame, next_frame: &Frame) -> bool {
    let locals = &frame.locals;
    let actual_type = nth0(index, locals);
    let type_transition = valid_type_transition(env, vec![], &actual_type, frame);

    is_assignable(&actual_type, type_) &&
        type_transition == next_frame
}

fn store_is_type_safe<'l>(env: &Environment, index: usize, type_: &UnifiedType<'l>, frame: &Frame, next_frame: &Frame) {
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

pub struct InstructionIsTypeSafeResult<'l> {
    next_frame: &'l Frame<'l>,
    exception_frame: &'l Frame<'l>,
}

fn instruction_is_type_safe<'l>(instruction: &InstructionInfo, env: &Environment, offset: usize, stack_frame: &Frame) -> Option<InstructionIsTypeSafeResult<'l>> {
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

fn instruction_is_type_safe_newarray(type_code: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
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

fn instruction_is_type_safe_sipush(value: usize, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_swap(env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn instruction_is_type_safe_tableswitch(targets: Vec<usize>, keys: Vec<usize>, env: &Environment, offset: usize, stack_frame: &Frame, next_frame: &Frame, exception_frame: &Frame) -> bool {
    unimplemented!()
}

fn different_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}

fn same_package_name(class1: &PrologClass, class2: &PrologClass) -> bool {
    unimplemented!()
}