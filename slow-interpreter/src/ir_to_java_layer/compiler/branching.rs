use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use rust_jvm_common::ByteCodeOffset;

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReferenceComparisonType {
    NE,
    EQ,
}

pub fn if_acmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, ref_comparison: ReferenceComparisonType, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    let compare_instr = match ref_comparison {
        ReferenceComparisonType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label },
        ReferenceComparisonType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label },
        // ReferenceComparisonType::GT => IRInstr::BranchAGreaterB { a: value1, b: value2, label: target_label },
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


pub enum IntEqualityType {
    NE,
    EQ,
    LT,
    GE,
    GT,
    LE,
}

pub fn if_icmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, ref_equality: IntEqualityType, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    let compare_instr = match ref_equality {
        IntEqualityType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label },
        IntEqualityType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label },
        IntEqualityType::GT => IRInstr::BranchAGreaterB { a: value1, b: value2, label: target_label },
        IntEqualityType::GE => IRInstr::BranchAGreaterEqualB { a: value1, b: value2, label: target_label },
        IntEqualityType::LT => IRInstr::BranchAGreaterB { a: value2, b: value1, label: target_label },
        IntEqualityType::LE => IRInstr::BranchAGreaterEqualB { a: value2, b: value1, label: target_label },
    };
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2 },
        compare_instr
    ])
}

pub fn if_(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, ref_equality: IntEqualityType, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    let compare_instr = match ref_equality {
        IntEqualityType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label },
        IntEqualityType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label },
        IntEqualityType::LT => IRInstr::BranchAGreaterB { a: value2, b: value1, label: target_label },
        IntEqualityType::LE => IRInstr::BranchAGreaterEqualB { a: value2, b: value1, label: target_label },
        _ => panic!()
    };
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1 },
        IRInstr::Const64bit { to: value2, const_: 0 },
        compare_instr
    ])
}


pub fn if_nonnull(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let null = Register(2);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    array_into_iter([
        IRInstr::Const64bit { to: null, const_: 0 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1 },
        IRInstr::BranchNotEqual {
            a: value1,
            b: null,
            label: target_label,
        }
    ])
}


pub fn if_null(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    //todo dup with above
    let value1 = Register(1);
    let null = Register(2);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    array_into_iter([
        IRInstr::Const64bit { to: null, const_: 0 },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1 },
        IRInstr::BranchEqual {
            a: value1,
            b: null,
            label: target_label,
        }
    ])
}