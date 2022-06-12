#![feature(box_syntax)]

use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::Arc;
use by_address::ByAddress;
use itertools::Itertools;
use another_jit_vm::IRMethodID;
use method_table::interface_table::{InterfaceID, InterfaceTable};
use runtime_class_stuff::method_numbers::MethodNumber;
use runtime_class_stuff::{RuntimeClass};
use rust_jvm_common::compressed_classfile::{CPDTypeOrderWrapper};
use rust_jvm_common::compressed_classfile::names::{CClassName};
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
pub struct ITableEntry {
    interface_id: InterfaceID,
    vtable: Vec<UnsafeCell<InterfaceVTableEntry>>,
}

#[repr(C)]
pub struct ITable {
    itable: Vec<ITableEntry>,
}

impl ITable {
    pub fn new<'gc>(interface_table: &InterfaceTable<'gc>, interfaces: &[Arc<RuntimeClass<'gc>>]) -> Self {
        Self {
            itable: interfaces
                .iter()
                .sorted_by_key(|interface| CPDTypeOrderWrapper(interface.unwrap_class_class().class_view.name().to_cpdtype()))
                .map(|interface| {
                    let interface_id = interface_table.get_interface_id((*interface).clone());
                    let vtable_elems = (0..interface.unwrap_class_class().recursive_num_methods).map(|_| UnsafeCell::new(InterfaceVTableEntry::unresolved())).collect_vec();
                    ITableEntry { interface_id, vtable: vtable_elems }
                })
                .collect_vec()
        }
    }

    pub fn lookup<'gc>(table: NonNull<ITable>, interface_id: InterfaceID, interface_method_number: MethodNumber) -> Option<InterfaceVTableEntry> {
        unsafe {
            lookup_unsafe(table, interface_id, interface_method_number)
                .map(|address| InterfaceVTableEntry { address: Some(address) })
        }
    }

    pub fn set_entry<'gc>(table: NonNull<ITable>, interface_id: InterfaceID, interface_method_number: MethodNumber, resolved: InterfaceVTableEntry) {
        unsafe { write_resolved_unsafe(table, interface_id, interface_method_number, resolved.address.unwrap()) }
    }
}

pub unsafe fn write_resolved_unsafe(mut itable: NonNull<ITable>, interface_id: InterfaceID, interface_method_number: MethodNumber, res: NonNull<c_void>) {
    let table = itable.as_mut().itable.iter().find(|table| table.interface_id == interface_id).unwrap();
    table.vtable[interface_method_number.0 as usize].get().as_mut().unwrap().address = Some(res);
}


pub unsafe fn lookup_unsafe(mut itable: NonNull<ITable>, interface_id: InterfaceID, interface_method_number: MethodNumber) -> Option<NonNull<c_void>> {
    let table = itable.as_mut().itable.iter().find(|table| table.interface_id == interface_id).unwrap();
    table.vtable[interface_method_number.0 as usize].get().as_ref().unwrap().address
}

pub struct ITables<'gc> {
    inner: HashMap<ByAddress<Arc<RuntimeClass<'gc>>>, NonNull<ITable>>,
    resolved_to_entry: HashMap<NonNull<c_void>, Vec<(Arc<RuntimeClass<'gc>>, InterfaceID, MethodNumber)>>,
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

    pub fn lookup_or_new_itable(&mut self, interface_table: &InterfaceTable<'gc>, rc: Arc<RuntimeClass<'gc>>) -> NonNull<ITable> {
        *self.inner.entry(ByAddress(rc.clone())).or_insert_with(|| {
            unsafe {
                ITABLE_ALLOCS += 1;
                if ITABLE_ALLOCS % 1_000 == 0 {
                    dbg!(ITABLE_ALLOCS);
                }
            }
            let interfaces = Self::all_interfaces(&rc);
            NonNull::new(Box::into_raw(box ITable::new(interface_table, interfaces.as_slice()))).unwrap()
        }
        )
    }

    pub fn update(&mut self, past_address: InterfaceVTableEntry, new_address: InterfaceVTableEntry) {
        if let Some(entries) = self.resolved_to_entry.remove(&past_address.address.unwrap()) {
            for (rc, interface, method_number) in entries.iter() {
                let table = *self.inner.get(&ByAddress(rc.clone())).unwrap();
                ITable::set_entry(table, *interface, *method_number, new_address);
            }
            self.resolved_to_entry.insert(new_address.address.unwrap(), entries);
        }
    }
}