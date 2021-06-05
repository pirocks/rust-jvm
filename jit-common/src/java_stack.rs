use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;
use std::sync::RwLock;

use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use classfile_view::loading::LoaderName;
use gc_memory_layout_common::{FrameHeader, FrameInfo, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};

use crate::SavedRegisters;

#[derive(Debug)]
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

    pub fn current_frame(&self, current_rbp: *mut c_void) -> !/*FrameView*/ {
        let header = current_rbp as *const FrameHeader;
        let header = unsafe { header.as_ref() }.unwrap();
        assert_eq!({ header.magic_part_1 }, MAGIC_1_EXPECTED);
        assert_eq!({ header.magic_part_2 }, MAGIC_2_EXPECTED);
        /*FrameView(current_rbp)*/
        todo!()
    }

    pub fn saved_registers(&self) -> SavedRegisters {
        self.saved_registers.read().unwrap().unwrap()
    }
}


