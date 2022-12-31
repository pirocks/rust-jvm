use crate::CompilerState;
use crate::ir_compiler_common::{BranchToLabelID, TargetLabelID};

impl CompilerState{
    pub fn create_label(&mut self) -> (BranchToLabelID, TargetLabelID){
        todo!()
    }

    pub fn set_label_target(&mut self, label: TargetLabelID){
        todo!()
    }
}

