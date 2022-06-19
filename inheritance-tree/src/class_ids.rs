use std::collections::HashMap;
use std::sync::Mutex;
use rust_jvm_common::compressed_classfile::CPDType;
use crate::ClassID;

pub struct ClassIdsInner{
    classes: Vec<CPDType>,
    mapping: HashMap<CPDType, ClassID>
}

pub struct ClassIDs {
    inner: Mutex<ClassIdsInner>
}

impl ClassIDs {
    pub fn new() -> Self{
        Self{
            inner: Mutex::new(ClassIdsInner { classes: vec![], mapping: HashMap::new() })
        }
    }

    pub fn get_id_or_add(&self, cpdtype: CPDType) -> ClassID{
        if let Some(inner) = self.inner.lock().unwrap().mapping.get(&cpdtype) {
            return *inner;
        }
        let mut inner = self.inner.lock().unwrap();
        let next_id = ClassID(inner.classes.len() as u32);
        inner.classes.push(cpdtype);
        inner.mapping.insert(cpdtype,next_id);
        next_id
    }

    pub fn lookup(&self, id: ClassID) -> CPDType{
        *self.inner.lock().unwrap().classes.get(id.0 as usize).unwrap()
    }
}

