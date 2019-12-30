use log::trace;
use std::sync::Arc;
use rust_jvm_common::unified_types::{UnifiedType, ClassWithLoader};
use rust_jvm_common::classnames::{get_referred_name, ClassName};
use crate::verifier::codecorrectness::{Environment, method_is_type_safe};
use crate::verifier::filecorrectness::{super_class_chain, loaded_class_, class_is_final, is_bootstrap_loader, get_class_methods};
use crate::types::MethodDescriptor;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::class_entry_from_string;
use rust_jvm_common::loading::BOOTSTRAP_LOADER;
use rust_jvm_common::utils::get_super_class_name;


macro_rules! unknown_error_verifying {
    () => {
        TypeSafetyError::NotSafe(format!("An unknown error occurred while verifying:{}:{}", file!(), line!()))
    };
}


pub mod instructions;
pub mod filecorrectness;
pub mod codecorrectness;

pub struct InternalFrame {
    pub locals: Vec<UnifiedType>,
    pub stack: Vec<UnifiedType>,
    pub max_locals: u16,
    pub current_offset: u16,
}

pub fn get_class(class: &ClassWithLoader) -> Arc<Classfile> {
    let referred_name = get_referred_name(&class.class_name);
    let class_entry = class_entry_from_string(&referred_name, false);
    match class.loader.loaded.read().unwrap().get(&class_entry) {
        None => {
            let map = class.loader.loading.read().unwrap();
            let option = map.get(&class_entry);
            match option {
                None => {
                    dbg!(map.keys());
                    panic!()
                }
                Some(c) => c.clone(),
            }
        }
        Some(c) => c.clone(),
    }
}

#[derive(Debug)]
pub struct ClassWithLoaderMethod<'l> {
    pub prolog_class: &'l ClassWithLoader,
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
pub enum TypeSafetyError {
    NotSafe(String),
    NeedToLoad(Vec<ClassName>),
}

pub fn class_is_type_safe(class: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    if get_referred_name(&class.class_name) == "java/lang/Object" {
        if !is_bootstrap_loader(&class.loader) {
            return Result::Err(TypeSafetyError::NotSafe("Loading object with something other than bootstrap loader".to_string()));
        }
        trace!("Class was java/lang/Object, skipping lots of overriding checks");
    } else {
        trace!("Class not java/lang/Object performing superclass checks");
        //class must have a superclass or be 'java/lang/Object'
        let mut chain = vec![];
        super_class_chain(class, class.loader.clone(), &mut chain)?;
        if chain.is_empty() {
            return Result::Err(TypeSafetyError::NotSafe("No superclass but object is not Object".to_string()));
        }
        let super_class_name = get_super_class_name(&get_class(class));
        let super_class = loaded_class_(super_class_name, BOOTSTRAP_LOADER.clone()).unwrap();//todo magic string
        if class_is_final(&super_class) {
            return Result::Err(TypeSafetyError::NotSafe("Superclass is final".to_string()));
        }
    }
    let methods = get_class_methods(class);
    trace!("got class methods:");
    let method_type_safety: Result<Vec<()>, _> = methods.iter().map(|m| {
        let res = method_is_type_safe(class, m);
        trace!("method was:");
        dbg!(&res);
        res
    }).collect();
    method_type_safety?;
    Ok(())
}


pub enum Descriptor {}

fn passes_protected_check(env: &Environment, member_class_name: String, _member_name: String/*, _member_descriptor: !*/, _stack_frame: &Frame) -> Result<(), TypeSafetyError> {
    let mut chain = vec![];
    super_class_chain(env.method.prolog_class, env.class_loader.clone(), &mut chain)?;//todo is this strictly correct?
    if chain.iter().any(|x| { get_referred_name(&x.class_name) == member_class_name }) {
        unimplemented!()
    } else {
        Result::Ok(())
    }
}

#[allow(unused)]
fn classes_in_other_pkg_with_protected_member(_class: &ClassWithLoader, _member_name: String, _member_descriptor: &MethodDescriptor, _member_class_name: String, _chain: Vec<ClassWithLoader>) -> Vec<ClassWithLoader> {
    unimplemented!()
}
