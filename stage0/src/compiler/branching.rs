use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use rust_jvm_common::ByteCodeOffset;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

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
        ReferenceComparisonType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label, size: Size::pointer() },
        ReferenceComparisonType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label, size: Size::pointer() },
        // ReferenceComparisonType::GT => IRInstr::BranchAGreaterB { a: value1, b: value2, label: target_label },
    };
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::pointer() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::pointer() },
        compare_instr
    ])
}

pub fn goto_(current_instr_data: CurrentInstructionCompilerData, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
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
        IntEqualityType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::GT => IRInstr::BranchAGreaterB { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::GE => IRInstr::BranchAGreaterEqualB { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::LT => IRInstr::BranchAGreaterB { a: value2, b: value1, label: target_label, size: Size::int() },
        IntEqualityType::LE => IRInstr::BranchAGreaterEqualB { a: value2, b: value1, label: target_label, size: Size::int() },
    };
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value1, size: Size::int() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value2, size: Size::int() },
        compare_instr
    ])
}

pub fn if_(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, ref_equality: IntEqualityType, bytecode_offset: i32) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + bytecode_offset) as u16);
    let target_label = current_instr_data.compiler_labeler.label_at(target_offset);

    let compare_instr = match ref_equality {
        IntEqualityType::NE => IRInstr::BranchNotEqual { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::EQ => IRInstr::BranchEqual { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::LT => IRInstr::BranchAGreaterB { a: value2, b: value1, label: target_label, size: Size::int() },
        IntEqualityType::LE => IRInstr::BranchAGreaterEqualB { a: value2, b: value1, label: target_label, size: Size::int() },
        IntEqualityType::GE => IRInstr::BranchAGreaterEqualB { a: value1, b: value2, label: target_label, size: Size::int() },
        IntEqualityType::GT => IRInstr::BranchAGreaterB { a: value1, b: value2, label: target_label, size: Size::int() },
    };
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::int() },
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
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::pointer() },
        IRInstr::BranchNotEqual {
            a: value1,
            b: null,
            label: target_label,
            size: Size::pointer(),
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
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::pointer() },
        IRInstr::BranchEqual {
            a: value1,
            b: null,
            label: target_label,
            size: Size::pointer(),
        }
    ])
}

pub fn lookup_switch(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, pairs: &Vec<(i32, i32)>, default: &i32) -> impl Iterator<Item=IRInstr> {
    let mut res = vec![];
    let key_register = Register(1);
    res.push(IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: key_register, size: Size::int() });
    for (key, offset) in pairs {
        let current_key_register = Register(2);
        let target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + offset) as u16);
        let target_label = current_instr_data.compiler_labeler.label_at(target_offset);
        res.push(IRInstr::Const32bit { to: current_key_register, const_: *key as u32 });
        res.push(IRInstr::BranchEqual {
            a: key_register,
            b: current_key_register,
            label: target_label,
            size: Size::int(),
        });
    }
    //todo dup have a lable_at_offset which does this addition
    let default_target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + default) as u16);
    let default_target_label = current_instr_data.compiler_labeler.label_at(default_target_offset);
    res.push(IRInstr::BranchToLabel { label: default_target_label });
    let iter = res.into_iter();
    iter
}


pub fn tableswitch(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, default: &i32, low: &i32, high: &i32, offsets: &Vec<i32>) -> impl Iterator<Item=IRInstr> {
    let index = Register(1);
    let low_register = Register(2);
    let high_register = Register(3);
    let current_test = Register(4);
    let default_target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + *default) as u16);
    let default_label = current_instr_data.compiler_labeler.label_at(default_target_offset);
    let mut res = vec![];
    res.push(IRInstr::LoadFPRelative {
        from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0),
        to: index,
        size: Size::int(),
    });
    res.push(IRInstr::Const32bit { to: low_register, const_: *low as u32 });
    res.push(IRInstr::Const32bit { to: high_register, const_: *high as u32 });
    res.push(IRInstr::BranchAGreaterB {
        a: low_register,
        b: index,
        label: default_label,
        size: Size::int(),
    });
    res.push(IRInstr::BranchAGreaterB {
        a: index,
        b: high_register,
        label: default_label,
        size: Size::int(),
    });
    res.push(IRInstr::Sub {
        res: index,
        to_subtract: low_register,
        size: Size::int(),
    });
    for (i, offset) in offsets.iter().enumerate() {
        res.push(IRInstr::Const32bit { to: current_test, const_: i as u32 });
        let current_target_offset = ByteCodeOffset((current_instr_data.current_offset.0 as i32 + *offset) as u16);
        let current_label = current_instr_data.compiler_labeler.label_at(current_target_offset);
        res.push(IRInstr::BranchEqual {
            a: index,
            b: current_test,
            label: current_label,
            size: Size::int(),
        });
    }
    res.push(IRInstr::VMExit2 { exit_type: IRVMExitType::Todo { java_pc: current_instr_data.current_offset } });
    res.into_iter()
}
