use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use rust_jvm_common::NativeJavaValue;

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn const_64(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, n: u64) -> impl Iterator<Item=IRInstr> {
    let const_register = Register(1);

    array_into_iter([
        IRInstr::Const64bit { to: const_register, const_: n },
        IRInstr::StoreFPRelative { from: const_register, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
    ])
}

pub fn sipush(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, val: i16) -> impl Iterator<Item=IRInstr> {
    let sign_extended = val as i32 as u32;
    array_into_iter([
        IRInstr::Const32bit { to: Register(1), const_: sign_extended },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }])
}

pub fn bipush(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, val_: i8) -> impl Iterator<Item=IRInstr> {
    let sign_extended = val_ as i32 as u32;
    array_into_iter([
        IRInstr::Const32bit { to: Register(1), const_: sign_extended },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::int() }])
}

pub fn fconst(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, float_const: f32) -> impl Iterator<Item=IRInstr> {
    let mut zeroed_native = NativeJavaValue { as_u64: 0 };
    zeroed_native.float = float_const;
    const_64(method_frame_data, current_instr_data, unsafe { zeroed_native.as_u64 })
}

pub fn dconst(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, double_const: f64) -> impl Iterator<Item=IRInstr> {
    let mut zeroed_native = NativeJavaValue { as_u64: 0 };
    zeroed_native.double = double_const;
    const_64(method_frame_data, current_instr_data, unsafe { zeroed_native.as_u64 })
}