#![feature(box_syntax)]

use std::ffi::c_void;
use gc_memory_layout_common::{FramePointerOffset, RegionData};

use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;

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
    pub runtime_type_info: RuntimeTypeInfo,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct RuntimeTypeInfo {
    pub small_num_regions: usize,
    pub medium_num_regions: usize,
    pub large_num_regions: usize,
    pub extra_large_num_regions: usize,
    pub small_region_index_to_region_data: *const RegionData,
    pub medium_region_index_to_region_data: *const RegionData,
    pub large_region_index_to_region_data: *const RegionData,
    pub extra_large_region_index_to_region_data: *const RegionData,

    pub allocated_type_to_vtable: *const VTableRaw,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VTableRaw {
    vtable_size: usize,
    vtable: *const *const c_void,
}

#[derive(Clone, Debug)]
pub enum VMExitData {
    CheckCast,
    InstanceOf,
    Throw,
    InvokeDynamic,
    InvokeStaticResolveTarget { method_name: MethodName, descriptor: CMethodDescriptor, classname_ref_type: CPRefType, native_start: *mut c_void, native_end: *mut c_void },
    InvokeVirtualResolveTarget {},
    InvokeSpecialResolveTarget {},
    InvokeInterfaceResolveTarget {},
    MonitorEnter,
    MonitorExit,
    MultiNewArray,
    ArrayOutOfBounds,
    DebugTestExit,
    DebugTestExitValue { value: FramePointerOffset },
    ExitDueToCompletion,
}

pub mod java_stack;