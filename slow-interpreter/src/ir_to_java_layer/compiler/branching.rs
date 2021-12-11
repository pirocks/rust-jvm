use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
use crate::jit::ByteCodeOffset;
use crate::jit::ir::{IRInstr, Register};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReferenceEqualityType {
    NE,
    EQ,
}

pub fn if_acmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, ref_equality: ReferenceEqualityType, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(0);
    let value2 = Register(1);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    let compare_instr = match ref_equality {
        ReferenceEqualityType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label },
        ReferenceEqualityType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label }
    };
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        compare_instr
    ])
}

pub fn goto_(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);
    array_into_iter([IRInstr::BranchToLabel { label: target_label }])
}
