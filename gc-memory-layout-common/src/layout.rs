use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;

use jvmti_jni_bindings::jlong;
use runtime_class_stuff::{FieldNumber, RuntimeClassClass};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::NativeJavaValue;
use verification::verifier::Frame;

use crate::memory_regions::FramePointerOffset;

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

pub const MAGIC_1_EXPECTED: u64 = 0xDEADBEEFDEADBEAF;
pub const MAGIC_2_EXPECTED: u64 = 0xDEADCAFEDEADDEAD;

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


pub trait StackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset;
    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset;
    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout;
    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout;
    fn full_frame_size(&self) -> usize;
    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset;
}

pub struct FrameBackedStackframeMemoryLayout {
    method_frames: HashMap<u16, Frame>,
    max_stack: usize,
    max_locals: usize,
}

impl FrameBackedStackframeMemoryLayout {
    pub fn new(max_stack: usize, max_locals: usize, frame_vtypes: HashMap<u16, Frame>) -> Self {
        Self { method_frames: frame_vtypes, max_stack, max_locals }
    }
}

impl StackframeMemoryLayout for FrameBackedStackframeMemoryLayout {
    fn local_var_entry(&self, pc: u16, i: u16) -> FramePointerOffset {
        let locals = self.method_frames.get(&pc).unwrap().locals.clone(); //todo this rc could cross threads
        FramePointerOffset(locals.iter().take(i as usize).map(|_local_type| 8).sum())
        //for now everything is 8 bytes
    }

    fn operand_stack_entry(&self, pc: u16, from_end: u16) -> FramePointerOffset {
        let operand_stack = &self.method_frames.get(&pc).unwrap().stack_map.data;
        let len = operand_stack.len();
        let entry_idx = len - 1 - from_end as usize;
        FramePointerOffset(self.max_locals * 8 + entry_idx)
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + (self.max_locals + self.max_stack + 1) * size_of::<jlong>()
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}


#[derive(Debug)]
pub struct NaiveStackframeLayout {
    pub(crate) max_locals: u16,
    pub(crate) max_stack: u16,
    pub(crate) stack_depth: HashMap<u16, u16>,
}

impl NaiveStackframeLayout {
    pub fn from_stack_depth(stack_depth: HashMap<u16, u16>, max_locals: u16, max_stack: u16) -> Self {
        Self { max_locals, max_stack, stack_depth }
    }
}

impl StackframeMemoryLayout for NaiveStackframeLayout {
    fn local_var_entry(&self, current_count: u16, i: u16) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + i as usize * size_of::<jlong>())
    }

    fn operand_stack_entry(&self, current_count: u16, from_end: u16) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + (self.max_locals + self.stack_depth[&current_count] - from_end) as usize * size_of::<jlong>())
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + (self.max_locals as usize + self.max_stack as usize + 1) * size_of::<jlong>()
        // max stack is maximum depth which means we need 1 one more for size
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
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

pub struct NativeStackframeMemoryLayout {}

impl StackframeMemoryLayout for NativeStackframeMemoryLayout {
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



