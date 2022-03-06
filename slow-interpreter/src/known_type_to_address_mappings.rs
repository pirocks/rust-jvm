use std::collections::HashMap;
use std::sync::RwLock;
use libc::c_void;
use gc_memory_layout_common::BaseAddressAndMask;
use rust_jvm_common::compressed_classfile::CPDType;



pub struct KnownAddresses{
    inner: RwLock<HashMap<CPDType, Vec<BaseAddressAndMask>>>
}

impl KnownAddresses {
    pub fn new() -> Self{
        Self{
            inner: RwLock::new(HashMap::new())
        }
    }

    pub fn known_addresses_for_type(&self, cpdtype: &CPDType) -> Vec<BaseAddressAndMask> {
        match self.inner.read().unwrap().get(cpdtype) {
            Some(x) => x,
            None => return vec![],
        }.clone()
    }

    pub fn sink_known_address(&self, cpdtype: CPDType, known_address: BaseAddressAndMask) {
        self.inner.write().unwrap().entry(cpdtype).or_insert(vec![]).push(known_address);
    }
}
