#![feature(box_syntax)]

use std::ffi::c_void;

use gc_memory_layout_common::FramePointerOffset;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;

use crate::java_stack::JavaStatus;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SavedRegisters {
    pub stack_pointer: *mut c_void,
    pub frame_pointer: *mut c_void,
    pub instruction_pointer: *mut c_void,
    pub status_register: *mut JavaStatus,
}

pub mod java_stack;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct JitCodeContext {
    //to jump back to when going back to native
    pub native_saved: SavedRegisters,
    pub java_saved: SavedRegisters,
    pub exit_handler_ip: *mut c_void
}

#[derive(Clone, Debug)]
pub enum VMExitData {
    CheckCast,
    InstanceOf,
    Throw,
    InvokeDynamic,
    InvokeStaticResolveTarget {
        method_name: MethodName,
        descriptor: CMethodDescriptor,
        classname_ref_type: CPRefType,
        native_start: *mut c_void,
        native_end: *mut c_void,
    },
    InvokeVirtualResolveTarget {
    },
    InvokeSpecialResolveTarget {
    },
    InvokeInterfaceResolveTarget {
    },
    MonitorEnter,
    MonitorExit,
    MultiNewArray,
    ArrayOutOfBounds,
    DebugTestExit,
    DebugTestExitValue {
        value: FramePointerOffset
    },
    ExitDueToCompletion,
}
