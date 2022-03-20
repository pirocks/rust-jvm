use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{BitwiseLogicType, IRInstr, Size};

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn lshl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let shift_amount = Register(1);
    let value = Register(3);
    let mask = Register(4);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: shift_amount, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value, size: Size::long() },
        IRInstr::Const16bit { to: mask, const_: 0x3f },
        IRInstr::BinaryBitAnd { res: shift_amount, a: mask, size: Size::long() },
        IRInstr::ShiftLeft { res: value, a: shift_amount, cl_aka_register_2: Register(2), size: Size::long(), signed: BitwiseLogicType::Arithmetic },
        IRInstr::StoreFPRelative { from: value, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}


pub fn ishl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let shift_amount = Register(1);
    let value = Register(3);
    let mask = Register(4);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: shift_amount, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value, size: Size::int() },
        IRInstr::Const16bit { to: mask, const_: 0x1f },
        IRInstr::BinaryBitAnd { res: shift_amount, a: mask, size: Size::int() },
        IRInstr::ShiftLeft { res: value, a: shift_amount, cl_aka_register_2: Register(2), size: Size::int(), signed: BitwiseLogicType::Arithmetic },
        IRInstr::StoreFPRelative { from: value, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}


pub fn land(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::BinaryBitAnd { res: value2, a: value1, size: Size::long() },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}

pub fn lor(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::BinaryBitOr { res: value2, a: value1, size: Size::long() },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}


pub fn ixor(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::BinaryBitXor { res: value2, a: value1, size: Size::int() },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn lxor(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::BinaryBitXor { res: value2, a: value1, size: Size::long() },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}

pub fn ior(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::BinaryBitOr { res: value2, a: value1, size: Size::int() },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn iand(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let mask = Register(3);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::BinaryBitAnd { res: value2, a: value1, size: Size::int() },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn iushr(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(3);
    let value1 = Register(4);
    let const_ = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Const16bit { to: const_, const_: 0x1f },
        IRInstr::BinaryBitAnd { res: value2, a: const_, size: Size::int() },
        IRInstr::ShiftRight {
            res: value1,
            a: value2,
            cl_aka_register_2: Register(2),
            size: Size::int(),
            signed: BitwiseLogicType::Logical,
        },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn lushr(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(3);
    let value1 = Register(4);
    let const_ = Register(5);
    //todo handle negative shift
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::Const16bit { to: const_, const_: 0x3f },
        IRInstr::BinaryBitAnd { res: value2, a: const_, size: Size::int() },
        IRInstr::ShiftRight {
            res: value1,
            a: value2,
            cl_aka_register_2: Register(2),
            size: Size::long(),
            signed: BitwiseLogicType::Logical,
        },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}


pub fn ishr(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(3);
    let value1 = Register(4);
    let const_ = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::Const16bit { to: const_, const_: 0x1f },
        IRInstr::BinaryBitAnd { res: value2, a: const_, size: Size::int() },
        IRInstr::ShiftRight {
            res: value1,
            a: value2,
            cl_aka_register_2: Register(2),
            size: Size::int(),
            signed: BitwiseLogicType::Arithmetic,
        },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}


pub fn lshr(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value2 = Register(3);
    let value1 = Register(4);
    let const_ = Register(5);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::long() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::long() },
        IRInstr::Const16bit { to: const_, const_: 0x3f },
        IRInstr::BinaryBitAnd { res: value2, a: const_, size: Size::long() },
        IRInstr::ShiftRight {
            res: value1,
            a: value2,
            cl_aka_register_2: Register(2),
            size: Size::long(),
            signed: BitwiseLogicType::Arithmetic,
        },
        IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::long() }
    ])
}
