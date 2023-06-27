#![feature(vec_into_raw_parts)]
#![feature(portable_simd)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{forget, size_of};
use std::ptr::NonNull;
use std::sync::Arc;

use by_address::ByAddress;
use iced_x86::code_asm::CodeAssembler;
use itertools::Itertools;
use memoffset::offset_of;

use another_jit_vm::Register;
use runtime_class_stuff::{RuntimeClass, RuntimeClassClass};
use runtime_class_stuff::method_numbers::MethodNumber;

pub mod lookup_cache;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct VTableEntry {
    pub address: Option<NonNull<c_void>>,
}

impl VTableEntry {
    pub fn unresolved() -> Self {
        VTableEntry {
            address: None,
        }
    }

    pub fn resolved(&self) -> Option<ResolvedVTableEntry> {
        Some(ResolvedVTableEntry {
            address: self.address?,
        })
    }
}


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ResolvedVTableEntry {
    pub address: NonNull<c_void>,
}

#[repr(C)]
pub struct RawNativeVTable {
    ptr: *mut VTableEntry,
    capacity: usize,
    len: usize,
}

impl RawNativeVTable {
    pub fn new(rc: &RuntimeClassClass) -> Self {
        let vec = (0..rc.recursive_num_methods).map(|_| VTableEntry {
            address: None,
        }).collect_vec();
        let (ptr, len, capacity) = Vec::into_raw_parts(vec);
        Self {
            ptr,
            capacity,
            len,
        }
    }
}

pub struct VTable {
    vtable: Vec<VTableEntry>,
}

impl VTable {
    pub fn lookup(raw_native_vtable: NonNull<RawNativeVTable>, method_number: MethodNumber) -> Option<ResolvedVTableEntry> {
        let raw_native_vtable = unsafe { raw_native_vtable.as_ref() };
        let RawNativeVTable { capacity, len, ptr } = *raw_native_vtable;
        let vtable = VTable { vtable: unsafe { Vec::from_raw_parts(ptr, len, capacity) } };
        let res = vtable.vtable[method_number.0 as usize].clone();
        forget(vtable);
        res.resolved()
    }

    fn update_vtable(mut raw_native_vtable: NonNull<RawNativeVTable>, updater: impl FnOnce(VTable) -> VTable) {
        let raw_native_vtable = unsafe { raw_native_vtable.as_mut() };
        let RawNativeVTable { capacity, len, ptr } = *raw_native_vtable;
        let vtable = VTable { vtable: unsafe { Vec::from_raw_parts(ptr, len, capacity) } };
        let res = updater(vtable);
        let (ptr, len, capacity) = res.vtable.into_raw_parts();
        *raw_native_vtable = RawNativeVTable {
            capacity,
            len,
            ptr,
        };
    }

    fn set_entry(&mut self, entry_number: MethodNumber, entry: VTableEntry) {
        *self.vtable.get_mut(entry_number.0 as usize).unwrap() = entry;
    }
}

pub fn generate_vtable_access(
    assembler: &mut CodeAssembler,
    method_number: MethodNumber,
    raw_native_vtable_address: Register,
    temp_1: Register,
    address: Register,
) {
    let ptr_val = temp_1;
    assembler.mov(ptr_val.to_native_64(), raw_native_vtable_address.to_native_64() + offset_of!(RawNativeVTable,ptr)).unwrap();
    let vtable_entry = temp_1;
    assembler.lea(vtable_entry.to_native_64(), ptr_val.to_native_64() + method_number.0 as usize * size_of::<VTableEntry>()).unwrap();
    assembler.mov(address.to_native_64(), vtable_entry.to_native_64() + offset_of!(VTableEntry,address)).unwrap();
}

pub struct VTables<'gc> {
    inner: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, NonNull<RawNativeVTable>>,
    resolved_to_entry: HashMap<NonNull<c_void>, Vec<(Arc<RuntimeClass<'gc>>, MethodNumber)>>,
}

static mut VTABLE_ALLOCS: u64 = 0;


impl<'gc> VTables<'gc> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            resolved_to_entry: Default::default(),
        }
    }

    pub fn lookup_or_new_vtable(&mut self, rc: Arc<RuntimeClass<'gc>>) -> NonNull<RawNativeVTable> {
        *self.inner.entry(ByAddress(rc.clone())).or_insert_with(|| {
            unsafe {
                VTABLE_ALLOCS += 1;
                // if VTABLE_ALLOCS % 1_000 == 0 {
                //     dbg!(VTABLE_ALLOCS);
                // }
            }
            NonNull::new(Box::into_raw(Box::new(RawNativeVTable::new(rc.unwrap_class_class())))).unwrap()
        }
        )
    }

    pub fn update_address(&mut self, from: NonNull<c_void>, to: NonNull<c_void>) {
        if let Some(addresses) = self.resolved_to_entry.remove(&from) {
            for (rc, method_number) in addresses {
                self.vtable_register_entry(rc, method_number, VTableEntry { address: Some(to) });
            }
        }
    }

    pub fn vtable_register_entry(&mut self, rc: Arc<RuntimeClass<'gc>>, method_number: MethodNumber, entry: VTableEntry) -> NonNull<RawNativeVTable> {
        self.resolved_to_entry.entry(entry.address.unwrap()).or_default().push((rc.clone(), method_number));
        let raw_native_table = self.lookup_or_new_vtable(rc);
        VTable::update_vtable(raw_native_table, move |mut vtable| {
            vtable.set_entry(method_number, entry);
            vtable
        });
        raw_native_table
    }
}
