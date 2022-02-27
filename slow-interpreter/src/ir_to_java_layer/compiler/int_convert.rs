use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};
use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn i2l(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    let to_offset = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
    let from_register = Register(1);
    let to_register = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative {
            from: from_offset,
            to: from_register,
            size: Size::int()
        },
        IRInstr::SignExtend {
            from: from_register,
            to: to_register,
            from_size: Size::int(),
            to_size: Size::long()
        },
        IRInstr::StoreFPRelative {
            from: to_register,
            to: to_offset,
            size: Size::long()
        }
    ])
}

pub fn i2c(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    let to_offset = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
    let from_register = Register(1);
    let to_register = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative {
            from: from_offset,
            to: from_register,
            size: Size::char()
        },
        IRInstr::SignExtend {
            from: from_register,
            to: to_register,
            from_size: Size::char(),
            to_size: Size::int()
        },
        IRInstr::StoreFPRelative {
            from: to_register,
            to: to_offset,
            size: Size::int()
        }
    ])
}

pub fn i2b(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let from_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index, 0);
    let to_offset = method_frame_data.operand_stack_entry(current_instr_data.next_index, 0);
    let from_register = Register(1);
    let to_register = Register(2);
    array_into_iter([
        IRInstr::LoadFPRelative {
            from: from_offset,
            to: from_register,
            size: Size::byte()
        },
        IRInstr::SignExtend {
            from: from_register,
            to: to_register,
            from_size: Size::byte(),
            to_size: Size::int()
        },
        IRInstr::StoreFPRelative {
            from: to_register,
            to: to_offset,
            size: Size::int()
        }
    ])
}

