use log::trace;
use std::sync::Arc;
use rust_jvm_common::unified_types::ClassWithLoader;
use rust_jvm_common::classnames::{get_referred_name, ClassName};
use crate::verifier::codecorrectness::{Environment, method_is_type_safe};
use crate::verifier::filecorrectness::{super_class_chain, class_is_final, is_bootstrap_loader, get_class_methods};
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::utils::get_super_class_name;
use crate::VerifierContext;
use crate::verifier::filecorrectness::loaded_class;
use crate::OperandStack;
use crate::verifier::filecorrectness::different_runtime_package;
use crate::verifier::filecorrectness::is_protected;
use classfile_parser::types::Descriptor;
use rust_jvm_common::unified_types::VerificationType;
use rust_jvm_common::unified_types::ParsedType;


macro_rules! unknown_error_verifying {
    () => {
        TypeSafetyError::NotSafe(format!("An unknown error occurred while verifying:{}:{}", file!(), line!()))
    };
}


pub mod instructions;
pub mod filecorrectness;
pub mod codecorrectness;

pub struct InternalFrame {
    pub locals: Vec<ParsedType>,
    pub stack: Vec<ParsedType>,
    pub max_locals: u16,
    pub current_offset: u16,
}

pub fn get_class(_verifier_context: &VerifierContext, class: &ClassWithLoader) -> Arc<Classfile> {
    //todo ideally we would just use parsed here so that we don't have infinite recursion in verify
    if class.loader.initiating_loader_of(&class.class_name) {
        match class.loader.load_class(&class.class_name) {
            Ok(c) => c,
            Err(_) => panic!(),
        }
    } else {
        match class.loader.pre_load(class.loader.clone(), &class.class_name) {
            Ok(c) => c,
            Err(_) => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct ClassWithLoaderMethod<'l> {
    pub class: &'l ClassWithLoader,
    pub method_index: usize,
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub struct Frame {
    pub locals: Vec<VerificationType>,
    pub stack_map: OperandStack,
    pub flag_this_uninit: bool,
}


#[derive(Debug)]
pub enum TypeSafetyError {
    NotSafe(String),
    NeedToLoad(Vec<ClassName>),
}

pub fn class_is_type_safe(vf: &VerifierContext, class: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    if get_referred_name(&class.class_name) == "java/lang/Object" {
        if !is_bootstrap_loader(vf, &class.loader) {
            return Result::Err(TypeSafetyError::NotSafe("Loading object with something other than bootstrap loader".to_string()));
        }
        trace!("Class was java/lang/Object, skipping lots of overriding checks");
    } else {
        trace!("Class not java/lang/Object performing superclass checks");
        //class must have a superclass or be 'java/lang/Object'
        let mut chain = vec![];
        super_class_chain(vf, class, class.loader.clone(), &mut chain)?;
        if chain.is_empty() {
            return Result::Err(TypeSafetyError::NotSafe("No superclass but object is not Object".to_string()));
        }
        let super_class_name = get_super_class_name(&get_class(vf, class));
        let super_class = loaded_class(vf, super_class_name, vf.bootstrap_loader.clone()).unwrap();
        if class_is_final(vf, &super_class) {
            return Result::Err(TypeSafetyError::NotSafe("Superclass is final".to_string()));
        }
    }
    let methods = get_class_methods(vf, class);
    trace!("got class methods:");
    let method_type_safety: Result<Vec<()>, _> = methods.iter().map(|m| {
        let res = method_is_type_safe(vf, class, m);
        trace!("method was:");
        dbg!(&res);
        //return early:
        match res {
            Ok(_) => {}
            Err(e) => {
                return Result::Err(e);//return early on error for debugging purposes
            }
        }
        res
    }).collect();
    method_type_safety?;
    Ok(())
}

pub fn passes_protected_check(_env: &Environment, _member_class_name: &ClassName, _member_name: String, _member_descriptor: Descriptor, _stack_frame: &Frame) -> Result<(), TypeSafetyError> {
// todo waiting on stackoverflow / further clarification
    Result::Ok(())
//    let mut chain = vec![];
//    super_class_chain(&env.vf, env.method.prolog_class, env.class_loader.clone(), &mut chain)?;//todo is this strictly correct?
//    if chain.iter().any(|x| {
//        &x.class_name == member_class_name
//    }) {
//        //not my descriptive variable name
//        //the spec's name not mine
//        dbg!(&chain);
//        let list = classes_in_other_pkg_with_protected_member(&env.vf, env.method.prolog_class, member_name.clone(), &member_descriptor, member_class_name.clone(), chain)?;
//        dbg!(&list);
//        if list.is_empty() {
//            Result::Ok(())
//        } else {
//            let referenced_class = loaded_class(&env.vf, member_class_name.clone(), env.class_loader.clone())?;
//            let protected = is_protected(&env.vf, &referenced_class, member_name.clone(), &member_descriptor);
//            dbg!(protected);
//            if protected {
//                is_assignable(&env.vf,&stack_frame.stack_map.peek(),&UnifiedType::Class(env.method.prolog_class.clone()))
//            }else {
//                Result::Ok(())
//            }
//        }
//    } else {
//        Result::Ok(())
//    }
}


pub fn classes_in_other_pkg_with_protected_member(vf: &VerifierContext, class: &ClassWithLoader, member_name: String, member_descriptor: &Descriptor, member_class_name: ClassName, chain: Vec<ClassWithLoader>) -> Result<Vec<ClassWithLoader>,TypeSafetyError> {
    let mut res = vec![];
    classes_in_other_pkg_with_protected_member_impl(vf,class,member_name,member_descriptor,member_class_name,chain.as_slice(),&mut res)?;
    Result::Ok(res)
}


fn classes_in_other_pkg_with_protected_member_impl(
    vf: &VerifierContext,
    class: &ClassWithLoader,
    member_name: String,
    member_descriptor: &Descriptor,
    member_class_name: ClassName,
    chain: &[ClassWithLoader],
    res: &mut Vec<ClassWithLoader>) -> Result<(),TypeSafetyError> {
    if !chain.is_empty() {
        let first = &chain[0];
        let rest = &chain[1..];
        if first.class_name != member_class_name{
            dbg!(&chain);
            dbg!(&member_class_name);
            panic!();
            return Result::Err(unknown_error_verifying!())
        }
        let l = first.loader.clone();
        if different_runtime_package(vf, class,first){
            let super_ = loaded_class(vf,member_class_name.clone(),l)?;
            if is_protected(vf,&super_,member_name.clone(),member_descriptor){
                dbg!(&res);
                res.push(first.clone())
            }
        }
        classes_in_other_pkg_with_protected_member_impl(
            vf,
            class,
            member_name.clone(),
            member_descriptor,
            member_class_name.clone(),
            rest,
            res)?;
    }
    Result::Ok(())
}

