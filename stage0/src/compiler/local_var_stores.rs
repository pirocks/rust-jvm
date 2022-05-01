use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn astore_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let to_offset = method_frame_data.local_var_entry(current_instr_data.current_index, n);
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: from_offset, to: Register(1), size: Size::pointer() },
        IRInstr::StoreFPRelative { from: Register(1), to: to_offset, size: Size::pointer() },
    ])
}

pub fn istore_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let to_offset = method_frame_data.local_var_entry(current_instr_data.current_index, n);
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: from_offset, to: Register(1), size: Size::int() },
        IRInstr::StoreFPRelative { from: Register(1), to: to_offset, size: Size::int() },
    ])
}

pub fn lstore_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let to_offset = method_frame_data.local_var_entry(current_instr_data.current_index, n);
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: from_offset, to: Register(1), size: Size::long() },
        IRInstr::StoreFPRelative { from: Register(1), to: to_offset, size: Size::long() },
    ])
}


pub fn fstore_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let to_offset = method_frame_data.local_var_entry(current_instr_data.current_index, n);
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: from_offset, to: Register(1), size: Size::float() },
        IRInstr::StoreFPRelative { from: Register(1), to: to_offset, size: Size::float() },
    ])
}



pub fn dstore_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let to_offset = method_frame_data.local_var_entry(current_instr_data.current_index, n);
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: from_offset, to: Register(1), size: Size::double() },
        IRInstr::StoreFPRelative { from: Register(1), to: to_offset, size: Size::double() },
    ])
}

