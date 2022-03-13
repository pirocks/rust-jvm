use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

use iced_x86::CC_b::c;
use iced_x86::ConditionCode::o;
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
    current_id: InheritanceMethodID,
}

impl InheritanceMethodIDs {
    pub fn new() -> Self {
        Self {
            ids: Default::default(),
            current_id: InheritanceMethodID(0),
        }
    }

    fn new_id(&mut self) -> InheritanceMethodID {
        let res = self.current_id;
        self.current_id.0 += 1;
        res
    }

    pub fn register_impl<'gc_life>(&mut self, rc: &Arc<RuntimeClass<'gc_life>>) -> HashMap<(MethodName, CMethodDescriptor), InheritanceMethodID> {
        return match rc.deref() {
            RuntimeClass::Object(class_class) => {
                match &class_class.parent {
                    None => {
                        let object_key = (CClassName::object(), LoaderName::BootstrapLoader);
                        match self.ids.get(&object_key) {
                            Some(object_methods) => {
                                return object_methods.clone();
                            }
                            None => {
                                let mut object_methods = HashMap::new();
                                assert_eq!(class_class.class_view.name().unwrap_name(), CClassName::object());
                                for method in class_class.class_view.virtual_methods() {
                                    let method_name = method.name();
                                    let desc = method.desc().clone();
                                    object_methods.insert((method_name, desc), self.new_id());
                                }
                                let res = object_methods.clone();
                                self.ids.insert(object_key, object_methods);
                                res
                            }
                        }
                    }
                    Some(parent_class) => {
                        let this_rc_key = (class_class.class_view.name().unwrap_name(), LoaderName::BootstrapLoader);//todo loader nonsense
                        let already_registered_methods = self.register_impl(parent_class);
                        let this_method_ids = if let Some(method_ids) = self.ids.get(&this_rc_key) {
                            method_ids.clone()
                        } else {
                            let mut this_class_method_ids = HashMap::new();
                            for method in class_class.class_view.virtual_methods() {
                                let method_name = method.name();
                                let c_method_descriptor = method.desc().clone();
                                let inheritance_id = already_registered_methods.get(&(method_name, c_method_descriptor.clone())).cloned().unwrap_or_else(|| self.new_id());
                                this_class_method_ids.insert((method_name, c_method_descriptor), inheritance_id);
                            }
                            let overwritten = self.ids.insert(this_rc_key, this_class_method_ids.clone());
                            assert!(overwritten.is_none());
                            this_class_method_ids
                        };
                        let mut all = HashMap::new();
                        all.extend(this_method_ids.into_iter());
                        all.extend(already_registered_methods.into_iter());
                        all
                    }
                }
            }
            _ => HashMap::new(),
        };
    }

    pub fn register<'gc_life>(&mut self, jvm: &'gc_life JVMState<'gc_life>, rc: &Arc<RuntimeClass<'gc_life>>) {
        let _ = self.register_impl(rc);
    }


    pub fn integrity_assert(&self) {
        let method_names_and_desc_to_id = todo!();
        for ((current_class_name, current_loader), names_to_ids) in self.ids.iter() {
            for ((current_method_name, current_method_desc), current_inheritance_id) in names_to_ids.iter() {
                todo!()
            }
        }
    }

    pub fn lookup<'gc_life>(&self, jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId) -> InheritanceMethodID {
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let class_view = rc.view();
        let name = class_view.name().unwrap_name();
        let method_view = class_view.method_view_i(method_i);
        let method_name = method_view.name();
        let method_desc = method_view.desc();
        let loader = jvm.classes.read().unwrap().get_initiating_loader(&rc);
        *self.ids.get(&(name, loader)).unwrap().get(&(method_name, method_desc.clone())).unwrap()
    }
}