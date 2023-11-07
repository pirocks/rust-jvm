#![feature(vec_into_raw_parts)]

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::{NonNull, slice_from_raw_parts, slice_from_raw_parts_mut};
use std::sync::Arc;

use by_address::ByAddress;
use iced_x86::code_asm::CodeAssembler;
use itertools::Itertools;
use memoffset::offset_of;

use another_jit_vm::{IRMethodID, Register};
use method_table::interface_table::{InterfaceID, InterfaceTable};
use runtime_class_stuff::RuntimeClass;
use runtime_class_stuff::method_numbers::MethodNumber;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDTypeOrderWrapper;


use rust_jvm_common::MethodId;

pub mod lookup_cache;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ResolvedInterfaceVTableEntry {
    pub address: NonNull<c_void>,
    pub ir_method_id: IRMethodID,
    pub method_id: MethodId,
    pub new_frame_size: usize,
}

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InterfaceVTableEntry {
    pub address: Option<NonNull<c_void>>,
}

impl InterfaceVTableEntry {
    pub fn unresolved() -> Self {
        Self {
            address: None,
        }
    }
}

impl From<NonNull<c_void>> for InterfaceVTableEntry {
    fn from(ptr: NonNull<c_void>) -> Self {
        Self {
            address: Some(ptr)
        }
    }
}


#[repr(C)]
pub struct ITableEntryRaw {
    interface_id: InterfaceID,
    vtable_ptr: *mut InterfaceVTableEntry,
    vtable_len: usize,
    vtable_capacity: usize,
}

pub enum ITableEntry {
    Owned {
        interface_id: InterfaceID,
        vtable: Vec<InterfaceVTableEntry>,
    },
    Ref {
        interface_id: InterfaceID,
        vtable: &'static mut [InterfaceVTableEntry],
    },
}

impl ITableEntry {
    pub fn vtable(&mut self) -> &mut [InterfaceVTableEntry] {
        match self {
            ITableEntry::Owned { vtable, .. } => vtable.as_mut_slice(),
            ITableEntry::Ref { vtable, .. } => vtable
        }
    }

    pub fn to_raw(self) -> ITableEntryRaw {
        let ITableEntry::Owned {
            interface_id, vtable
        } = self else { todo!() };
        let (vtable_ptr, vtable_len, vtable_capacity) = vtable.into_raw_parts();
        ITableEntryRaw {
            interface_id,
            vtable_ptr,
            vtable_len,
            vtable_capacity,
        }
    }

    pub fn from_raw(raw: &ITableEntryRaw) -> ITableEntry {
        unsafe {
            ITableEntry::Ref {
                interface_id: raw.interface_id,
                vtable: &mut *slice_from_raw_parts_mut(raw.vtable_ptr, raw.vtable_len),
            }
        }
    }
}

#[repr(C)]
pub struct ITableRaw {
    itable_ptr: *mut ITableEntryRaw,
    len: usize,
    capacity: usize,
}

pub enum ITable {
    Owned {
        itable: Vec<ITableEntryRaw>,
    },
    Ref {
        itable: &'static [ITableEntryRaw],
    },
}

impl ITable {
    pub fn itable(&self) -> &[ITableEntryRaw] {
        match self {
            ITable::Owned { itable } => itable.as_slice(),
            ITable::Ref { itable } => itable
        }
    }

    pub fn from_raw(raw: &ITableRaw) -> ITable {
        unsafe {
            ITable::Ref {
                itable: &*slice_from_raw_parts(raw.itable_ptr, raw.len)/*Vec::from_raw_parts(raw.itable_ptr, raw.len, raw.capacity)*/
            }
        }
    }

    pub fn to_raw(self) -> ITableRaw {
        let Self::Owned { itable } = self else { todo!() };
        let (itable_ptr, len, capacity) = itable.into_raw_parts();
        ITableRaw {
            itable_ptr,
            len,
            capacity,
        }
    }

