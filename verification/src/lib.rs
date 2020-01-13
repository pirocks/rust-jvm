extern crate tempfile;
extern crate log;
extern crate simple_logger;

use std::sync::Arc;
use crate::verifier::class_is_type_safe;
use rust_jvm_common::loading::Loader;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::verifier::InternalFrame;
use rust_jvm_common::unified_types::ArrayType;
use rust_jvm_common::classnames::{get_referred_name, class_name};
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;
use std::collections::vec_deque::VecDeque;
use rust_jvm_common::unified_types::VerificationType;
use rust_jvm_common::unified_types::ParsedType;


pub mod verifier;


/**
We can only verify one class at a time, all needed classes need to be in jvm state as loading, including the class to verify.
*/
pub fn verify(vf:&VerifierContext,to_verify: Arc<Classfile>,loader: Arc<dyn Loader + Send + Sync>) -> Result<(), TypeSafetyError> {
    match class_is_type_safe(vf,&ClassWithLoader {
        class_name: class_name(&to_verify),
        loader,
    }) {
        Ok(_) => Result::Ok(()),
        Err(err) => {
            match err {
                TypeSafetyError::NotSafe(_) => unimplemented!(),
                TypeSafetyError::NeedToLoad(_) => unimplemented!(),
            }
        }
    }
}


#[derive(Debug)]
pub struct StackMap {
    pub offset: usize,
    pub map_frame: Frame,
}


pub fn init_frame(parameter_types: Vec<ParsedType>, this_pointer: Option<ParsedType>, max_locals: u16) -> InternalFrame {
    let mut locals = Vec::with_capacity(max_locals as usize);
    match this_pointer {
        None => {}//class is static etc.
        Some(t) => {
            locals_push_convert_type(&mut locals, t)
        }
    }
    for parameter_type in parameter_types {
        locals_push_convert_type(&mut locals, parameter_type)
    }
    InternalFrame { max_locals, locals, stack: Vec::new(), current_offset: 0 }
}

fn locals_push_convert_type(res: &mut Vec<VerificationType>, type_: ParsedType) -> () {
    match type_ {
        ParsedType::ByteType => {
            res.push(VerificationType::IntType);
        }
        ParsedType::CharType => {
            res.push(VerificationType::IntType);
        }
        ParsedType::DoubleType => {
            res.push(VerificationType::DoubleType);
            res.push(VerificationType::TopType);
        }
        ParsedType::FloatType => {
            res.push(VerificationType::FloatType);
        }
        ParsedType::IntType => {
            res.push(VerificationType::IntType);
        }
        ParsedType::LongType => {
            res.push(VerificationType::LongType);
            res.push(VerificationType::TopType);
        }
        ParsedType::Class(r) => {
            assert_ne!(get_referred_name(&r.class_name).chars().nth(0).unwrap(), '[');
            res.push(VerificationType::Class(r));
        }
        ParsedType::ShortType => {
            res.push(VerificationType::IntType);
        }
        ParsedType::BooleanType => {
            res.push(VerificationType::IntType);
        }
        ParsedType::ArrayReferenceType(art) => {
            res.push(VerificationType::ArrayReferenceType(art.clone()));
        }
        ParsedType::VoidType => { panic!() }
        _ => { panic!("Case wasn't coverred with non-unified types") }
    }
}


pub struct VerifierContext{
    pub bootstrap_loader : Arc<dyn Loader + Send + Sync>
}

impl Clone for VerifierContext{
    fn clone(&self) -> Self {
        VerifierContext { bootstrap_loader: self.bootstrap_loader.clone() }
    }
}

#[derive(Eq,Debug)]
pub struct OperandStack{
    data: VecDeque<VerificationType>
}

impl Clone for OperandStack{
    fn clone(&self) -> Self {
        OperandStack {
            data: self.data.clone()
        }
    }
}

impl PartialEq for OperandStack{
    fn eq(&self, _other: &OperandStack) -> bool {
        unimplemented!()
    }
}


impl OperandStack{

    pub fn operand_push(&mut self, type_: VerificationType){
        self.data.push_front(type_);
    }

    pub fn operand_pop(&mut self) -> VerificationType{
        self.data.pop_front().unwrap()
    }

    pub fn len(&self) -> usize{
        self.data.len()
    }

    pub fn peek(&self) -> VerificationType{
        self.data.front().unwrap().clone()
    }

    pub fn new_prolog_display_order(types: &Vec<VerificationType>) -> OperandStack{
        dbg!(types);
        let mut o = OperandStack::empty();
        for type_ in types{
            o.operand_push(type_.clone())
        }
        dbg!(&o);
        o
    }

    pub fn new_reverse_display_order(_types: &Vec<VerificationType>) -> OperandStack{
        unimplemented!()
    }

    pub fn empty() -> OperandStack{
        OperandStack { data: VecDeque::new() }
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, VerificationType>{
        self.data.iter()
    }

    pub(crate) fn substitute(&mut self, old: & VerificationType, new: &VerificationType){
        for entry in &mut self.data{
            if entry == old{
                *entry = new.clone();
            }
        }
    }
}
