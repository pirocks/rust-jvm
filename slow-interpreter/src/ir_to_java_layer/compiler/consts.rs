use another_jit_vm::Register;
use another_jit_vm_ir::compiler::IRInstr;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};


pub fn const_64(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, n: u64) -> impl Iterator<Item=IRInstr> {
    let const_register = Register(1);

    array_into_iter([
        IRInstr::Const64bit { to: const_register, const_: n },
        IRInstr::StoreFPRelative { from: const_register, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) },
    ])
}

pub fn sipush(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, val: &u16) -> impl Iterator<Item=IRInstr> {
    array_into_iter([IRInstr::Const16bit { to: Register(1), const_: *val },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }])
}

pub fn bipush(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, val_: &u8) -> impl Iterator<Item=IRInstr> {
    array_into_iter([IRInstr::Const32bit { to: Register(1), const_: *val_ as i8 as i32 as u32 },
        IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }])
}