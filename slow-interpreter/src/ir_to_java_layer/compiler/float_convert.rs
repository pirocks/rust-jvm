use another_jit_vm::{FloatRegister, MMRegister, Register};
use another_jit_vm_ir::compiler::IRInstr;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn i2f(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: Register(1) },
        IRInstr::IntegerToFloatConvert { to: FloatRegister(1), temp: MMRegister(1), from: Register(1) },
        IRInstr::StoreFPRelativeFloat { from: FloatRegister(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn f2i(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelativeFloat { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: FloatRegister(1) },
        IRInstr::FloatToIntegerConvert { to: Register(1), temp: MMRegister(1), from: FloatRegister(1) },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}
