use std::collections::HashMap;
use std::ptr::NonNull;

use crate::{BitPath256};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct BitVecPathID(u64);

pub struct BitVecPaths {
    vals: Vec<NonNull<BitPath256>>,
    //owned box leak
    mapping: HashMap<BitPath256, BitVecPathID>,
}

impl BitVecPaths {
    pub fn new() -> Self {
        Self {
            vals: vec![],
            mapping: HashMap::new(),
        }
    }

    pub fn lookup_or_add(&mut self, bit_path: BitPath256) -> BitVecPathID {
        if let Some(bit_path_id) = self.mapping.get(&bit_path) {
            return *bit_path_id;
        }
        let new_id = BitVecPathID(self.vals.len() as u64);
        self.vals.push(NonNull::new(Box::into_raw(Box::new(bit_path.clone()))).unwrap());
        self.mapping.insert(bit_path, new_id);
        new_id
    }

    pub fn get_ptr_from_id(&self, id: BitVecPathID) -> NonNull<BitPath256>{
        self.vals[id.0 as usize]
    }
}
