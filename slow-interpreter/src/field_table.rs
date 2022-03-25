use std::collections::HashMap;
use std::sync::Arc;

use by_address::ByAddress;
use rust_jvm_common::{FieldId, FieldTableIndex};

use crate::runtime_class::RuntimeClass;

pub struct FieldTable<'gc> {
    table: Vec<(Arc<RuntimeClass<'gc>>, u16)>,
    //todo at a later date will contain compiled code data etc.
    index: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, HashMap<u16, FieldTableIndex>>,
}

//todo duplication with MethodTable
impl<'gc> FieldTable<'gc> {
    pub fn get_field_id(&mut self, rc: Arc<RuntimeClass<'gc>>, index: u16) -> FieldTableIndex {
        match match self.index.get(&rc.clone().into()) {
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

    pub fn register_with_table(&mut self, rc: Arc<RuntimeClass<'gc>>, field_index: u16) -> FieldTableIndex {
        let res = self.table.len();
        self.table.push((rc.clone(), field_index));
        match self.index.get_mut(&rc.clone().into()) {
            None => {
                let mut class_methods = HashMap::new();
                class_methods.insert(field_index, res);
                self.index.insert(rc.clone().into(), class_methods);
            }
            Some(class_methods) => {
                class_methods.insert(field_index, res);
            }
        }
        res
    }

    pub fn lookup(&self, id: FieldId) -> (Arc<RuntimeClass<'gc>>, u16) {
        self.table[id].clone()
    }

    pub fn new() -> Self {
        Self { table: vec![], index: HashMap::new() }
    }
}