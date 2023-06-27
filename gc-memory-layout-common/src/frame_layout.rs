use std::ffi::c_void;
use std::mem::size_of;

use memoffset::offset_of;

use another_jit_vm::FramePointerOffset;
use jvmti_jni_bindings::{jlong};

//todo frane info will need to be reworked to be based of rip
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
    pub prev_rip: *mut c_void,
    pub prev_rpb: *mut c_void,
    pub ir_method_id: u64,
    pub methodid: usize,
    pub magic_part_1: u64,
    pub magic_part_2: u64,
}


pub const FRAME_HEADER_PREV_RIP_OFFSET: usize = offset_of!(FrameHeader,prev_rip);
pub const FRAME_HEADER_PREV_RBP_OFFSET: usize = offset_of!(FrameHeader, prev_rpb);
pub const FRAME_HEADER_IR_METHOD_ID_OFFSET: usize = offset_of!(FrameHeader,ir_method_id);
pub const FRAME_HEADER_METHOD_ID_OFFSET: usize = offset_of!(FrameHeader,methodid);
pub const FRAME_HEADER_PREV_MAGIC_1_OFFSET: usize = offset_of!(FrameHeader,magic_part_1);
pub const FRAME_HEADER_PREV_MAGIC_2_OFFSET: usize = offset_of!(FrameHeader,magic_part_2);
pub const FRAME_HEADER_END_OFFSET: usize = size_of::<FrameHeader>();



pub struct NativeStackframeMemoryLayout {
    pub num_locals: u16,// num_locals does include top native functions, to allow same ircall mechanism
}

impl NativeStackframeMemoryLayout {
    pub fn local_var_entry(&self, i: u16) -> FramePointerOffset {
        assert!(i < self.num_locals);
        FramePointerOffset(size_of::<FrameHeader>() + i as usize * size_of::<jlong>())
    }

    pub fn data_entry(&self) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + self.num_locals as usize * size_of::<jlong>())
    }

    pub fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + self.num_locals as usize * size_of::<jlong>() + size_of::<jlong>() //extra jlong for extra native data entry
    }
}


