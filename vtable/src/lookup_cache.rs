use std::collections::HashMap;
use std::sync::Arc;
use by_address::ByAddress;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CompressedMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::MethodName;

pub struct InvokeVirtualLookupCache<'gc> {
    inner: HashMap<(ByAddress<Arc<RuntimeClass<'gc>>>, MethodName, CompressedMethodDescriptor), (Arc<RuntimeClass<'gc>>, u16)>
}

impl<'gc> InvokeVirtualLookupCache<'gc> {
    pub fn new() -> Self{
        Self{
            inner: HashMap::new()
        }
    }

    pub fn add_entry(&mut self, rc: Arc<RuntimeClass<'gc>>, name: MethodName, desc: CompressedMethodDescriptor, res: (Arc<RuntimeClass<'gc>>, u16)) {
        self.inner.insert((ByAddress(rc), name, desc), res);
    }

    pub fn lookup(&self, rc: Arc<RuntimeClass<'gc>>, name: MethodName, desc: CompressedMethodDescriptor) -> Option<(Arc<RuntimeClass<'gc>>, u16)>{
        self.inner.get(&(ByAddress(rc), name, desc)).cloned()
    }
}