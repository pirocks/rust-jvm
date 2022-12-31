use itertools::Either;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, Size};

use crate::{array_into_iter};
use compiler_common::{CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};

pub fn dup<'vm>(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
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
    } else {
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


pub fn dup2_x2(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1 = Register(2);
    let value2 = Register(3);
    let value3 = Register(4);
    let value4 = Register(5);
    let value1_is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 0);
    let value2_is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 1);
    if value1_is_category_2 {
        Either::Left(if value2_is_category_2 {
            //form 4
            Either::Left(array_into_iter([
                IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
                IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
            ]))
        } else {
            //form 2
            let value3_is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 1);
            assert!(!value3_is_category_2);
            Either::Right(
                array_into_iter([
                    IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
                    IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
                    IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: value3, size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value3, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 3), size: Size::X86QWord },
                ])
            )
        })
    } else {
        let value3_is_category_2 = method_frame_data.is_category_2(current_instr_data.current_index, 1);
        assert!(value3_is_category_2);
        Either::Right(if value3_is_category_2 {
            //form 3
            Either::Left(
                array_into_iter([
                    IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
                    IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
                    IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: value3, size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value3, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 3), size: Size::X86QWord },
                    IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 4), size: Size::X86QWord },
                ])
            )
        } else {
            //form 1
            Either::Right(array_into_iter([
                IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1, size: Size::X86QWord },
                IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2, size: Size::X86QWord },
                IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 2), to: value3, size: Size::X86QWord },
                IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 3), to: value4, size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value3, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 2), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value4, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 3), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value1, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 4), size: Size::X86QWord },
                IRInstr::StoreFPRelative { from: value2, to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 5), size: Size::X86QWord },
            ]))
        })
    }
}