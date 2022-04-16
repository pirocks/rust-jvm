use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Signed, Size};

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn ladd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let a = Register(1);
    let b = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: a, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: b, size: Size::long() },
        IRInstr::Add { res: b, a, size: Size::long() },
        IRInstr::StoreFPRelative { from: b, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}


pub fn isub(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Sub { res: value1, to_subtract: value2, size: Size::int() },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn lsub(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::Sub { res: value1, to_subtract: value2, size: Size::long() },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}


pub fn iadd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Add { res: value1, a: value2, size: Size::int() },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn irem(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Mod { res: value1, divisor: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3), size: Size::int(), signed: Signed::Signed },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn lrem(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::Mod { res: value1, divisor: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3), size: Size::long(), signed: Signed::Signed },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}

pub fn idiv(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Div { res: value1, divisor: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3), size: Size::int(), signed: Signed::Signed },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn ldiv(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::Div { res: value1, divisor: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3), size: Size::long(), signed: Signed::Signed },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}


pub fn ineg(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let integer_to_neg = Register(6);
    let zero = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: integer_to_neg, size: Size::int() },
        IRInstr::Const32bit { to: zero, const_: 0 },
        IRInstr::Sub { res: zero, to_subtract: integer_to_neg, size: Size::int() },
        IRInstr::StoreFPRelative { from: zero, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn lneg(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let integer_to_neg = Register(6);
    let zero = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: integer_to_neg, size: Size::long() },
        IRInstr::Const64bit { to: zero, const_: 0 },
        IRInstr::Sub { res: zero, to_subtract: integer_to_neg, size: Size::long() },
        IRInstr::StoreFPRelative { from: zero, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}

pub fn imul(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Mul { res: value1, a: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3), size: Size::int(), signed: Signed::Signed },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn lmul(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(6);
    let value1 = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::Mul { res: value1, a: value2, must_be_rax: Register(0), must_be_rbx: Register(1), must_be_rcx: Register(2), must_be_rdx: Register(3), size: Size::long(), signed: Signed::Signed },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}

pub fn iinc(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, index: u16, const_: i16) -> impl Iterator<Item=IRInstr> {
    let temp = Register(1);
    let const_register = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, index), to: temp, size: Size::int() },
        IRInstr::Const64bit { to: const_register, const_: const_ as i64 as u64 },
        IRInstr::Add { res: temp, a: const_register, size: Size::int() },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.local_var_entry(current_instr_data.current_index, index), size: Size::int() }
    ])
}

pub fn lcmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(1);
    let value1 = Register(2);
    let res = Register(3);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::IntCompare { value1, value2, temp1: Register(4), temp2: Register(5), res, temp3: Register(6), size: Size::long() },
        IRInstr::StoreFPRelative { from: res, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}