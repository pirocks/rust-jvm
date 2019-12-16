use class_loading::Loader;
use classfile::Classfile;
use verification::prolog_info_writer::{get_super_class_name, class_name};
use verification::unified_type::UnifiedType;
use verification::verifier::TypeSafetyResult::{NeedToLoad};
use verification::verifier::filecorrectness::{is_bootstrap_loader, super_class_chain, class_is_final, loaded_class_, get_class_methods};
use verification::verifier::codecorrectness::{method_is_type_safe, Environment};
use verification::classnames::{ClassName, get_referred_name};
use log::trace;
use std::sync::Arc;

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

#[derive(Debug)]
pub struct PrologClass {
    pub loader: Arc<Loader>,
    pub class: Arc<Classfile>,
}

#[derive(Debug)]
pub struct PrologClassMethod<'l> {
    pub prolog_class: &'l PrologClass,
    pub method_index: usize,
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub struct Frame {
    pub locals: Vec<UnifiedType>,
    pub stack_map: Vec<UnifiedType>,
    pub flag_this_uninit: bool,
}

//pub fn nth1OperandStackIs

#[derive(Debug)]
pub enum TypeSafetyResult {
    NotSafe(String),
    //reason is a String
    Safe(),
    NeedToLoad(Vec<ClassName>),
}

pub fn class_is_type_safe(class: &PrologClass ) -> TypeSafetyResult {
    if get_referred_name(&class_name(&class.class)) == "java/lang/Object" {
        if !is_bootstrap_loader(&class.loader) {
            return TypeSafetyResult::NotSafe("Loading object with something other than bootstrap loader".to_string());
        }
        trace!("Class was java/lang/Object, skipping lots of overriding checks");
    } else {
        trace!("Class not java/lang/Object performing superclass checks");
        //class must have a superclass or be 'java/lang/Object'
        //todo loader shouldnt really be a string
        let mut chain = vec![];
        let chain_res = super_class_chain(class, class.loader.clone(), &mut chain);
        unimplemented!();
        if chain.is_empty() {
            return TypeSafetyResult::NotSafe("No superclass but object is not Object".to_string());
        }
        let super_class_name = get_super_class_name(&class.class);
        let super_class = loaded_class_(super_class_name, "bl".to_string()).unwrap();//todo magic string
        if class_is_final(&super_class) {
            return TypeSafetyResult::NotSafe("Superclass is final".to_string());
        }
    }
    let methods = get_class_methods(class);
    trace!("got class methods:");
    dbg!(&methods);
    let method_type_safety: Vec<TypeSafetyResult> = methods.iter().map(|m| {
        let res = method_is_type_safe(class, m);
        trace!("method was:");
        dbg!(&res);
        res
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
