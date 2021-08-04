use std::mem::size_of;

use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, StackframeMemoryLayout};
use jit_common::VMExitType;
use jit_ir::{ArithmeticType, BranchType, Constant, IRInstruction, IRLabel, Size};
use jvmti_jni_bindings::{jbyte, jint, jlong, jshort};

use crate::{JitBlock, JITError, JitState};

pub fn array_out_of_bounds_block(current_jit_state: &mut JitState, _index_offset: FramePointerOffset) -> Result<(JitBlock, IRLabel), JITError> {
    let mut block = JitBlock {
        ir_to_java_pc: Default::default(),
        instructions: vec![],
    };
    let label = current_jit_state.new_ir_label();
    block.add_instruction(IRInstruction::Label(label), current_jit_state.java_pc);
    block.add_instruction(IRInstruction::VMExit(VMExitType::ArrayOutOfBounds), current_jit_state.java_pc);
    Ok((block, label))
}


pub fn array_store(current_jit_state: &mut JitState, size: Size) -> Result<(), JITError> {
    // array, i, val
    let java_pc = current_jit_state.java_pc;
    let array_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 2);
    let layout: ArrayMemoryLayout = current_jit_state.memory_layout.operand_stack_entry_array_layout(java_pc, 2);
    let index_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 1);
    let value_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 0);

    array_bounds_check(current_jit_state, array_operand, &layout, index_operand)?;
    let final_address = array_final_address(current_jit_state, &size, array_operand, layout, index_operand)?;
    let store = IRInstruction::StoreAbsolute {
        address_to: final_address,
        size,
        input_offset: value_operand,
    };
    current_jit_state.output.main_block.add_instruction(store, current_jit_state.java_pc);
    Ok(())
}

pub fn array_bounds_check(current_jit_state: &mut JitState, array_operand: FramePointerOffset, layout: &ArrayMemoryLayout, index_operand: FramePointerOffset) -> Result<(), JITError> {
    let java_pc = current_jit_state.java_pc;
    let zero = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let load_zero = IRInstruction::Constant { output_offset: zero.clone(), constant: Constant::Int(0) };
    current_jit_state.output.main_block.add_instruction(load_zero, current_jit_state.java_pc);
    let (exception_block, excpetion_block_label) = array_out_of_bounds_block(current_jit_state, index_operand)?;
    current_jit_state.output.add_block(exception_block);
    let branch_if_zero_or_less = IRInstruction::BranchIfComparison {
        offset_a: index_operand,
        offset_b: zero,
        size: Size::Int,
        to: excpetion_block_label,
        branch_type: BranchType::Less,
    };
    current_jit_state.output.main_block.add_instruction(branch_if_zero_or_less, current_jit_state.java_pc);
    let length_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let length_offset = IRInstruction::Constant {
        output_offset: length_location,
        constant: Constant::Pointer(layout.len_entry()),
    };
    current_jit_state.output.main_block.add_instruction(length_offset, current_jit_state.java_pc);
    let length_offset_add = IRInstruction::IntegerArithmetic {
        input_offset_a: length_location,
        input_offset_b: array_operand,
        output_offset: length_location,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::Add,
    };
    current_jit_state.output.main_block.add_instruction(length_offset_add, current_jit_state.java_pc);
    let length = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let length_load = IRInstruction::LoadAbsolute {
        address_from: length_location,
        output_offset: length,
        size: Size::Int,
    };
    current_jit_state.output.main_block.add_instruction(length_load, current_jit_state.java_pc);
    let length_branch_if_index_too_big = IRInstruction::BranchIfComparison {
        offset_a: index_operand,
        offset_b: length,
        size: Size::Int,
        to: excpetion_block_label,
        branch_type: BranchType::MoreEqual,
    };
    current_jit_state.output.main_block.add_instruction(length_branch_if_index_too_big, current_jit_state.java_pc);
    Ok(())
}

pub fn array_load(current_jit_state: &mut JitState, size: Size) -> Result<(), JITError> {
    // array, i
    let java_pc = current_jit_state.java_pc;
    let array_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 1);
    let layout: ArrayMemoryLayout = current_jit_state.memory_layout.operand_stack_entry_array_layout(java_pc, 1);
    let index_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 0);

    array_bounds_check(current_jit_state, array_operand, &layout, index_operand)?;
    let final_address = array_final_address(current_jit_state, &size, array_operand, layout, index_operand)?;
    let load = IRInstruction::LoadAbsolute {
        address_from: final_address,
        output_offset: current_jit_state.memory_layout.operand_stack_entry(current_jit_state.next_pc.unwrap().get() as u16, 0),
        size,
    };
    current_jit_state.output.main_block.add_instruction(load, current_jit_state.java_pc);
    Ok(())
}

pub fn array_final_address(current_jit_state: &mut JitState, size: &Size, array_operand: FramePointerOffset, layout: ArrayMemoryLayout, index_operand: FramePointerOffset) -> Result<FramePointerOffset, JITError> {
    let java_pc = current_jit_state.java_pc;
    let shift_amount = match size {
        Size::Byte => Constant::Long(size_of::<jbyte>() as i64),
        Size::Short => Constant::Long(size_of::<jshort>() as i64),
        Size::Int => Constant::Long(size_of::<jint>() as i64),
        Size::Long => Constant::Long(size_of::<jlong>() as i64)
    };
    let shift_constant_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let shift_amount = IRInstruction::Constant { output_offset: shift_constant_location, constant: shift_amount };
    current_jit_state.output.main_block.add_instruction(shift_amount, current_jit_state.java_pc);
    let shift_instruction = IRInstruction::IntegerArithmetic {
        input_offset_a: index_operand,
        input_offset_b: shift_constant_location,
        output_offset: index_operand,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::LeftShift,
    };
    current_jit_state.output.main_block.add_instruction(shift_instruction, current_jit_state.java_pc);
    let base_offset = layout.elem_0_entry();
    let base_offset_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 1);
    let base_offset_instruction = IRInstruction::Constant {
        output_offset: base_offset_location,
        constant: Constant::Long(base_offset as i64),
    };
    current_jit_state.output.main_block.add_instruction(base_offset_instruction, current_jit_state.java_pc);
    let base_offset_add = IRInstruction::IntegerArithmetic {
        input_offset_a: base_offset_location,
        input_offset_b: array_operand,
        output_offset: array_operand,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::Add,
    };
    current_jit_state.output.main_block.add_instruction(base_offset_add, current_jit_state.java_pc);
    let index_add = IRInstruction::IntegerArithmetic {
        input_offset_a: array_operand,
        input_offset_b: index_operand,
        output_offset: array_operand,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::Add,
    };
    current_jit_state.output.main_block.add_instruction(index_add, current_jit_state.java_pc);
    Ok(array_operand)
}
