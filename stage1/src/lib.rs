use another_jit_vm::IRMethodID;
use compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};
use rust_jvm_common::MethodId;

pub enum Stage1IRInstr{
    IRStart {
        ir_method_id: IRMethodID,
        method_id: MethodId,
        frame_size: usize,
    },
}



pub struct CompilerState{

}

impl CompilerState{
    pub fn new() -> Self{
        Self{

        }
    }
}

pub fn compile_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, method_frame_data: &JavaCompilerMethodAndFrameData, method_id: MethodId, ir_method_id: IRMethodID) -> Vec<Stage1IRInstr> {
    let mut res = vec![];
    res.push(Stage1IRInstr::IRStart {
        ir_method_id,
        method_id,
        frame_size: method_frame_data.full_frame_size(),
    });
    res
}