use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn aload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp, size: Size::pointer() },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::pointer() },
    ])
}


pub fn iload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    //todo should mask or only load
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp, size: Size::int() },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() },
    ])
}


pub fn lload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    //todo should mask or only load
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp, size: Size::long() },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() },
    ])
}

pub fn dload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    //todo should mask or only load
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp, size: Size::double() },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::double() },
    ])
}

pub fn fload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp, size: Size::long()},
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() },
    ])
}