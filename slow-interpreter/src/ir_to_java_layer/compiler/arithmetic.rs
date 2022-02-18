use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn ladd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let a = Register(1);
    let b = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: a },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: b },
        IRInstr::Add { res: b, a },
        IRInstr::StoreFPRelative { from: b, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn isub(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::Sub { res: value1, to_subtract: value2 },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn iadd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::Add { res: value1, a: value2 },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn irem(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::Mod { res: value1, divisor: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3) },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn iinc(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, index: &u16, const_: &i16) -> impl Iterator<Item=IRInstr> {
    let temp = Register(1);
    let const_register = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, *index), to: temp },
        IRInstr::Const64bit { to: const_register, const_: *const_ as i64 as u64 },
        IRInstr::Add { res: temp, a: const_register },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.local_var_entry(current_instr_data.current_index, *index) }
    ])
}

pub fn lcmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    let res = Register(3);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::IntCompare { value1, value2, temp1: Register(4), temp2: Register(5), res, temp3: Register(6) },
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}