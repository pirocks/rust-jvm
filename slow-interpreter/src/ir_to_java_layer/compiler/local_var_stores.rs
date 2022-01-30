use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn astore_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let to_offset = method_frame_data.local_var_entry(current_instr_data.current_index, n);
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: from_offset, to: Register(1) },
        IRInstr::StoreFPRelative { from: Register(1), to: to_offset },
    ])
}

