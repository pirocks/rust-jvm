#![feature(box_syntax)]

use std::ffi::c_void;

use gc_memory_layout_common::FramePointerOffset;

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
}

#[derive(Copy, Clone, Debug)]
pub enum VMExitType {
    CheckCast,
    InstanceOf,
    Throw,
    InvokeDynamic,
    InvokeStaticResolveTarget {
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
