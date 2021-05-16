use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::fs::read;
use std::mem::size_of;

use iced_x86::ConditionCode::s;
use itertools::{Either, Itertools};

use verification::verifier::Frame;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum PointerMemoryLayout {}

impl PointerMemoryLayout {
    pub fn get_gc_pointer_offsets(&self) -> Vec<usize> {
        todo!()
    }

    pub fn as_object(&self) -> ObjectMemoryLayout {
        todo!()
    }

    pub fn as_array(&self) -> ArrayMemoryLayout {
        todo!()
    }

    pub fn monitor_entry(&self) -> usize {
        todo!()
    }

    pub fn class_pointer_entry(&self) -> usize {
        todo!()
    }

    pub fn total_size(&self) -> usize {
        todo!()
    }
}


pub struct ObjectMemoryLayout {
    elems: HashMap<usize/*field id*/, usize>,
}

impl ObjectMemoryLayout {
    pub fn field_entry(&self) -> usize {
        todo!()
    }
}

pub struct ArrayMemoryLayout {}

impl ArrayMemoryLayout {
    pub fn elem_0_entry(&self) -> usize {
        todo!()
    }
    pub fn len_entry(&self) -> usize {
        todo!()
    }
    pub fn elem_size(&self) -> usize {
        todo!()
    }
}


pub struct FramePointerOffset(pub usize);

pub struct StackframeMemoryLayout {
    method_frames: HashMap<usize, Frame>,
    max_stack: usize,
    max_locals: usize,
}

impl StackframeMemoryLayout {
    pub fn local_var_entry(&self, pc: usize, i: usize) -> FramePointerOffset {
        let locals = self.method_frames.get(&pc).unwrap().locals.clone();//todo this rc could cross threads
        FramePointerOffset(locals.iter().take(i).map(|_local_type| 8).sum())//for now everything is 8 bytes
    }

    pub fn operand_stack_entry(&self, pc: usize, from_end: usize) -> FramePointerOffset {
        let operand_stack = &self.method_frames.get(&pc).unwrap().stack_map.data;
        let len = operand_stack.len();
        let entry_idx = len - 1 - from_end;
        FramePointerOffset(self.max_locals * 8 + entry_idx)
    }

    pub fn full_frame_size(&self) -> usize {
        todo!()
    }
}