use std::collections::HashMap;
use std::sync::Arc;
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::MethodId;
use crate::JVMState;

use crate::runtime_class::RuntimeClass;

pub struct InheritanceMethodID(u64);

pub type MethodI = u16;

pub struct InheritanceMethodIDs<'gc_life> {
    //todo need loader here?
    ids: HashMap<(CClassName, MethodI, LoaderName), InheritanceMethodID>,
}

impl<'gc_life> InheritanceMethodIDs<'gc_life> {
    pub fn new() -> Self{
        Self{
            ids: Default::default(),
        }
    }

    pub fn register(&mut self, jvm: &'gc_life JVMState<'gc_life>, rc: Arc<RuntimeClass<'gc_life>>){
        todo!()
    }

    pub fn lookup(&self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId){
        todo!()
    }
}