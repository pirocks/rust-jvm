use std::ffi::c_void;
use std::mem::size_of;
use memoffset::offset_of;
use another_jit_vm::FramePointerOffset;

use jvmti_jni_bindings::jlong;
use runtime_class_stuff::{RuntimeClassClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::NativeJavaValue;


pub struct ObjectMemoryLayout {
    max_field_number_exclusive: u32,
}

impl ObjectMemoryLayout {
    pub fn from_rc(rc: &RuntimeClassClass) -> Self {
        Self {
            max_field_number_exclusive: rc.recursive_num_fields
        }
    }

    pub fn field_entry(&self, field_number: FieldNumber) -> usize {
        assert!(field_number.0 < self.max_field_number_exclusive);
        (field_number.0 as usize) * size_of::<NativeJavaValue>()
    }

    pub fn size(&self) -> usize {
        (self.max_field_number_exclusive as usize) * size_of::<NativeJavaValue>()
    }
}

pub struct ArrayMemoryLayout {
    sub_type: CPDType,
}

impl ArrayMemoryLayout {
    pub fn from_cpdtype(sub_type: CPDType) -> Self {
        assert_eq!(size_of::<jlong>(), size_of::<NativeJavaValue>());

        Self {
            sub_type
        }
    }

    pub fn elem_0_entry_offset(&self) -> usize {
        size_of::<jlong>()
    }
    pub fn len_entry_offset(&self) -> usize {
        0
    }
    pub fn elem_size(&self) -> usize {
        size_of::<NativeJavaValue>()
    }
    pub fn array_size(&self, len: i32) -> usize {
        self.elem_0_entry_offset() + len as usize * self.elem_size()
    }
}

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

pub trait StackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset;
    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset;
    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout;
    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout;
    fn full_frame_size(&self) -> usize;
    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset;
}
const MAX_OPERAND_STACK_NEEDED_FOR_FUNCTION_INVOCATION: usize = 256 * size_of::<jlong>();

pub struct FullyOpaqueFrame {
    pub max_stack: usize,
    pub max_frame: usize,
}

impl StackframeMemoryLayout for FullyOpaqueFrame {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }

    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset {
        todo!()
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + MAX_OPERAND_STACK_NEEDED_FOR_FUNCTION_INVOCATION + size_of::<jlong>()
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}

pub struct NativeStackframeMemoryLayout {
    pub num_locals: u16// num_locals does include top native functions, to allow same ircall mechanism
}

impl NativeStackframeMemoryLayout {
    pub fn local_var_entry(&self, i: u16) -> FramePointerOffset {
        assert!(i < self.num_locals as u16);
        FramePointerOffset(size_of::<FrameHeader>() + i as usize*size_of::<jlong>())
    }

    pub fn data_entry(&self) -> FramePointerOffset{
        FramePointerOffset(size_of::<FrameHeader>() + self.num_locals as usize*size_of::<jlong>())
    }

    pub fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + self.num_locals as usize*size_of::<jlong>() + size_of::<jlong>() //extra jlong for extra native data entry
    }
}



