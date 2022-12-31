use another_jit_vm::IRMethodID;
use compiler_common::JavaCompilerMethodAndFrameData;
use rust_jvm_common::MethodId;
use crate::ir_compiler_common::{IRCompilerState, PointerValueToken, Stage1IRInstr};

impl IRCompilerState {
    pub fn new(
        method_id: MethodId,
        ir_method_id: IRMethodID,
        method_frame_data: &JavaCompilerMethodAndFrameData
    ) -> Self {
        Self {}
    }

    pub fn complete(self) -> Vec<Stage1IRInstr> {
        todo!()
    }

    pub fn emit_ir_start(&mut self) {
        todo!()
    }

    pub fn emit_monitor_enter(&mut self, obj: PointerValueToken) {
        todo!()
    }

    pub fn emit_get_class_object(&mut self) -> PointerValueToken {
        todo!()
    }

    pub fn emit_load_arg_pointer(&mut self, arg_num: u16) -> PointerValueToken {
        todo!()
    }
}
