use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;

use crate::{array_into_iter, CurrentInstructionCompilerData};
use compiler_common::JavaCompilerMethodAndFrameData;

pub fn athrow(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([IRInstr::VMExit2 {
        exit_type: IRVMExitType::Throw {
            to_throw_obj_offset: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
            java_pc: current_instr_data.current_offset,
        }
    }])
}
