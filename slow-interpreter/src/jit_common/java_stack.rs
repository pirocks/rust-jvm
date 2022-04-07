use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::Debug;
use std::ptr::null_mut;
use another_jit_vm::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};

use classfile_view::view::ptype_view::PTypeView;
use gc_memory_layout_common::layout::{FrameHeader};
use jvmti_jni_bindings::jobject;

use crate::jit_common::SavedRegisters;

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct JavaStatus {
    throw: jobject,
    pub function_return: bool,
    java_pc: isize,
}

impl Default for JavaStatus {
    fn default() -> Self {
        Self { throw: null_mut(), function_return: false, java_pc: 0 }
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
            return self.top; //don't want to assert in this case
        }
        let header = self.frame_pointer() as *const FrameHeader;
        let header = unsafe { header.as_ref() }.unwrap();
        assert_eq!({ header.magic_part_1 }, MAGIC_1_EXPECTED);
        assert_eq!({ header.magic_part_2 }, MAGIC_2_EXPECTED);
        self.frame_pointer()
    }

    pub fn previous_frame_ptr(&self) -> *mut c_void {
        if self.frame_pointer() == self.top {
            return null_mut(); //don't want to assert in this case
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