use rust_jvm_common::ByteCodeOffset;
use crate::ir_compiler_common::{BranchToLabelID, IntegerValueToken, TargetLabelID, TargetLabelIDInternal};
use crate::ir_compiler_common::special::IRCompilerState;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum IntegerCompareKind{
    NotEqual,
    Equal,
    LessThan,
    GreaterThan,
    LessThanEqual,
    GreaterThanEqual
}

impl IRCompilerState<'_>{
    pub fn create_label(&mut self) -> (BranchToLabelID, TargetLabelID){
        //todo this could be simplified since ids are always  same
        let new_label_id = self.labels.len();
        let branch_to_label_id = BranchToLabelID(new_label_id as u32);
        let target_label_id = TargetLabelIDInternal(new_label_id as u32);
        self.labels.insert(target_label_id, branch_to_label_id);
        (branch_to_label_id, TargetLabelID(new_label_id as u32))
    }

    pub fn set_label_target(&mut self, label: TargetLabelID){
        todo!()
    }

    pub fn set_label_target_pending(&mut self, byte_code_offset: ByteCodeOffset, label: TargetLabelID) {
        todo!()
    }

    pub fn emit_branch_compare_int(&self, branch_to: BranchToLabelID, a: IntegerValueToken, b: IntegerValueToken, compare_kind: IntegerCompareKind){
        todo!()
    }
}

