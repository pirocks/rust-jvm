use std::ffi::c_void;

use gc_memory_layout_common::FramePointerOffset;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SavedRegisters {
    pub stack_pointer: *mut c_void,
    pub frame_pointer: *mut c_void,
    pub instruction_pointer: *mut c_void,
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
        resolved: FramePointerOffset
    },
    InvokeVirtualResolveTarget {
        resolved: FramePointerOffset
    },
    InvokeSpecialResolveTarget {
        resolved: FramePointerOffset
    },
    InvokeInterfaceResolveTarget {
        resolved: FramePointerOffset
    },
    MonitorEnter,
    MonitorExit,
    MultiNewArray,
    ArrayOutOfBounds,
    DebugTestExit,
    ExitDueToCompletion,
}
