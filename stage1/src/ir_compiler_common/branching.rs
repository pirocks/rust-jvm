use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset};
use crate::CompilerState;
use crate::ir_compiler_common::{BranchToLabelID, IntegerValueToken, TargetLabelID};
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
        todo!()
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

