use std::ffi::c_void;


use crate::jit_common::java_stack::JavaStatus;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SavedRegisters {
    pub stack_pointer: *mut c_void,
    pub frame_pointer: *mut c_void,
    pub instruction_pointer: *mut c_void,
    pub status_register: *mut JavaStatus,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct JitCodeContext {
    //to jump back to when going back to native
    pub native_saved: SavedRegisters,
    pub java_saved: SavedRegisters,
    pub exit_handler_ip: *mut c_void,
}


#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VTableRaw {
    vtable_size: usize,
    vtable: *const *const c_void,
}

pub mod java_stack;