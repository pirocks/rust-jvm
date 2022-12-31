use another_jit_vm::{DoubleRegister, FloatRegister, Register};
use another_jit_vm::intrinsic_helpers::IntrinsicHelperType;
use another_jit_vm_ir::compiler::{FloatCompareMode, IRInstr, Size};

use crate::{array_into_iter};
use compiler_common::{CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

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
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::float() }
    ])
}


pub fn dcmpg(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let compare_mode = FloatCompareMode::G;
    dcmp(method_frame_data, current_instr_data, compare_mode)
}

pub fn dcmpl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let compare_mode = FloatCompareMode::L;
    dcmp(method_frame_data, current_instr_data, compare_mode)
}

fn dcmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, compare_mode: FloatCompareMode) -> impl Iterator<Item=IRInstr> {
    let value1 = DoubleRegister(0);
    let value2 = DoubleRegister(1);
    let res = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::DoubleCompare {
            value1,
            value2,
            res,
            temp1: Register(2),
            temp2: Register(3),
            temp3: Register(4),
            compare_mode,
        },
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::float() }
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

pub fn dmul(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = DoubleRegister(0);
    let value1 = DoubleRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::MulDouble { res: value1, a: value2 },
        IRInstr::StoreFPRelativeDouble { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
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

pub fn frem(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = FloatRegister(4);
    let value1 = FloatRegister(5);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::CallIntrinsicHelper { intrinsic_helper_type: IntrinsicHelperType::FRemF, integer_args: vec![], integer_res: None, float_args: vec![value1, value2], float_res: Some(value1), double_args: vec![], double_res: None },
        IRInstr::StoreFPRelativeFloat { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn drem(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = DoubleRegister(4);
    let value1 = DoubleRegister(5);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::CallIntrinsicHelper { intrinsic_helper_type: IntrinsicHelperType::DRemD, integer_args: vec![], integer_res: None, float_args: vec![], float_res: None, double_args: vec![value1, value2], double_res: Some(value1) },
        IRInstr::StoreFPRelativeDouble { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}


pub fn ddiv(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = DoubleRegister(0);
    let value1 = DoubleRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::DivDouble { res: value1, divisor: value2 },
        IRInstr::StoreFPRelativeDouble { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
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

pub fn fsub(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = FloatRegister(0);
    let value1 = FloatRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::SubFloat { res: value1, a: value2 },
        IRInstr::StoreFPRelativeFloat { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn fneg(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = FloatRegister(0);
    let zero = FloatRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::ConstFloat { to: zero, temp: Register(1), const_: -1.0 },
        IRInstr::MulFloat { res: zero, a: value2 },
        IRInstr::StoreFPRelativeFloat { from: zero, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn dneg(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = DoubleRegister(0);
    let zero = DoubleRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::ConstDouble { to: zero, temp: Register(1), const_: -1.0 },
        IRInstr::MulDouble { res: zero, a: value2 },
        IRInstr::StoreFPRelativeDouble { from: zero, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}


pub fn dsub(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = DoubleRegister(0);
    let value1 = DoubleRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::SubDouble { res: value1, a: value2 },
        IRInstr::StoreFPRelativeDouble { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn dadd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = DoubleRegister(0);
    let value1 = DoubleRegister(1);
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::AddDouble { res: value1, a: value2 },
        IRInstr::StoreFPRelativeDouble { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}