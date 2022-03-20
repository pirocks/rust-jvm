use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};

use crate::ir_to_java_layer::compiler::{array_into_iter, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn dup<'vm_life>(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let temp_register = Register(1);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: temp_register, size: Size::X86QWord },
        IRInstr::StoreFPRelative { from: temp_register, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord }
    ])
}


pub fn dup_x1(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(2);
    let value2 = Register(3);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), from: value1, size: Size::X86QWord },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), from: value2, size: Size::X86QWord },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), from: value1, size: Size::X86QWord },
    ])
}

pub fn dup_x2(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(2);
    let value2 = Register(3);
    let value3 = Register(4);
    let value_2_is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 1);
    if value_2_is_category_2 {
        Either::Left(array_into_iter([
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
        ]))
    }else {
        Either::Right(array_into_iter([
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: value3, size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value3, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 3), size: Size::X86QWord },
        ]))
    }
}

pub fn dup2(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 0);
    if is_category_2 {
        Either::Left(array_into_iter([
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
        ]))
    } else {
        assert!(!method_frame_data.is_category_2(current_instr_data.current_index, 1));
        Either::Right(array_into_iter([
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 3), size: Size::X86QWord }
        ]))
    }
}


pub fn dup2_x1(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(1);
    let value2 = Register(2);
    let is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 0);
    if is_category_2 {
        //value1 is type 2
        //value2 is type 1
        Either::Left(array_into_iter([
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
        ]))
    } else {
        let value3 = Register(3);
        //all are type 1
        Either::Right(array_into_iter([
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
            IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: value3, size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value3, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 3), size: Size::X86QWord },
            IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 4), size: Size::X86QWord },
        ]))
    }
}
