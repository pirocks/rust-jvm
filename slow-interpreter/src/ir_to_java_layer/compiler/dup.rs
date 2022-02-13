use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn dup<'vm_life>(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let temp_register = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: temp_register },
        IRInstr::StoreFPRelative { from: temp_register, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}


pub fn dup_x1(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(2);
    let value2 = Register(3);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1 },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), from: value1 },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), from: value2 },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), from: value1 },
    ])
}

