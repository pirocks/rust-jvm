use another_jit_vm::IRMethodID;
use compiler_common::JavaCompilerMethodAndFrameData;
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset, MethodId};
use crate::ir_compiler_common::{PointerValueToken, Stage1IRInstr, TargetLabelID};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRCompilerPosition {
    current_pc: ByteCodeOffset,
    index: ByteCodeIndex,
}

pub struct PendingExit{
    target_label_id: TargetLabelID,
    vm_exit_instr: Stage1IRInstr
}

pub struct IRCompilerState<'l> {
    method_frame_data: &'l JavaCompilerMethodAndFrameData,
    current: Option<IRCompilerPosition>,
    method_id: MethodId,
    ir_method_id: IRMethodID,
    res: Vec<Stage1IRInstr>,
    pending_exits: Vec<PendingExit>,
    should_trace_instructions: bool
}

impl <'l> IRCompilerState<'l> {
    pub fn new(
        method_id: MethodId,
        ir_method_id: IRMethodID,
        method_frame_data: &'l JavaCompilerMethodAndFrameData,
        should_trace_instructions: bool
    ) -> Self {
        let vec_capacity = method_frame_data.index_by_bytecode_offset.len()*2;
        Self {
            method_frame_data,
            current: None,
            method_id,
            ir_method_id,
            res: Vec::with_capacity(vec_capacity),
            pending_exits: vec![],
            should_trace_instructions,
        }
    }

    pub fn complete(self) -> Vec<Stage1IRInstr> {
        todo!()
    }

    pub fn emit_ir_start(&mut self) {
        self.res.push(Stage1IRInstr::IRStart {
            ir_method_id: self.ir_method_id,
            method_id: self.method_id,
            frame_size: self.method_frame_data.full_frame_size(),//todo actual frame size needs to be calculated after the fact todo.
        })
    }

    pub fn emit_monitor_enter(&mut self, obj: PointerValueToken) {
        self.res.push(Stage1IRInstr::MonitorEnter { java_pc: self.current.unwrap().current_pc, obj })
    }

    pub fn emit_get_class_object(&mut self) -> PointerValueToken {
        todo!()
    }

    pub fn emit_load_arg_pointer(&mut self, arg_num: u16) -> PointerValueToken {
        todo!()
    }

    pub fn notify_before_instruction(&mut self, current_pc: ByteCodeOffset, byte_code_index: ByteCodeIndex) {
        if self.should_trace_instructions{
            todo!("emit tracing exits")
        }
        self.current = Some(IRCompilerPosition{ current_pc, index: byte_code_index })
    }

    pub fn notify_after_instruction(&mut self, byte_code_offset: ByteCodeOffset) {
        todo!()
    }
}
