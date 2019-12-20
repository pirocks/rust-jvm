use log::trace;
use std::sync::Arc;
use rust_jvm_common::unified_types::UnifiedType;
use rust_jvm_common::classnames::{get_referred_name, class_name, ClassName};
use crate::verifier::codecorrectness::{Environment, method_is_type_safe};
use crate::verifier::filecorrectness::{super_class_chain, loaded_class_, class_is_final, is_bootstrap_loader, get_class_methods};
use crate::types::MethodDescriptor;
use crate::prolog::prolog_info_writer::get_super_class_name;
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
pub enum TypeSafetyError{
    NotSafe(String),
    NeedToLoad(Vec<ClassName>)
}

pub fn class_is_type_safe(class: &PrologClass ) -> Result<(),TypeSafetyError> {
    if get_referred_name(&class_name(&class.class)) == "java/lang/Object" {
        if !is_bootstrap_loader(&class.loader) {
            return Result::Err(TypeSafetyError::NotSafe("Loading object with something other than bootstrap loader".to_string()));
        }
        trace!("Class was java/lang/Object, skipping lots of overriding checks");
    } else {
        trace!("Class not java/lang/Object performing superclass checks");
        //class must have a superclass or be 'java/lang/Object'
        //todo loader shouldnt really be a string
        let mut chain = vec![];
        let _chain_res = super_class_chain(class, class.loader.clone(), &mut chain);
        unimplemented!();
        if chain.is_empty() {
            return Result::Err(TypeSafetyError::NotSafe("No superclass but object is not Object".to_string()));
        }
        let super_class_name = get_super_class_name(&class.class);
        let super_class = loaded_class_(super_class_name, "bl".to_string()).unwrap();//todo magic string
        if class_is_final(&super_class) {
            return Result::Err(TypeSafetyError::NotSafe("Superclass is final".to_string()));
        }
    }
    let methods = get_class_methods(class);
    trace!("got class methods:");
    dbg!(&methods);
    let method_type_safety: Result<Vec<()>,_> = methods.iter().map(|m| {
        let res = method_is_type_safe(class, m);
        trace!("method was:");
        dbg!(&res);
        res
    }).collect();
    method_type_safety?;
    Ok(())
}


pub struct FieldDescriptor {
    //todo
}

pub enum Descriptor {}

//fn modify_local_variable() //todo

fn passes_protected_check(env: &Environment, member_class_name: String, _member_name: String, _member_descriptor: &MethodDescriptor, _stack_frame: &Frame) -> Result<(),TypeSafetyError> {
    let mut chain = vec![];
    super_class_chain(env.method.prolog_class,env.class_loader.clone(),&mut chain)?;//todo is this strictly correct?
    if chain.iter().any(|x|{get_referred_name(&class_name(&x.class)) == member_class_name}){
        unimplemented!()
    }else {
        Result::Ok(())
    }
}

#[allow(unused)]
fn classes_in_other_pkg_with_protected_member(_class: &PrologClass, _member_name:String, _member_descriptor:&MethodDescriptor,_member_class_name:String,_chain: Vec<PrologClass>) -> Vec<PrologClass>{
    unimplemented!()
}
