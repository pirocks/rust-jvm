use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;
use std::sync::RwLock;

use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::FramePointerOffset;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SavedRegisters {
    pub stack_pointer: *mut c_void,
    pub frame_pointer: *mut c_void,
    pub instruction_pointer: *mut c_void,
}

pub struct JavaStack {
    top: *mut c_void,
    saved_registers: RwLock<Option<SavedRegisters>>,

}

pub const STACK_LOCATION: usize = 0x1_000_000_000usize;

pub const MAX_STACK_SIZE: usize = 0x10_000_000usize;


impl JavaStack {
    pub fn new(initial_frame_size: usize) -> Self {
        assert!(initial_frame_size < 4096);
        let prot_flags = ProtFlags::PROT_WRITE | ProtFlags::PROT_READ | ProtFlags::PROT_GROWSDOWN;
        let map_flags = MapFlags::MAP_STACK | MapFlags::MAP_NORESERVE | MapFlags::MAP_PRIVATE | MapFlags::MAP_ANONYMOUS;
        let raw = unsafe { mmap(transmute(STACK_LOCATION), MAX_STACK_SIZE, prot_flags, map_flags, -1, 0) }.unwrap();
        Self {
            top: raw,
            saved_registers: RwLock::new(Some(SavedRegisters {
                stack_pointer: unsafe { raw.offset(initial_frame_size as isize) },
                frame_pointer: raw,
                instruction_pointer: null_mut(),
            })),
        }
    }

    pub fn current_jitted_code_id(&self) -> usize {
        //current memory layout includes prev rbp, prev rsp, method id, local vars, operand stack
        let current_rbp = self.saved_registers.read().unwrap();
        let rbp = current_rbp.as_ref().unwrap().frame_pointer;
        let method_id = unsafe { rbp.offset((2 * size_of::<u64>()) as isize) };
        (unsafe { *(method_id as *mut usize) }) as usize
    }

    pub fn handle_vm_exit(&self, to_save: SavedRegisters) {
        *self.saved_registers.write().unwrap() = Some(to_save);
    }
    pub fn handle_vm_entry(&self) -> SavedRegisters {
        self.saved_registers.write().unwrap().take().unwrap()
    }
}


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
