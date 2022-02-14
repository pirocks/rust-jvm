use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn aload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) },
    ])
}


pub fn iload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    //todo should mask or only load
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) },
    ])
}


pub fn lload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    //todo should mask or only load
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) },
    ])
}

pub fn fload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) },
    ])
}