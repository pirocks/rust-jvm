use class_loading::class_entry;
use class_loading::Loader;
use classfile::{ACC_NATIVE, ACC_PRIVATE, ACC_STATIC, Classfile, stack_map_table_attribute};
use classfile::ACC_ABSTRACT;
use classfile::ACC_FINAL;
use classfile::ACC_INTERFACE;
use classfile::attribute_infos::StackMapTable;
use classfile::code::Instruction;
use classfile::code::InstructionInfo;
use classfile::code_attribute;
use std::rc::Rc;
use verification::code_writer::ParseCodeAttribute;
use verification::code_writer::StackMap;
use verification::prolog_info_writer::{class_name_legacy, get_access_flags, get_super_class_name};
use verification::unified_type::ClassNameReference;
use verification::unified_type::NameReference;
use verification::unified_type::UnifiedType;
use verification::verifier::TypeSafetyResult::{NeedToLoad, NotSafe, Safe};
use verification::verifier::filecorrectness::{is_bootstrap_loader, super_class_chain, class_is_final, loaded_class_, get_class_methods};
use verification::verifier::codecorrectness::{method_is_type_safe, Environment};

pub mod instructions;
pub mod filecorrectness;
pub mod codecorrectness;

pub struct InternalFrame {
    pub locals: Vec<UnifiedType>,
    pub stack: Vec<UnifiedType>,
    pub max_locals: u16,
    pub current_offset: u16,
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

#[derive(Eq, PartialEq)]
pub struct Frame<'l> {
    pub locals: &'l Vec<UnifiedType>,
    pub stack_map: Vec<UnifiedType>,
    pub flag_this_uninit: bool,
}

//pub fn nth1OperandStackIs

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
