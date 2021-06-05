use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;
use std::sync::RwLock;

use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::{FrameHeader, FrameInfo, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, StackframeMemoryLayout};
use jvmti_jni_bindings::jobject;
use rust_jvm_common::classfile::InstructionInfo::new;

use crate::SavedRegisters;

pub struct JavaStatus {
    pub throw: jobject,
}

#[derive(Debug)]
pub struct JavaStack {
    top: *mut c_void,
    saved_registers: RwLock<Option<SavedRegisters>>,
}

pub const STACK_LOCATION: usize = 0x1_000_000_000usize;

pub const MAX_STACK_SIZE: usize = 0x10_000_000usize;


impl JavaStack {
    pub fn new(initial_frame_size: usize, thread_status_register: *mut JavaStatus) -> Self {
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
                status_register: thread_status_register
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

    pub fn frame_pointer(&self) -> *mut c_void {
        self.saved_registers().frame_pointer
    }

    pub fn stack_pointer(&self) -> *mut c_void {
        self.saved_registers().stack_pointer
    }

    pub fn current_frame_ptr(&self) -> *mut c_void {
        let header = self.frame_pointer() as *const FrameHeader;
        let header = unsafe { header.as_ref() }.unwrap();
        assert_eq!({ header.magic_part_1 }, MAGIC_1_EXPECTED);
        assert_eq!({ header.magic_part_2 }, MAGIC_2_EXPECTED);
        self.frame_pointer()
    }

    pub fn saved_registers(&self) -> SavedRegisters {
        self.saved_registers.read().unwrap().unwrap()
    }

    pub fn set_stack_pointer(&self, sp: *mut c_void) {
        self.saved_registers.write().unwrap().as_mut().unwrap().stack_pointer = sp;
    }

    pub unsafe fn push_frame(&self, layout: &dyn StackframeMemoryLayout, frame_info: FrameInfo) {
        let prev_rbp = self.frame_pointer();
        let prev_sp = self.stack_pointer();
        let new_rbp = prev_sp;
        let new_sp = new_rbp.offset(layout.full_frame_size() as isize);
        self.set_stack_pointer(new_sp);
        let new_header = (new_rbp as *mut FrameHeader).as_mut().unwrap();
        new_header.magic_part_1 = MAGIC_1_EXPECTED;
        new_header.magic_part_2 = MAGIC_2_EXPECTED;
        new_header.frame_info_ptr = Box::into_raw(box frame_info);//TODO DEAL WITH THIS LEAK in frame pop
        new_header.debug_ptr = null_mut();
        new_header.prev_rip = transmute(0xDEADDEADDEADDEADusize);
        new_header.prev_rpb = prev_rbp;
    }

    pub fn throw(&self) -> jobject {
        unsafe { self.saved_registers.read().unwrap().unwrap().status_register.as_ref() }.unwrap().throw
    }
}


