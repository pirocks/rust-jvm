use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn lshl(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let shift_amount = Register(1);
    let value = Register(3);
    let mask = Register(4);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: shift_amount },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value },
        IRInstr::Const64bit { to: mask, const_: 0x3f },
        IRInstr::BinaryBitAnd { res: shift_amount, a: mask },
        IRInstr::LogicalShiftLeft { res: value, a: shift_amount, cl_aka_register_2: Register(2) },
        IRInstr::StoreFPRelative { from: value, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}


pub fn land(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let mask = Register(3);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::BinaryBitAnd { res: value2, a: value1 },
        IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index,0) }
    ])
}
