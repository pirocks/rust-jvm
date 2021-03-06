extern crate elapsed;

use std::collections::vec_deque::VecDeque;
use std::sync::Arc;

use elapsed::measure_time;

use classfile_view::loading::{ClassWithLoader, LivePoolGetter, LoaderName};
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::vtype::VType;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;

use crate::verifier::class_is_type_safe;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;

pub mod verifier;

pub fn verify(vf: &VerifierContext, to_verify: &ClassBackedView, loader: LoaderName) -> Result<(), TypeSafetyError> {
    let (_time, res) = measure_time(|| match class_is_type_safe(vf, &ClassWithLoader {
        class_name: to_verify.name(),
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
    });
    res
}

#[derive(Debug)]
pub struct StackMap {
    pub offset: usize,
    pub map_frame: Frame,
}

pub struct VerifierContext<'l> {
    pub live_pool_getter: Arc<dyn LivePoolGetter>,
    pub classfile_getter: Arc<dyn ClassFileGetter + 'l>,
    // pub classes: &'l ,
    pub current_loader: LoaderName,
}


pub trait ClassFileGetter {
    fn get_classfile(&self, loader: LoaderName, class: ClassName) -> Arc<Classfile>;
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

    pub fn new_prolog_display_order(types: &[VType]) -> OperandStack {
        let mut o = OperandStack::empty();
        for type_ in types {
            o.operand_push(type_.clone())
        }
        o
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


