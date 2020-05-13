use std::collections::HashMap;
use std::sync::Arc;
use crate::runtime_class::RuntimeClass;


type MethodTableIndex = usize;
pub type MethodId = MethodTableIndex;
pub struct MethodTable{
    table: Vec<(Arc<RuntimeClass>,u16)>,//todo at a later date will contain compiled code etc.
    index: HashMap<Arc<RuntimeClass>,HashMap<u16,MethodTableIndex>>
}

impl MethodTable{
    pub fn get_method_id(&mut self, rc: Arc<RuntimeClass>, index: u16) -> MethodTableIndex{
        match match self.index.get(&rc) {
            Some(x) => x,
            None => {
                return self.register_with_table(rc,index)
            },
        }.get(&index) {
            Some(x) => *x,
            None => self.register_with_table(rc,index),
        }
    }

    pub fn register_with_table(&mut self, rc: Arc<RuntimeClass>, method_index : u16) -> MethodTableIndex{
        let res = self.table.len();
        self.table.push((rc.clone(),method_index));
        match self.index.get_mut(&rc){
            None => {
                let mut class_methods = HashMap::new();
                class_methods.insert(method_index,res);
                self.index.insert(rc,class_methods);
            },
            Some(class_methods) => {
                class_methods.insert(method_index,res);
            },
        }
        res
    }

    pub fn lookup(&self, id: MethodId) ->(Arc<RuntimeClass>,u16) {
        self.table[id].clone()
    }

    pub fn new() -> Self{
        Self{
            table: vec![],
            index: HashMap::new()
        }
    }
}

