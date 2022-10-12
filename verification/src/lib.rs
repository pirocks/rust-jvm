use std::collections::HashMap;
use std::collections::vec_deque::VecDeque;
use std::sync::{Arc, Mutex};

use classfile_view::view::ClassView;
use perf_metrics::PerfMetrics;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;
use rust_jvm_common::loading::{ClassLoadingError, ClassWithLoader, LivePoolGetter, LoaderName};
use rust_jvm_common::vtype::VType;

use crate::verifier::class_is_type_safe;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;

pub mod verifier;

pub fn verify(vf: &mut VerifierContext, to_verify: CClassName, loader: LoaderName) -> Result<(), TypeSafetyError> {
    let verify = vf.perf_metrics.verifier_start();
    let res = class_is_type_safe(vf, &ClassWithLoader { class_name: to_verify, loader });
    drop(verify);
    res
}

#[derive(Debug)]
pub struct StackMap {
    pub offset: ByteCodeOffset,
    pub map_frame: Frame,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct CodeIndex(u16);


pub struct VerifierContext<'l> {
    pub live_pool_getter: Arc<dyn LivePoolGetter + 'l>,
    pub classfile_getter: Arc<dyn ClassFileGetter + 'l>,
    pub string_pool: &'l CompressedClassfileStringPool,
    pub current_class: CClassName,
    pub class_view_cache: Mutex<HashMap<ClassWithLoader, Arc<dyn ClassView>>>,
    pub current_loader: LoaderName,
    pub verification_types: HashMap<u16, HashMap<ByteCodeOffset, Frame>>,
    pub debug: bool,
    pub perf_metrics: &'l PerfMetrics,
    pub permissive_types_workaround: bool,
}

pub trait ClassFileGetter {
    fn get_classfile(&self, vf_context: &VerifierContext, loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError>;
}

pub struct NoopClassFileGetter;

impl ClassFileGetter for NoopClassFileGetter {
    fn get_classfile(&self, _vf_context: &VerifierContext, loader: LoaderName, class: CClassName) -> Result<Arc<dyn ClassView>, ClassLoadingError> {
        todo!("{:?}{:?}", loader, class)
    }
}

#[derive(Eq, Debug)]
pub struct OperandStack {
    pub data: VecDeque<VType>,
}

impl Clone for OperandStack {
    fn clone(&self) -> Self {
        OperandStack { data: self.data.clone() }
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