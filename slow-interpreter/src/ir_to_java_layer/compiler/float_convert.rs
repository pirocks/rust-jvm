use another_jit_vm::{DoubleRegister, FloatRegister, MMRegister, Register};
use another_jit_vm_ir::compiler::{IRInstr, Size};
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn i2f(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: Register(1), size: Size::int() },
        IRInstr::IntegerToFloatConvert { to: FloatRegister(1), temp: MMRegister(1), from: Register(1) },
        IRInstr::StoreFPRelativeFloat { from: FloatRegister(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn l2f(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: Register(1), size: Size::int() },
        IRInstr::LongToFloatConvert { to: FloatRegister(1), from: Register(1) },
        IRInstr::StoreFPRelativeFloat { from: FloatRegister(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}


pub fn f2i(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: FloatRegister(1) },
        IRInstr::FloatToIntegerConvert { to: Register(1), temp: MMRegister(1), from: FloatRegister(1) },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}


pub fn i2d(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: Register(1), size: Size::int() },
        IRInstr::IntegerToDoubleConvert { to: DoubleRegister(1), temp: MMRegister(1), from: Register(1) },
        IRInstr::StoreFPRelativeDouble { from: DoubleRegister(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn d2i(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: DoubleRegister(1) },
        IRInstr::DoubleToIntegerConvert { to: Register(1), temp: MMRegister(1), from: DoubleRegister(1) },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn d2l(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelativeDouble { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: DoubleRegister(1) },
        IRInstr::DoubleToLongConvert { to: Register(1), temp: MMRegister(1), from: DoubleRegister(1) },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}

pub fn f2d(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: FloatRegister(1) },
        IRInstr::FloatToDoubleConvert { from: FloatRegister(1), to: DoubleRegister(2) },
        IRInstr::StoreFPRelativeDouble { from: DoubleRegister(2), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

