extern crate elapsed;

use std::collections::HashMap;
use std::collections::vec_deque::VecDeque;
use std::sync::Arc;

use classfile_view::loading::{ClassWithLoader, LivePoolGetter, LoaderName};
use classfile_view::view::{ClassBackedView, ClassView};
use classfile_view::vtype::VType;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::classnames::ClassName;

use crate::verifier::class_is_type_safe;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;

pub mod verifier;

pub fn verify(vf: &mut VerifierContext, to_verify: &ClassBackedView, loader: LoaderName) -> Result<(), TypeSafetyError> {
    class_is_type_safe(vf, &ClassWithLoader {
        class_name: to_verify.name().unwrap_name(),
        loader,
    })
}

#[derive(Debug)]
pub struct StackMap {
    pub offset: u16,
    pub map_frame: Frame,
}

pub struct VerifierContext<'l> {
    pub live_pool_getter: Arc<dyn LivePoolGetter + 'l>,
    pub classfile_getter: Arc<dyn ClassFileGetter + 'l>,
    // pub classes: &'l ,
    pub current_loader: LoaderName,
    pub verification_types: HashMap<u16, HashMap<u16, Frame>>,
    pub debug: bool,
}


pub trait ClassFileGetter {
    fn get_classfile(&self, loader: LoaderName, class: ClassName) -> Arc<Classfile>;
}

#[derive(Eq, Debug)]
pub struct OperandStack {
    pub data: VecDeque<VType>,
}

impl Clone for OperandStack {
    fn clone(&self) -> Self {
        OperandStack {
            data: self.data.clone()
        }
    }
}

impl PartialEq for OperandStack {
    fn eq(&self, other: &OperandStack) -> bool {
        self.data == other.data
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


