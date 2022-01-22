use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};


pub fn const_64(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, n: u64) -> impl Iterator<Item=IRInstr> {
    let const_register = Register(1);

    array_into_iter([
        IRInstr::Const64bit { to: const_register, const_: n },
        IRInstr::StoreFPRelative { from: const_register, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) },
        IRInstr::VMExit2 { exit_type: IRVMExitType::LogWholeFrame {} }
    ])
}
