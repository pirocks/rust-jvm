
use std::collections::HashMap;
use std::sync::Arc;

use by_address::ByAddress;

use jvmti_jni_bindings::_jmethodID;
use rust_jvm_common::{MethodId, MethodTableIndex};
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;

use runtime_class_stuff::RuntimeClass;

pub fn from_jmethod_id(jmethod: *mut _jmethodID) -> MethodId {
    jmethod as MethodId
}

pub struct MethodTable<'gc> {
    table: Vec<(Arc<RuntimeClass<'gc>>, u16)>,
    //at a later date will contain compiled code etc.
    index: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, HashMap<u16, MethodTableIndex>>,
}

impl<'gc> MethodTable<'gc> {
    pub fn get_method_id(&mut self, rc: Arc<RuntimeClass<'gc>>, index: u16) -> MethodTableIndex {
        assert_ne!(index, u16::max_value());
        match match self.index.get(&ByAddress(rc.clone())) {
            Some(x) => x,
            None => {
                return self.register_with_table(rc, index);
            }
        }
            .get(&index)
        {
            Some(x) => *x,
            None => self.register_with_table(rc, index),
        }
    }

    pub fn register_with_table(&mut self, rc: Arc<RuntimeClass<'gc>>, method_index: u16) -> MethodTableIndex {
        assert_ne!(method_index, u16::max_value());
        let res = self.table.len();
        self.table.push((rc.clone(), method_index));
        match self.index.get_mut(&ByAddress(rc.clone())) {
            None => {
                let mut class_methods = HashMap::new();
                class_methods.insert(method_index, res);
                self.index.insert(ByAddress(rc), class_methods);
            }
            Some(class_methods) => {
                class_methods.insert(method_index, res);
            }
        }
        res
    }

    pub fn try_lookup(&self, id: MethodId) -> Option<(Arc<RuntimeClass<'gc>>, u16)> {
        if id < self.table.len() {
            self.table[id].clone().into()
        } else {
            None
        }
    }

    pub fn lookup_method_string(&self, method_id: MethodId, string_pool: &CompressedClassfileStringPool) -> String {
        let (rc, method_i) = self.try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let method_name = method_view.name().0.to_str(string_pool);
        let method_desc = method_view.desc_str().to_str(string_pool);
        let class_name = view.name().unwrap_name().0.to_str(string_pool);
        format!("{}/{}/{}", class_name, method_name, method_desc)
    }

    pub fn lookup_method_string_no_desc(&self, method_id: MethodId, string_pool: &CompressedClassfileStringPool) -> String {
        let (rc, method_i) = self.try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let method_name = method_view.name().0.to_str(string_pool);
        let class_name = view.name().unwrap_name().0.to_str(string_pool);
        format!("{}/{}", class_name, method_name)
    }

    pub fn new() -> Self {
        Self { table: vec![], index: HashMap::new() }
    }
}