use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use by_address::ByAddress;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::MethodId;
use crate::ResolvedInterfaceVTableEntry;

pub struct InvokeInterfaceLookupCache<'gc> {
    inner: HashMap<(ByAddress<Arc<RuntimeClass<'gc>>>, MethodName, CMethodDescriptor), ResolvedInterfaceVTableEntry>,
    method_id_to_method: HashMap<MethodId, HashSet<(ByAddress<Arc<RuntimeClass<'gc>>>, MethodName, CMethodDescriptor)>>
}

impl <'gc> InvokeInterfaceLookupCache<'gc> {
    pub fn new() -> Self{
        Self{
            inner: Default::default(),
            method_id_to_method: Default::default()
        }
    }

    pub fn lookup(&self, rc: Arc<RuntimeClass<'gc>>, method_name: MethodName, desc: CMethodDescriptor) -> Option<ResolvedInterfaceVTableEntry>{
        self.inner.get(&(ByAddress(rc),method_name,desc)).cloned()
    }

    pub fn update(&mut self, method_id: MethodId, update_to: ResolvedInterfaceVTableEntry){
        if let Some(inner_key) = self.method_id_to_method.get(&method_id) {
            for inner_key in inner_key{
                *self.inner.get_mut(inner_key).unwrap() = update_to;
            }
        }
    }

    pub fn register_entry(&mut self, rc: Arc<RuntimeClass<'gc>>, method_name: MethodName, desc: CMethodDescriptor, resolved: ResolvedInterfaceVTableEntry){
        self.method_id_to_method.entry(resolved.method_id).or_default().insert((ByAddress(rc.clone()), method_name, desc.clone()));
        self.inner.insert((ByAddress(rc), method_name, desc), resolved);
    }

}
