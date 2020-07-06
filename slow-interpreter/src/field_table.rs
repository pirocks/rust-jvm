use std::collections::HashMap;
use std::sync::Arc;

use crate::runtime_class::RuntimeClass;

pub type FieldTableIndex = usize;
pub type FieldId = usize;


pub struct FieldTable {
    table: Vec<(Arc<RuntimeClass>, u16)>,
    //todo at a later date will contain compiled code data etc.
    index: HashMap<Arc<RuntimeClass>, HashMap<u16, FieldTableIndex>>,
}

//todo duplication with MethodTable
impl FieldTable {
    pub fn get_field_id(&mut self, rc: Arc<RuntimeClass>, index: u16) -> FieldTableIndex {
        match match self.index.get(&rc) {
            Some(x) => x,
            None => {
                return self.register_with_table(rc, index);
            }
        }.get(&index) {
            Some(x) => *x,
            None => self.register_with_table(rc, index),
        }
    }

    pub fn register_with_table(&mut self, rc: Arc<RuntimeClass>, field_index: u16) -> FieldTableIndex {
        let res = self.table.len();
        self.table.push((rc.clone(), field_index));
        match self.index.get_mut(&rc) {
            None => {
                let mut class_methods = HashMap::new();
                class_methods.insert(field_index, res);
                self.index.insert(rc, class_methods);
            }
            Some(class_methods) => {
                class_methods.insert(field_index, res);
            }
        }
        res
    }

    pub fn lookup(&self, id: FieldId) -> (Arc<RuntimeClass>, u16) {
        self.table[id].clone()
    }

    pub fn new() -> Self {
        Self {
            table: vec![],
            index: HashMap::new(),
        }
    }
}