    pub fn new<'gc>(interface_table: &InterfaceTable<'gc>, interfaces: &[Arc<RuntimeClass<'gc>>]) -> Self {
        Self::Owned {
            itable: interfaces
                .iter()
                .sorted_by_key(|interface| CPDTypeOrderWrapper(interface.unwrap_class_class().class_view.name().to_cpdtype()))
                .map(|interface| {
                    let interface_id = interface_table.get_interface_id((*interface).clone());
                    let vtable = (0..interface.unwrap_class_class().recursive_num_methods).map(|_| InterfaceVTableEntry::unresolved()).collect_vec();
                    ITableEntry::Owned { interface_id, vtable }.to_raw()
                })
                .collect_vec()
        }
    }

    pub fn lookup<'gc>(table: NonNull<ITableRaw>, interface_id: InterfaceID, interface_method_number: MethodNumber) -> Option<InterfaceVTableEntry> {
        unsafe {
            lookup_unsafe(table, interface_id, interface_method_number)
                .map(|address| InterfaceVTableEntry { address: Some(address) })
        }
    }

    fn set_entry<'gc>(table: NonNull<ITableRaw>, interface_id: InterfaceID, interface_method_number: MethodNumber, resolved: InterfaceVTableEntry) {
        unsafe { write_resolved_unsafe(table, interface_id, interface_method_number, resolved.address.unwrap()) }
    }
}

pub unsafe fn write_resolved_unsafe(mut itable: NonNull<ITableRaw>, interface_id: InterfaceID, interface_method_number: MethodNumber, res: NonNull<c_void>) {
    let itable = ITable::from_raw(itable.as_mut());
    let table: &ITableEntryRaw = itable.itable().iter().find(|table| table.interface_id == interface_id).unwrap();
    ITableEntry::from_raw(table).vtable().get_mut(interface_method_number.0 as usize).unwrap().address = Some(res);
}

pub unsafe fn lookup_unsafe(mut itable: NonNull<ITableRaw>, interface_id: InterfaceID, interface_method_number: MethodNumber) -> Option<NonNull<c_void>> {
    let itable = ITable::from_raw(itable.as_mut());
    let table: &ITableEntryRaw = itable.itable().iter().find(|table| table.interface_id == interface_id).unwrap();
    ITableEntry::from_raw(table).vtable().get_mut(interface_method_number.0 as usize).unwrap().address
}

pub struct ITables<'gc> {
    inner: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, NonNull<ITableRaw>>,
    resolved_to_entry: HashMap<NonNull<c_void>, HashSet<(ByAddress<Arc<RuntimeClass<'gc>>>, InterfaceID, MethodNumber)>>,
}

static mut ITABLE_ALLOCS: usize = 0;

