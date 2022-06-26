use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use itertools::Itertools;

use class_ids::{ClassID};


#[cfg(test)]
pub mod test;

pub const FAST_INSTANCE_OF_TABLE_SIZE: u32 = 512;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(transparent)]
pub struct FastInstanceOfTableIndex(u32);

pub struct FastInstanceOfTableIndexEntry {
    inner: Arc<AtomicU32>
}

impl FastInstanceOfTableIndexEntry {
    pub fn new(inner: FastInstanceOfTableIndex)  -> Self{
        Self{
            inner: Arc::new(AtomicU32::new(inner.0))
        }
    }

    pub fn load(&self) -> FastInstanceOfTableIndex {
        FastInstanceOfTableIndex(self.inner.load(Ordering::SeqCst))
    }

    pub fn store(&mut self, val: FastInstanceOfTableIndex) {
        self.inner.store(val.0, Ordering::SeqCst);
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct FastInstanceOfTableMemberRaw {
    is_valid_and_is_instance_and_in_use: AtomicU8,
}

const IS_VALID_MASK: u8 = 0x1;
const IS_INSTANCE_MASK: u8 = 0x2;
const IS_IN_USE_MASK:u8 = 0x4;

impl FastInstanceOfTableMemberRaw {
    pub fn new(is_valid: bool, is_instance: bool) -> Self {
        let res = Self {
            is_valid_and_is_instance_and_in_use: AtomicU8::new(0)
        };
        res.store(FastInstanceOfTableMember {
            is_valid,
            is_instance,
        });
        res
    }

    pub fn load(&self) -> FastInstanceOfTableMember {
        let raw_val = self.is_valid_and_is_instance_and_in_use.load(Ordering::SeqCst);
        let is_valid = (raw_val & IS_VALID_MASK) != 0;
        let is_instance = (raw_val & IS_INSTANCE_MASK) != 0;
        FastInstanceOfTableMember {
            is_valid,
            is_instance,
        }
    }

    pub fn store(&self, to_store: FastInstanceOfTableMember) {
        let mut new_raw = 0;
        if to_store.is_valid {
            new_raw |= IS_VALID_MASK;
        }
        if to_store.is_instance {
            new_raw |= IS_INSTANCE_MASK;
        }
        self.is_valid_and_is_instance_and_in_use.store(new_raw, Ordering::SeqCst);
    }

    pub fn set_instance_of(&mut self){
        let FastInstanceOfTableMember{ is_valid, is_instance } = self.load();
        if !is_valid{
            return;
        }
        if is_instance{
            self.store(FastInstanceOfTableMember{ is_valid: false, is_instance: true });
        }else {
            self.store(FastInstanceOfTableMember{ is_valid: true, is_instance: true })
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct FastInstanceOfTableMember {
    is_valid: bool,
    is_instance: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct ObjectFastInstanceOfTable {
    table: [FastInstanceOfTableMemberRaw; FAST_INSTANCE_OF_TABLE_SIZE as usize],
}

pub struct ObjectFastInstanceOfTables {
    interfaces: HashMap<ClassID, FastInstanceOfTableIndexEntry>,
    tables: HashMap<ClassID, Arc<ObjectFastInstanceOfTable>>,
    current_index: u32,
}

impl ObjectFastInstanceOfTables {
    pub fn new() -> Self {
        Self {
            interfaces: HashMap::new(),
            tables: HashMap::new(),
            current_index: 0,
        }
    }

    fn next_index(&mut self) -> FastInstanceOfTableIndex {
        let res = self.current_index;
        self.current_index += 1;
        if res == FAST_INSTANCE_OF_TABLE_SIZE {
            self.current_index = 0;
            return FastInstanceOfTableIndex(0);
        }
        FastInstanceOfTableIndex(res)
    }

    fn setup_interface_numbers(&mut self, interfaces: &[ClassID]) {
        for interface in interfaces.iter().cloned() {
            let next_index = self.next_index();
            self.interfaces.entry(interface).or_insert(FastInstanceOfTableIndexEntry::new(next_index));
        }
    }

    pub fn add_class(&mut self, class: ClassID, interfaces: Vec<ClassID>) {
        assert!(!interfaces.contains(&class));
        self.setup_interface_numbers(interfaces.as_slice());
        let mut table = ObjectFastInstanceOfTable {
            table: (0..FAST_INSTANCE_OF_TABLE_SIZE).map(|_|FastInstanceOfTableMemberRaw::new(true,false)).collect_vec().try_into().unwrap()
        };
        for interface in interfaces {
            let index = self.interfaces.get(&interface).unwrap().load();
            table.table[index.0 as usize].set_instance_of();
        }
        self.tables.insert(class, Arc::new(table));
    }

    pub fn interface_instance_of_fast(&self, object_class: ClassID, interface: ClassID) -> Option<bool> {
        let interface_index = self.interfaces.get(&interface).unwrap().load();
        let table_elem = self.tables.get(&object_class).unwrap().table[interface_index.0 as usize].load();
        if table_elem.is_valid {
            Some(table_elem.is_instance)
        } else {
            None
        }
    }

    pub fn migrate_interface(&mut self, interface_class_id: ClassID) {
        let current_index = self.interfaces.get(&interface_class_id).unwrap().load();
        let next_class_index = loop {
            let next_class_index = self.next_index();
            if next_class_index != current_index{
                break next_class_index;
            }
        };
        //set all target interfaces to invalid
        //
        self.interfaces
    }
}
