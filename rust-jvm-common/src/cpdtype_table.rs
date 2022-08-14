use std::collections::HashMap;


use crate::compressed_classfile::CPDType;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct CPDTypeID(pub u32);

//todo duplication with other tables
pub struct CPDTypeTable {
    table: Vec<CPDType>,
    index: HashMap<CPDType, CPDTypeID>,
}


impl CPDTypeTable {
    pub fn get_cpdtype_id(&mut self, cpdtype: CPDType) -> CPDTypeID {
        assert_eq!(self.table.len(), self.index.len());
        match self.index.get(&cpdtype) {
            None => {
                let new_id = self.table.len();
                self.table.push(cpdtype);
                self.index.insert(cpdtype, CPDTypeID(new_id as u32));
                CPDTypeID(new_id as u32)
            }
            Some(cpdtype_id) => {
                *cpdtype_id
            }
        }
    }

    pub fn get_cpdtype(&self, id: CPDTypeID) -> &CPDType {
        &self.table[id.0 as usize]
    }

    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { table: vec![], index: HashMap::new() }
    }
}

