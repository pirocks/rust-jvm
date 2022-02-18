use another_jit_vm::{FloatRegister, Register};
use another_jit_vm_ir::compiler::{FloatCompareMode, IRInstr};
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn fcmpg(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let compare_mode = FloatCompareMode::G;
    fcmp(method_frame_data, current_instr_data, compare_mode)
}

pub fn fcmpl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let compare_mode = FloatCompareMode::L;
    fcmp(method_frame_data, current_instr_data, compare_mode)
}

fn fcmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, compare_mode: FloatCompareMode) -> impl Iterator<Item=IRInstr> {
    let value1 = FloatRegister(0);
    let value2 = FloatRegister(1);
    let res = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::FloatCompare {
            value1,
            value2,
            res,
            temp1: Register(2),
            temp2: Register(3),
            temp3: Register(4),
            compare_mode,
        },
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}


pub fn fmul(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = FloatRegister(0);
    let value1 = FloatRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::MulFloat { res: value1, a: value2 },
        IRInstr::StoreFPRelativeFloat { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn fdiv(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = FloatRegister(0);
    let value1 = FloatRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::DivFloat { res: value1, divisor: value2 },
        IRInstr::StoreFPRelativeFloat { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn fadd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = FloatRegister(0);
    let value1 = FloatRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::AddFloat { res: value1, a: value2 },
        IRInstr::StoreFPRelativeFloat { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}