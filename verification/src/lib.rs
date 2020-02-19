extern crate log;
extern crate simple_logger;

use std::sync::Arc;
use crate::verifier::class_is_type_safe;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::class_name;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;
use std::collections::vec_deque::VecDeque;
use rust_jvm_common::loading::{LoaderArc, ClassWithLoader};
use rust_jvm_common::vtype::VType;

pub mod verifier;


pub fn verify(vf: &VerifierContext, to_verify: Arc<Classfile>, loader: LoaderArc) -> Result<(), TypeSafetyError> {
    match class_is_type_safe(vf, &ClassWithLoader {
        class_name: class_name(&to_verify),
        loader,
    }) {
        Ok(_) => Result::Ok(()),
        Err(err) => {
            match err {
                TypeSafetyError::NotSafe(s) => {
                    dbg!(s);
                    unimplemented!()
                }
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

pub struct VerifierContext {
    pub bootstrap_loader: LoaderArc
}

impl Clone for VerifierContext {
    fn clone(&self) -> Self {
        VerifierContext { bootstrap_loader: self.bootstrap_loader.clone() }
    }
}

#[derive(Eq, Debug)]
pub struct OperandStack {
    data: VecDeque<VType>
}

impl Clone for OperandStack {
    fn clone(&self) -> Self {
        OperandStack {
            data: self.data.clone()
        }
    }
}

impl PartialEq for OperandStack {
    fn eq(&self, _other: &OperandStack) -> bool {
        unimplemented!()
    }
}


impl OperandStack {
    pub fn operand_push(&mut self, type_: VType) {
        self.data.push_front(type_);
    }

    pub fn operand_pop(&mut self) -> VType {
        self.data.pop_front().unwrap()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn peek(&self) -> VType {
        self.data.front().unwrap().clone()
    }

    pub fn new_prolog_display_order(types: &Vec<VType>) -> OperandStack {
        let mut o = OperandStack::empty();
        for type_ in types {
            o.operand_push(type_.clone())
        }
        o
    }

    pub fn new_reverse_display_order(_types: &Vec<VType>) -> OperandStack {
        unimplemented!()
    }

    pub fn empty() -> OperandStack {
        OperandStack { data: VecDeque::new() }
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<'_, VType> {
        self.data.iter()
    }

    pub fn substitute(&mut self, old: &VType, new: &VType) {
        for entry in &mut self.data {
            if entry == old {
                *entry = new.clone();
            }
        }
    }
}


