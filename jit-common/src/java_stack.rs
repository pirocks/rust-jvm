use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::ptr::null_mut;

use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use classfile_view::view::ptype_view::PTypeView;
use gc_memory_layout_common::{FrameHeader, FrameInfo, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, StackframeMemoryLayout};
use jvmti_jni_bindings::jobject;

use crate::SavedRegisters;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct JavaStatus {
    throw: jobject,
    pub function_return: bool,
    java_pc: isize,
}

impl Default for JavaStatus {
    fn default() -> Self {
        Self {
            throw: null_mut(),
            function_return: false,
            java_pc: 0,
        }
    }
}

#[derive(Debug)]
pub struct TopOfFrame(pub *mut c_void);

#[derive(Debug)]
pub struct JavaStack {
    pub top: *mut c_void,
    pub saved_registers: Option<SavedRegisters>,
    operand_stack_type_info: HashMap<TopOfFrame, Vec<PTypeView>>,
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
            saved_registers: Some(SavedRegisters {
                stack_pointer: unsafe { raw.offset(initial_frame_size as isize) },
                frame_pointer: raw,
                instruction_pointer: null_mut(),
                status_register: thread_status_register,
            }),
            operand_stack_type_info: Default::default(),
        }
    }

    pub fn current_jitted_code_id(&self) -> usize {
        //current memory layout includes prev rbp, prev rsp, method id, local vars, operand stack
        let current_rbp = &self.saved_registers;
        let rbp = current_rbp.as_ref().unwrap().frame_pointer;
        let method_id = unsafe { rbp.offset((2 * size_of::<u64>()) as isize) };
        (unsafe { *(method_id as *mut usize) }) as usize
    }

    pub fn handle_vm_exit(&mut self, to_save: SavedRegisters) {
        self.saved_registers = Some(to_save);
    }

    pub fn handle_vm_entry(&mut self) -> SavedRegisters {
        self.saved_registers.take().unwrap()
    }

    pub fn frame_pointer(&self) -> *mut c_void {
        self.saved_registers().frame_pointer
    }
    pub fn stack_pointer(&self) -> *mut c_void {
        self.saved_registers().stack_pointer
    }

    pub fn current_frame_ptr(&self) -> *mut c_void {
        if self.frame_pointer() == self.top {
            return self.top;//don't want to assert in this case
        }
        let header = self.frame_pointer() as *const FrameHeader;
        let header = unsafe { header.as_ref() }.unwrap();
        assert_eq!({ header.magic_part_1 }, MAGIC_1_EXPECTED);
        assert_eq!({ header.magic_part_2 }, MAGIC_2_EXPECTED);
        self.frame_pointer()
    }

    pub fn previous_frame_ptr(&self) -> *mut c_void {
        if self.frame_pointer() == self.top {
            return null_mut();//don't want to assert in this case
        }
        let header = self.frame_pointer() as *const FrameHeader;
        let header = unsafe { header.as_ref() }.unwrap();
        assert_eq!({ header.magic_part_1 }, MAGIC_1_EXPECTED);
        assert_eq!({ header.magic_part_2 }, MAGIC_2_EXPECTED);
        header.prev_rpb
    }

    pub fn saved_registers(&self) -> SavedRegisters {
        self.saved_registers.unwrap()
    }

    pub fn set_stack_pointer(&mut self, sp: *mut c_void) {
        self.saved_registers.as_mut().unwrap().stack_pointer = sp;
    }

    pub fn set_frame_pointer(&mut self, fp: *mut c_void) {
        self.saved_registers.as_mut().unwrap().frame_pointer = fp;
    }

    pub unsafe fn push_frame(&mut self, layout: &dyn StackframeMemoryLayout, frame_info: FrameInfo, prev_rip: Option<*mut c_void>) {
        let prev_rbp = self.frame_pointer();
        let prev_sp = self.stack_pointer();
        let new_rbp = prev_sp;
        let new_sp = new_rbp.offset(dbg!(layout.full_frame_size()) as isize);
        self.set_stack_pointer(new_sp);
        self.set_frame_pointer(new_rbp);
        dbg!(self.stack_pointer());
        let new_header = (new_rbp as *mut FrameHeader).as_mut().unwrap();
        new_header.magic_part_1 = MAGIC_1_EXPECTED;
        new_header.magic_part_2 = MAGIC_2_EXPECTED;
        new_header.frame_info_ptr = Box::into_raw(box frame_info);//leak dealt with in frame pop
        dbg!(new_header.frame_info_ptr);
        new_header.debug_ptr = null_mut();
        new_header.prev_rip = match prev_rip {
            None => transmute(0xDEADDEADDEADDEADusize),
            Some(prev_rip) => prev_rip
        };
        new_header.prev_rpb = prev_rbp;
    }

    pub unsafe fn pop_frame(&mut self) {
        let current_header = self.current_frame_ptr() as *const FrameHeader;
        let current_frame_info = (*current_header).frame_info_ptr;
        dbg!(current_frame_info);
        drop(Box::from_raw(current_frame_info));
        let new_rbp = (*current_header).prev_rpb;
        let new_sp = self.current_frame_ptr();
        self.set_frame_pointer(new_rbp);
        self.set_stack_pointer(new_sp);
        dbg!(self.stack_pointer());
    }

    pub fn throw(&self) -> jobject {
        unsafe { self.saved_registers.unwrap().status_register.as_ref() }.unwrap().throw
    }

    pub fn set_throw(&mut self, throw: jobject) {
        unsafe { self.saved_registers.unwrap().status_register.as_mut() }.unwrap().throw = throw;
    }

    pub unsafe fn call_stack_depth(&self) -> usize {
        let mut frame_header = self.frame_pointer() as *const FrameHeader;
        let mut depth = 0;
        loop {
            if (*frame_header).prev_rpb == self.top {
                return depth;
            } else {
                frame_header = (*frame_header).prev_rpb as *const FrameHeader;
                depth += 1;
            }
        }
    }
}


