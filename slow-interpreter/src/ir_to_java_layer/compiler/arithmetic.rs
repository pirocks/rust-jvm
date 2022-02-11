use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn ladd(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let a = Register(1);
    let b = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: a },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: b },
        IRInstr::Add { res: b, a },
        IRInstr::StoreFPRelative { from: b, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }
    ])
}

