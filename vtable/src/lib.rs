#![feature(vec_into_raw_parts)]
#![feature(box_syntax)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::NonNull;
use std::sync::Arc;
use by_address::ByAddress;
use iced_x86::code_asm::CodeAssembler;
use itertools::Itertools;
use memoffset::offset_of;
use another_jit_vm::Register;

use another_jit_vm_ir::IRMethodID;
use runtime_class_stuff::{MethodNumber, RuntimeClass, RuntimeClassClass};
use rust_jvm_common::MethodId;

#[repr(C)]
pub struct VTableEntry {
    address: Option<NonNull<c_void>>, //null indicates need for resolve
    ir_method_id: IRMethodID,
    method_id: MethodId,
    new_frame_size: usize,
}

impl VTableEntry {
    pub fn unresolved() -> Self{
        VTableEntry{
            address: None,
            ir_method_id: IRMethodID(0),
            method_id: 0,
            new_frame_size: 0
        }
    }
}

#[repr(C)]
pub struct RawNativeVTable {
    ptr: *mut VTableEntry,
    capacity: usize,
    len: usize,
}

impl RawNativeVTable{
    pub fn new<'gc>(rc: &RuntimeClassClass<'gc>) -> Self{
        let vec = (0..rc.recursive_num_methods).map(|_|VTableEntry{
            address: None,
            ir_method_id: IRMethodID(0),
            method_id: 0,
            new_frame_size: 0
        }).collect_vec();
        let (ptr, len, capacity) = Vec::into_raw_parts(vec);
        Self{
            ptr,
            capacity,
            len
        }
    }
}

pub struct VTable {
    vtable: Vec<VTableEntry>,
}

impl VTable {
    fn update_vtable(raw_native_vtable: &mut RawNativeVTable, updater: impl FnOnce(VTable) -> VTable) {
        let RawNativeVTable { capacity, len, ptr } = *raw_native_vtable;
        let vtable = VTable { vtable: unsafe { Vec::from_raw_parts(ptr, len, capacity) } };
        let res = updater(vtable);
        let (ptr, len, capacity) = res.vtable.into_raw_parts();
        *raw_native_vtable = RawNativeVTable{
            capacity,
            len,
            ptr
        };
    }

    fn set_entry(&mut self, entry_number: MethodNumber, entry: VTableEntry) {
        *self.vtable.get_mut(entry_number.0 as usize).unwrap() = entry;
    }
}

pub struct VTables<'gc>{
    inner: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>,&'gc mut RawNativeVTable>//ref is leaked box
}

impl<'gc> VTables<'gc> {
    pub fn vtable_register_entry(&mut self, rc: Arc<RuntimeClass<'gc>>, method_number: MethodNumber, entry: VTableEntry){
        let raw_native_table = self.inner.entry(ByAddress(rc.clone())).or_insert(Box::leak(box RawNativeVTable::new(rc.unwrap_class_class())));
        VTable::update_vtable(raw_native_table, move |mut vtable|{
            vtable.set_entry(method_number, entry);
            vtable
        })
    }
}


pub fn generate_vtable_access(
    assembler: &mut CodeAssembler,
    method_number: MethodNumber,
    raw_native_vtable_address: Register,
    temp_1: Register,
    address: Register,
    ir_method_id: Register,
    method_id: Register,
    new_frame_size: Register
) {
    let ptr_val = temp_1;
    assembler.mov(ptr_val.to_native_64(), raw_native_vtable_address.to_native_64() + offset_of!(RawNativeVTable,ptr)).unwrap();
    let vtable_entry = temp_1;
    assembler.lea(vtable_entry.to_native_64(), ptr_val.to_native_64() + method_number.0 as usize *size_of::<VTableEntry>()).unwrap();
    assembler.mov(address.to_native_64(), vtable_entry.to_native_64() + offset_of!(VTableEntry,address)).unwrap();
    assembler.mov(ir_method_id.to_native_64(), vtable_entry.to_native_64() + offset_of!(VTableEntry,ir_method_id)).unwrap();
    assembler.mov(method_id.to_native_64(), vtable_entry.to_native_64() + offset_of!(VTableEntry,method_id)).unwrap();
    assembler.mov(new_frame_size.to_native_64(), vtable_entry.to_native_64() + offset_of!(VTableEntry,new_frame_size)).unwrap();
}