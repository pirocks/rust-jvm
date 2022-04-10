use std::collections::HashMap;
use std::sync::Arc;
use by_address::ByAddress;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;
use crate::ResolvedInterfaceVTableEntry;

pub struct InvokeInterfaceLookupCache<'gc> {
    inner: HashMap<(ByAddress<Arc<RuntimeClass<'gc>>>, MethodName, CMethodDescriptor), ResolvedInterfaceVTableEntry>,
}

impl <'gc> InvokeInterfaceLookupCache<'gc> {
    pub fn new() -> Self{
        Self{
            inner: Default::default()
        }
    }

    pub fn lookup(&self, rc: Arc<RuntimeClass<'gc>>, method_name: MethodName, desc: CMethodDescriptor) -> Option<ResolvedInterfaceVTableEntry>{
        self.inner.get(&(ByAddress(rc),method_name,desc)).cloned()
    }

    pub fn register_entry(&mut self, rc: Arc<RuntimeClass<'gc>>, method_name: MethodName, desc: CMethodDescriptor, resolved: ResolvedInterfaceVTableEntry){
        self.inner.insert((ByAddress(rc), method_name, desc), resolved);
    }

}
