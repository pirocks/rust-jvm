use jit_ir::{ArithmeticType, Constant, IRInstruction, Size};

use crate::{JitBlock, JITError, JitState};

pub enum ShiftDirection {
    ArithmeticLeft,
    LogicalLeft,
    ArithmeticRight,
    LogicalRight,
}

impl ShiftDirection {
    pub fn signed(&self) -> bool {
        match self {
            ShiftDirection::ArithmeticLeft => true,
            ShiftDirection::LogicalLeft => false,
            ShiftDirection::ArithmeticRight => true,
            ShiftDirection::LogicalRight => false,
        }
    }

    pub fn to_arithmetic_type(&self) -> ArithmeticType {
        match self {
            ShiftDirection::ArithmeticLeft => ArithmeticType::LeftShift,
            ShiftDirection::LogicalLeft => ArithmeticType::LeftShift,
            ShiftDirection::ArithmeticRight => ArithmeticType::RightShift,
            ShiftDirection::LogicalRight => ArithmeticType::RightShift,
        }
    }
}

pub fn shift(current_jit_state: &mut JitState, main_block: &mut JitBlock, java_pc: usize, size: Size, shift_direction: ShiftDirection) -> Result<(), JITError> {
    let mask = current_jit_state.memory_layout.safe_temp_location(java_pc, 0);
    let value_to_shift = current_jit_state.memory_layout.operand_stack_entry(java_pc, 1);
    let amount_to_shift_by = current_jit_state.memory_layout.operand_stack_entry(java_pc, 0);

    let mask_constant = IRInstruction::Constant {
        output_offset: mask,
        constant: match size {
            Size::Byte => panic!(),
            Size::Short => panic!(),
            Size::Int => Constant::Int(0x1f),
            Size::Long => Constant::Int(0x3f)
        },
    };
    main_block.add_instruction(mask_constant);
    let mask_shift_value = IRInstruction::IntegerArithmetic {
        input_offset_a: value_to_shift,
        input_offset_b: mask,
        output_offset: value_to_shift,
        size: Size::Byte,
        signed: false,
        arithmetic_type: ArithmeticType::BinaryAnd,
    };
    main_block.add_instruction(mask_shift_value);
    let copy_int_to_long = IRInstruction::CopyRelative {
        input_offset: amount_to_shift_by,
        output_offset: amount_to_shift_by,
        input_size: Size::Int,
        output_size: Size::Long,
        signed: true,
    };
    main_block.add_instruction(copy_int_to_long);
    let instruct = IRInstruction::IntegerArithmetic {
        input_offset_a: value_to_shift,
        input_offset_b: amount_to_shift_by,
        output_offset: current_jit_state.memory_layout.operand_stack_entry(current_jit_state.next_pc(), 1),
        size,
        signed: shift_direction.signed(),
        arithmetic_type: shift_direction.to_arithmetic_type(),
    };
    main_block.add_instruction(instruct);
    Ok(())
}


pub fn arithmetic_common(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size, atype: ArithmeticType) -> Result<(), JITError> {
    let instruct = IRInstruction::IntegerArithmetic {
        input_offset_a: current_jit_state.memory_layout.operand_stack_entry(current_jit_state.java_pc, 1),
        input_offset_b: current_jit_state.memory_layout.operand_stack_entry(current_jit_state.java_pc, 0),
        output_offset: current_jit_state.memory_layout.operand_stack_entry(current_jit_state.next_pc(), 0),
        size,
        signed: false,
        arithmetic_type: atype,
    };
    main_block.add_instruction(instruct);
    Ok(())
}

pub fn integer_mul(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::Mul)
}


pub fn integer_div(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::Div)
}


pub fn integer_sub(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::Sub)
}

pub fn integer_add(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::Add)
}

pub fn binary_or(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::BinaryOr)
}

pub fn binary_xor(current_jit_state: &mut JitState, main_block: &mut JitBlock, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::BinaryXor)
}

pub fn binary_and(current_jit_state: &mut JitState, size: Size) -> Result<(), JITError> {
    arithmetic_common(current_jit_state, main_block, size, ArithmeticType::BinaryAnd)
}
