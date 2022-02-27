use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn arraylength(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: Register(1), size: Size::int() },
        IRInstr::Load { to: Register(2), from_address: Register(1), size: Size::int() },
        IRInstr::StoreFPRelative { from: Register(2), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}
