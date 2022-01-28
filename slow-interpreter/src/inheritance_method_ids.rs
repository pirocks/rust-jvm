use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;
use iced_x86::CC_b::c;

use iced_x86::OpCodeOperandKind::cl;
use itertools::Itertools;

use rust_jvm_common::{InheritanceMethodID, MethodI, MethodId};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedMethodDescriptor};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::LoaderName;

use crate::JVMState;
use crate::runtime_class::RuntimeClass;

pub struct InheritanceMethodIDs {
    //todo need loader here?
    ids: HashMap<(CClassName, LoaderName), HashMap<(MethodName, CMethodDescriptor), InheritanceMethodID>>,
}

impl InheritanceMethodIDs {
    pub fn new() -> Self {
        Self {
            ids: Default::default(),
        }
    }

    pub fn register_impl(&mut self,  rc: &Arc<RuntimeClass<'gc_life>>) -> HashMap<(MethodName, CMethodDescriptor), InheritanceMethodID> {
        return match rc.deref() {
            RuntimeClass::Object(class_class) => {
                dbg!(class_class.class_view.name().unwrap_name());
                match &class_class.parent {
                    None => {
                        let object_methods = self.ids.entry((CClassName::object(), LoaderName::BootstrapLoader)).or_default();
                        assert_eq!(class_class.class_view.name().unwrap_name(), CClassName::object());
                        for (i,method) in class_class.class_view.methods().enumerate(){
                            let method_name = method.name();
                            let desc = method.desc().clone();
                            object_methods.insert((method_name,desc),InheritanceMethodID(i as u64));
                        }
                        object_methods.clone()
                    }
                    Some(parent_class) => {
                        let already_registered_methods = self.register_impl(parent_class);
                        let mut this_class_ids = HashMap::new();
                        for method in class_class.class_view.methods() {
                            let mut next_inheritance_id = InheritanceMethodID(already_registered_methods.len() as u64 + this_class_ids.len() as u64);
                            let method_name = method.name();
                            let c_method_descriptor = method.desc().clone();
                            let inheritance_id = *match already_registered_methods.get(&(method_name, c_method_descriptor.clone())) {
                                Some(x) => x,
                                None => {
                                    &next_inheritance_id
                                },
                            };
                            this_class_ids.insert((method_name, c_method_descriptor), inheritance_id);
                        }
                        self.ids.insert((class_class.class_view.name().unwrap_name(), LoaderName::BootstrapLoader), this_class_ids.clone());//todo loader nonsense
                        this_class_ids
                    }
                }
            }
            _ => HashMap::new(),
        };
    }

    pub fn register(&mut self, jvm: &'gc_life JVMState<'gc_life>, rc: &Arc<RuntimeClass<'gc_life>>) {
        self.register_impl(rc);
    }

    pub fn lookup(&self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId) -> InheritanceMethodID {
        dbg!(jvm.method_table.read().unwrap().lookup_method_string(method_id, &jvm.string_pool));
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let class_view = rc.view();
        let name = class_view.name().unwrap_name();
        let method_view = class_view.method_view_i(method_i);
        let method_name = method_view.name();
        let method_desc = method_view.desc().clone();
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
        *dbg!(self.ids.get(&(name, loader)).unwrap()).get(dbg!(&(method_name, method_desc))).unwrap()
    }
}