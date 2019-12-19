use log::trace;
use std::sync::Arc;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::classnames::{get_referred_name, class_name, ClassName};
use crate::verification::verifier::codecorrectness::{Environment, method_is_type_safe};
use crate::verification::verifier::filecorrectness::{super_class_chain, loaded_class_, class_is_final, is_bootstrap_loader, get_class_methods};
use crate::verification::types::MethodDescriptor;
use crate::verification::verifier::TypeSafetyResult::{Safe, NotSafe, NeedToLoad};
use crate::verification::prolog_info_writer::get_super_class_name;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::Loader;

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
#[derive(Eq)]
pub enum TypeSafetyResult {
    NotSafe(String),
    //reason is a String
    Safe(),
    NeedToLoad(Vec<ClassName>),
}

pub fn and(left: TypeSafetyResult, right: TypeSafetyResult) -> TypeSafetyResult{
    return merge_type_safety_results(vec![left,right].into_boxed_slice());
}

impl PartialEq for TypeSafetyResult{
    fn eq(&self, other: &TypeSafetyResult) -> bool {
        match self {
            Safe() => match other {
                Safe() => true,
                _ => false
            },
            NotSafe(s1) => match other {
                TypeSafetyResult::NotSafe(s2) => {s1 == s2},
                _ => false
            }
            _ => {unimplemented!()}
        }
    }
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

pub fn merge_type_safety_results(method_type_safety: Box<[TypeSafetyResult]>) -> TypeSafetyResult {
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

pub enum Descriptor {}

//fn modify_local_variable() //todo

fn passes_protected_check(env: &Environment, member_class_name: String, member_name: String, member_descriptor: &MethodDescriptor, stack_frame: &Frame) -> bool {
    let mut chain = vec![];
    super_class_chain(env.method.prolog_class,env.class_loader.clone(),&mut chain);//todo is this strictly correct?
    if chain.iter().any(|x|{get_referred_name(&class_name(&x.class)) == member_class_name}){
        unimplemented!()
    }else {
        true
    }
}

fn classes_in_other_pkg_with_protected_member(class: &PrologClass, member_name:String, member_descriptor:&MethodDescriptor,member_class_name:String,chain: Vec<PrologClass>) -> Vec<PrologClass>{
    unimplemented!()
}
