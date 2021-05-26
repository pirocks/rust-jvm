use std::mem::size_of;

use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset};
use jit_ir::{ArithmeticType, BranchType, Constant, IRInstruction, IRLabel, Size, VMExitType};

use crate::{JitBlock, JITError, JitState};

fn array_out_of_bounds_block(current_jit_state: &mut JitState, _index_offset: FramePointerOffset) -> Result<(JitBlock, IRLabel), JITError> {
    let mut block = JitBlock {
        java_pc_to_ir: Default::default(),
        instructions: vec![],
    };
    let label = current_jit_state.new_ir_label();
    block.add_instruction(IRInstruction::Label(label));
    block.add_instruction(IRInstruction::VMExit(VMExitType::ArrayOutOfBounds));
    Ok((block, label))
}


fn array_store(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    // array, i, val
    let array_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 2);
    let layout: ArrayMemoryLayout = current_jit_state.memory_layout.operand_stack_entry_array_layout(java_pc, 2);
    let index_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 1);
    let value_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 0);

    array_bounds_check(current_jit_state, main_block, array_operand, &layout, index_operand)?;
    let final_address = array_final_address(current_jit_state, main_block, &size, array_operand, layout, index_operand)?;
    let store = IRInstruction::StoreAbsolute {
        address_to: final_address,
        size,
        input_offset: value_operand,
    };
    main_block.add_instruction(store);
    Ok(())
}

fn array_bounds_check(current_jit_state: &mut JitState, main_block: &mut JitBlock, array_operand: FramePointerOffset, layout: &ArrayMemoryLayout, index_operand: FramePointerOffset) -> Result<(), JITError> {
    let zero = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let load_zero = IRInstruction::Constant { output_offset: zero.clone(), constant: Constant::Int(0) };
    main_block.add_instruction(load_zero);
    let (exception_block, excpetion_block_label) = array_out_of_bounds_block(current_jit_state, index_operand)?;
    current_jit_state.output.add_block(exception_block);
    let branch_if_zero_or_less = IRInstruction::BranchIfComparison {
        offset_a: index_operand,
        offset_b: zero,
        size: Size::Int,
        to: excpetion_block_label,
        branch_type: BranchType::Less,
    };
    main_block.add_instruction(branch_if_zero_or_less);
    let length_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let length_offset = IRInstruction::Constant {
        output_offset: length_location,
        constant: Constant::Pointer(layout.len_entry()),
    };
    main_block.add_instruction(length_offset);
    let length_offset_add = IRInstruction::IntegerArithmetic {
        input_offset_a: length_location,
        input_offset_b: array_operand,
        output_offset: length_location,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::Add,
    };
    main_block.add_instruction(length_offset_add);
    let length = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let length_load = IRInstruction::LoadAbsolute {
        address_from: length_location,
        output_offset: length,
        size: Size::Int,
    };
    main_block.add_instruction(length_load);
    let length_branch_if_index_too_big = IRInstruction::BranchIfComparison {
        offset_a: index_operand,
        offset_b: length,
        size: Size::Int,
        to: excpetion_block_label,
        branch_type: BranchType::MoreEqual,
    };
    main_block.add_instruction(length_branch_if_index_too_big);
    Ok(())
}

fn array_load(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    // array, i
    let array_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 1);
    let layout: ArrayMemoryLayout = current_jit_state.memory_layout.operand_stack_entry_array_layout(java_pc, 1);
    let index_operand = current_jit_state.memory_layout.operand_stack_entry(java_pc, 0);

    array_bounds_check(current_jit_state, main_block, array_operand, &layout, index_operand)?;
    let final_address = array_final_address(current_jit_state, main_block, &size, array_operand, layout, index_operand)?;
    let load = IRInstruction::LoadAbsolute {
        address_from: final_address,
        output_offset: current_jit_state.memory_layout.operand_stack_entry(current_jit_state.next_pc.unwrap().get(), 0),
        size,
    };
    main_block.add_instruction(load);
    Ok(())
}

fn array_final_address(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: &Size, array_operand: FramePointerOffset, layout: ArrayMemoryLayout, index_operand: FramePointerOffset) -> Result<FramePointerOffset, JITError> {
    let shift_amount = match size {
        Size::Byte => Constant::Long(size_of::<jbyte>() as i64),
        Size::Short => Constant::Long(size_of::<jshort>() as i64),
        Size::Int => Constant::Long(size_of::<jint>() as i64),
        Size::Long => Constant::Long(size_of::<jlong>() as i64)
    };
    let shift_amount = IRInstruction::Constant { output_offset: shift_constant_location, constant: shift_amount };
    main_block.add_instruction(shift_amount);
    let shift_instruction = IRInstruction::IntegerArithmetic {
        input_offset_a: index_operand,
        input_offset_b: shift_constant_location,
        output_offset: index_operand,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::LeftShift,
    };
    main_block.add_instruction(shift_instruction);
    let base_offset = layout.elem_0_entry();
    let base_offset_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 1);
    let base_offset_instruction = IRInstruction::Constant {
        output_offset: base_offset_location,
        constant: Constant::Long(base_offset as i64),
    };
    main_block.add_instruction(base_offset_instruction);
    let base_offset_add = IRInstruction::IntegerArithmetic {
        input_offset_a: base_offset_location,
        input_offset_b: array_operand,
        output_offset: array_operand,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::Add,
    };
    main_block.add_instruction(base_offset_add);
    let index_add = IRInstruction::IntegerArithmetic {
        input_offset_a: array_operand,
        input_offset_b: index_operand,
        output_offset: array_operand,
        size: Size::Long,
        signed: false,
        arithmetic_type: ArithmeticType::Add,
    };
    main_block.add_instruction(index_add);
    Ok(array_operand)
}
