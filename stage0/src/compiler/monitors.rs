use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn monitor_enter(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    //todo probably needs null check
    array_into_iter([IRInstr::VMExit2 { exit_type: IRVMExitType::MonitorEnter { obj: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), java_pc: current_instr_data.current_offset } }])
}


pub fn monitor_exit(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([IRInstr::VMExit2 { exit_type: IRVMExitType::MonitorExit { obj: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), java_pc: current_instr_data.current_offset } }])
}
