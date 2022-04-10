use std::ffi::c_void;
use std::ptr::NonNull;
use another_jit_vm::IRMethodID;
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
#[derive(Copy, Clone)]
pub struct InterfaceVTableEntry {
    pub address: Option<NonNull<c_void>>,
    //null indicates need for resolve
    pub ir_method_id: IRMethodID,
    pub method_id: MethodId,
    pub new_frame_size: usize,
}
