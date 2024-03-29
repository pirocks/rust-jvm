use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};

use crate::compiler::{array_into_iter, CurrentInstructionCompilerData};
use crate::compiler_common::JavaCompilerMethodAndFrameData;

pub fn ireturn(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let return_temp = Register(1);

    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: return_temp, size: Size::int() },
        IRInstr::Return {
            return_val: Some(return_temp),
            temp_register_1: Register(2),
            temp_register_2: Register(3),
            temp_register_3: Register(4),
            temp_register_4: Register(5),
            frame_size: method_frame_data.full_frame_size(),
        }])
}

pub fn return_void<'vm>(method_frame_data: &JavaCompilerMethodAndFrameData) -> impl Iterator<Item=IRInstr> {
    array_into_iter([IRInstr::Return {
        return_val: None,
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size: method_frame_data.full_frame_size(),
    }])
}


pub fn areturn(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let return_temp = Register(1);

    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: return_temp, size: Size::pointer() },
        IRInstr::Return {
            return_val: Some(return_temp),
            temp_register_1: Register(2),
            temp_register_2: Register(3),
            temp_register_3: Register(4),
            temp_register_4: Register(5),
            frame_size: method_frame_data.full_frame_size(),
        }])
}


pub fn dreturn(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let return_temp = Register(1);

    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: return_temp, size: Size::double() },
        IRInstr::Return {
            return_val: Some(return_temp),
            temp_register_1: Register(2),
            temp_register_2: Register(3),
            temp_register_3: Register(4),
            temp_register_4: Register(5),
            frame_size: method_frame_data.full_frame_size(),
        }])
}

pub fn freturn(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let return_temp = Register(1);

    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: return_temp, size: Size::float() },
        IRInstr::Return {
            return_val: Some(return_temp),
            temp_register_1: Register(2),
            temp_register_2: Register(3),
            temp_register_3: Register(4),
            temp_register_4: Register(5),
            frame_size: method_frame_data.full_frame_size(),
        }])
}

pub fn lreturn(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let return_temp = Register(1);

    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: return_temp, size: Size::long() },
        IRInstr::Return {
            return_val: Some(return_temp),
            temp_register_1: Register(2),
            temp_register_2: Register(3),
            temp_register_3: Register(4),
            temp_register_4: Register(5),
            frame_size: method_frame_data.full_frame_size(),
        }])
}
