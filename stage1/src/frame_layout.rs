use std::collections::HashMap;
use rust_jvm_common::ByteCodeOffset;
use crate::{DoubleValue, FloatValue, IntegerValue, LongValue, PointerValue, ValueStatusChange};

pub struct Stage1FrameLayout {
    frame_size: usize,
    // first several mappings are at 0 offset and setyp local vars/define abi
    value_statues_mappings: Vec<(ByteCodeOffset, ValueStatusChange)>,
}

impl Stage1FrameLayout{
    pub fn compute_layout(&self) {
        todo!()
        /*let current = ComputedStage1FrameLayoutAtOffset{};
        for (offset, status_change) in self.value_statues_mappings {

        }*/
    }
}

pub enum ValueMapping {
    Pointer(PointerValue),
    Long(LongValue),
    Double(DoubleValue),
    Float(FloatValue),
    Integer(IntegerValue),
}

pub struct ComputedStage1FrameLayoutAtOffset {
    local_vars: Vec<ValueMapping>,
    operand_stack: Vec<ValueMapping>,
}

pub struct ComputedStage1FrameLayout {
    mappings: HashMap<ByteCodeOffset, ComputedStage1FrameLayoutAtOffset>,
}
