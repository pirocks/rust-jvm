use std::collections::HashMap;
use std::sync::Arc;

use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::MethodId;

use crate::JVMState;
use crate::runtime_class::RuntimeClass;

pub struct InheritanceMethodIDs {
    //todo need loader here?
    ids: HashMap<(CClassName, MethodI, LoaderName), InheritanceMethodID>,
}

impl InheritanceMethodIDs {
    pub fn new() -> Self {
        Self {
            ids: Default::default(),
        }
    }

    pub fn register(&mut self, jvm: &'gc_life JVMState<'gc_life>, rc: Arc<RuntimeClass<'gc_life>>) {
        todo!()
    }

    pub fn lookup(&self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId) -> InheritanceMethodID {
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let name = rc.view().name().unwrap_name();
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
        *self.ids.get(&(name, method_i, loader)).unwrap()
    }
}