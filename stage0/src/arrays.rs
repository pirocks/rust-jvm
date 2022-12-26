use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};

use crate::{array_into_iter, CurrentInstructionCompilerData};
use compiler_common::JavaCompilerMethodAndFrameData;

pub fn arraylength(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: Register(1), size: Size::pointer() },
        IRInstr::Load { to: Register(2), from_address: Register(1), size: Size::int() },
        IRInstr::StoreFPRelative { from: Register(2), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }
    ])
}
