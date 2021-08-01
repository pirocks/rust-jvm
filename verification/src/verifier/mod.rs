use std::rc::Rc;
use std::sync::Arc;

use classfile_view::view::ClassView;
use rust_jvm_common::compressed_classfile::CCString;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::descriptor_parser::Descriptor;
use rust_jvm_common::loading::*;
use rust_jvm_common::vtype::VType;

use crate::OperandStack;
use crate::verifier::codecorrectness::{Environment, method_is_type_safe};
use crate::verifier::filecorrectness::{class_is_final, get_class_methods, is_bootstrap_loader, super_class_chain};
use crate::verifier::filecorrectness::different_runtime_package;
use crate::verifier::filecorrectness::is_protected;
use crate::verifier::filecorrectness::loaded_class;
use crate::verifier::instructions::{exception_stack_frame, InstructionTypeSafe, ResultFrames};
use crate::VerifierContext;

macro_rules! unknown_error_verifying {
    () => {
        TypeSafetyError::NotSafe(format!("An unknown error occurred while verifying:{}:{}", file!(), line!()))
    };
}


pub mod instructions;
pub mod filecorrectness;
pub mod codecorrectness;

pub struct InternalFrame {
    pub locals: Vec<VType>,
    pub stack: Vec<VType>,
    pub max_locals: u16,
    pub current_offset: u16,
}

//todo impl on VerifierContext
pub fn get_class(verifier_context: &VerifierContext, class: &ClassWithLoader) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
    let mut guard = verifier_context.class_view_cache.lock().unwrap();
    match guard.get(class) {
        None => {
            let res = verifier_context.classfile_getter.get_classfile(class.loader, class.class_name.clone())?;
            guard.insert(class.clone(), res.clone());
            Ok(res)
        }
        Some(res) => Ok(res.clone())
    }
    // Arc::new(ClassView::from(verifier_context.classes.pre_load(class.class_name.clone(), class.loader.clone()).unwrap()))

    //todo ideally we would just use parsed here so that we don't have infinite recursion in verify
    // if class.loader.initiating_loader_of(&class.class_name) {
    //     // verifier_context.jvm
    //     //todo maybe trace load here
    //     match class.loader.clone().load_class(class.loader.clone(), &class.class_name, verifier_context.current_loader.clone(), verifier_context.live_pool_getter.clone()) {
    //         Ok(c) => c,
    //         Err(_) => panic!(),
    //     }
    // } else {
    //     match class.loader.pre_load(&class.class_name) {
    //         Ok(c) => c,
    //         Err(_) => panic!(),
    //     }
    // }
    // unimplemented!()
}

#[derive(Debug, Clone)]
pub struct ClassWithLoaderMethod {
    pub class: ClassWithLoader,
    pub method_index: usize,
}

#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub struct Frame {
    pub locals: Rc<Vec<VType>>,
    pub stack_map: OperandStack,
    pub flag_this_uninit: bool,
}

//todo in future get rid of this clone implementation
impl Clone for Frame {
    fn clone(&self) -> Self {
        Self {
            locals: self.locals.clone(),
            stack_map: self.stack_map.clone(),
            flag_this_uninit: self.flag_this_uninit,
        }
    }
}


#[derive(Debug)]
pub enum TypeSafetyError {
    NotSafe(String),
    Java5Maybe,
    ClassNotFound(ClassLoadingError),
}

impl From<ClassLoadingError> for TypeSafetyError {
    fn from(err: ClassLoadingError) -> Self {
        TypeSafetyError::ClassNotFound(err)
    }
}

pub fn class_is_type_safe(vf: &mut VerifierContext, class: &ClassWithLoader) -> Result<(), TypeSafetyError> {
    if class.class_name == CClassName::object() {
        if !is_bootstrap_loader(&class.loader) {
            return Result::Err(TypeSafetyError::NotSafe("Loading object with something other than bootstrap loader".to_string()));
        }
    } else {
        let mut chain = vec![];
        super_class_chain(vf, class, class.loader.clone(), &mut chain)?;
        if chain.is_empty() {
            return Result::Err(TypeSafetyError::NotSafe("No superclass but object is not Object".to_string()));
        }
        let super_class_name = get_class(vf, class)?.super_name();
        let super_class = loaded_class(vf, super_class_name.unwrap(), vf.current_loader.clone()).unwrap();
        if class_is_final(vf, &super_class)? {
            return Result::Err(TypeSafetyError::NotSafe("Superclass is final".to_string()));
        }
    }
    let methods = get_class_methods(vf, class.clone())?;
    let method_type_safety: Result<Vec<()>, _> = methods.iter().map(|m| {
        method_is_type_safe(vf, class, m)
    }).collect();
    method_type_safety?;
    Ok(())
}

pub fn passes_protected_check(_env: &Environment, _member_class_name: CClassName, _member_name: CCString, _member_descriptor: Descriptor, _stack_frame: &Frame) -> Result<(), TypeSafetyError> {
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


pub fn classes_in_other_pkg_with_protected_member(vf: &VerifierContext, class: &ClassWithLoader, member_name: CCString, member_descriptor: &Descriptor, member_class_name: CClassName, chain: Vec<ClassWithLoader>) -> Result<Vec<ClassWithLoader>, TypeSafetyError> {
    let mut res = vec![];
    classes_in_other_pkg_with_protected_member_impl(vf, class, member_name, member_descriptor, member_class_name, chain.as_slice(), &mut res)?;
    Result::Ok(res)
}


fn classes_in_other_pkg_with_protected_member_impl(
    vf: &VerifierContext,
    class: &ClassWithLoader,
    member_name: CCString,
    member_descriptor: &Descriptor,
    member_class_name: CClassName,
    chain: &[ClassWithLoader],
    res: &mut Vec<ClassWithLoader>) -> Result<(), TypeSafetyError> {
    if !chain.is_empty() {
        let first = &chain[0];
        let rest = &chain[1..];
        if first.class_name != member_class_name {
            dbg!(&chain);
            dbg!(&member_class_name);
            panic!();
        }
        let l = first.loader.clone();
        if different_runtime_package(vf, class, first) {
            let super_ = loaded_class(vf, member_class_name.clone(), l)?;
            if is_protected(vf, &super_, member_name.clone(), member_descriptor)? {
                res.push(first.clone())
            }
        }
        classes_in_other_pkg_with_protected_member_impl(
            vf,
            class,
            member_name,
            member_descriptor,
            member_class_name,
            rest,
            res)?;
    }
    Result::Ok(())
}


pub fn standard_exception_frame(stack_frame_locals: Rc<Vec<VType>>, stack_frame_flag: bool, next_frame: Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let exception_frame = exception_stack_frame(stack_frame_locals, stack_frame_flag);
    Result::Ok(InstructionTypeSafe::Safe(ResultFrames { next_frame, exception_frame }))
}


pub mod stackmapframes;