impl<'gc> ITables<'gc> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            resolved_to_entry: Default::default(),
        }
    }

    fn all_interfaces_impl(rc: &Arc<RuntimeClass<'gc>>, hashset: &mut HashSet<ByAddress<Arc<RuntimeClass<'gc>>>>, array_interfaces_only: bool) {
        if rc.view().is_interface() {
            if array_interfaces_only {
                if rc.cpdtype().is_primitive() {
                    //already handled on array case
                } else {
                    let name = rc.view().name();
                    if name == CClassName::serializable().into() && name == CClassName::cloneable().into() {
                        hashset.insert(ByAddress(rc.clone()));
                    }
                }
            } else {
                hashset.insert(ByAddress(rc.clone()));
            }
        }

        match rc.deref() {
            RuntimeClass::Array(arr) => {
                if arr.sub_class.cpdtype().is_primitive() {
                    hashset.insert(ByAddress(arr.serializable.clone()));
                    hashset.insert(ByAddress(arr.cloneable.clone()));
                } else {
                    Self::all_interfaces_impl(&arr.sub_class, hashset, true);
                }
            }
            RuntimeClass::Object(obj) => {
                if let Some(parent) = obj.parent.as_ref() {
                    Self::all_interfaces_impl(parent, hashset, array_interfaces_only);
                }
                for interface in obj.interfaces.iter() {
                    Self::all_interfaces_impl(interface, hashset, array_interfaces_only)
                }
            }
            _ => {
                todo!()
            }
        }
    }

    fn all_interfaces(rc: &Arc<RuntimeClass<'gc>>) -> Vec<Arc<RuntimeClass<'gc>>> {
        let mut res = HashSet::new();
        Self::all_interfaces_impl(rc, &mut res, false);
        res.into_iter().map(|rc| rc.0).collect_vec()
    }

    pub fn set_entry(&mut self, rc: Arc<RuntimeClass<'gc>>, interface_id: InterfaceID, interface_method_number: MethodNumber, ptr: NonNull<c_void>) {
        let by_address = ByAddress(rc);
        let itable: NonNull<ITableRaw> = *self.inner.get(&by_address).unwrap();
        ITable::set_entry(itable, interface_id, interface_method_number, InterfaceVTableEntry { address: Some(ptr) });
        self.resolved_to_entry.entry(ptr).or_default().insert((by_address, interface_id, interface_method_number));
    }

    pub fn lookup_or_new_itable(&mut self, interface_table: &InterfaceTable<'gc>, rc: Arc<RuntimeClass<'gc>>) -> NonNull<ITableRaw> {
        *self.inner.entry(ByAddress(rc.clone())).or_insert_with(|| {
            unsafe {
                ITABLE_ALLOCS += 1;
                if ITABLE_ALLOCS % 1_000 == 0 {
                    dbg!(ITABLE_ALLOCS);
                }
            }
            let interfaces = Self::all_interfaces(&rc);
            let table = ITable::new(interface_table, interfaces.as_slice()).to_raw();
            NonNull::new(Box::into_raw(Box::new(table))).unwrap()
        }
        )
    }

    pub fn update(&mut self, past_address: InterfaceVTableEntry, new_address: InterfaceVTableEntry) {
        if let Some(entries) = self.resolved_to_entry.remove(&past_address.address.unwrap()) {
            for (rc, interface, method_number) in entries.iter() {
                let by_address = ByAddress(rc.clone());
                let table = *self.inner.get(&by_address).unwrap();
                ITable::set_entry(table, *interface, *method_number, new_address);
            }
            self.resolved_to_entry.insert(new_address.address.unwrap(), entries);
        }
    }
}

pub fn generate_itable_access(
    assembler: &mut CodeAssembler,
    method_number: MethodNumber,
    interface_id: InterfaceID,
    raw_native_itable_address: Register,
    temp_1: Register,
    temp_2: Register,
    temp_3: Register,
    address: Register,
) {
    assert_ne!(temp_1, temp_2);
    assert_ne!(temp_1, temp_3);
    assert_ne!(temp_2, temp_3);
    assert_ne!(address, temp_3);
    assert_ne!(address, temp_2);
    assert_ne!(address, temp_1);
    let ptr_val = temp_1;
    let mut interface_found = assembler.create_label();
    let mut loop_start = assembler.create_label();
    let target_interface_id = temp_2;
    assembler.mov(target_interface_id.to_native_64(), interface_id.0 as u64).unwrap();
    assembler.mov(ptr_val.to_native_64(), raw_native_itable_address.to_native_64() + offset_of!(ITableRaw,itable_ptr)).unwrap();
    assembler.set_label(&mut loop_start).unwrap();
    assert_eq!(offset_of!(ITableEntryRaw,interface_id), 0);
    assembler.cmp(ptr_val.to_native_64() + offset_of!(ITableEntryRaw,interface_id), target_interface_id.to_native_32()).unwrap();
    assembler.je(interface_found.clone()).unwrap();
    assembler.lea(ptr_val.to_native_64(), ptr_val.to_native_64() + size_of::<ITableEntryRaw>()).unwrap();
    assembler.jmp(loop_start.clone()).unwrap();
    assembler.set_label(&mut interface_found).unwrap();
    assembler.mov(temp_3.to_native_64(), ptr_val.to_native_64() + offset_of!(ITableEntryRaw,vtable_ptr)).unwrap();
    assembler.mov(temp_2.to_native_64(), temp_3.to_native_64() + method_number.0 * size_of::<InterfaceVTableEntry>() as u32).unwrap();
    assembler.mov(address.to_native_64(), temp_2.to_native_64()).unwrap();
}
