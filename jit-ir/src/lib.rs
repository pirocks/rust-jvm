use iced_x86::{Code, Instruction, MemoryOperand, Register};
use iced_x86::CodeSize::Code16;

use gc_memory_layout_common::FramePointerOffset;

pub struct RelativeAddress(isize);

pub enum Size {
    Byte,
    Short,
    Int,
    Long,
}


pub struct VariableSize(pub usize);

pub enum ArithmeticType {
    Add,
    Sub,
    Mul,
    Div,
    BinaryAnd,
    BinaryOr,
    BinaryXor,
    LeftShift,
    RightShift,
    Rotate,

}

pub enum VMExitType {}

pub enum Constant {
    Pointer(usize),
    Double(f64),
    Float(f32),
    Int(i32),
    Short(i16),
    Byte(i8),
}

pub enum IRInstruction {
    LoadAbsolute {
        address_from: FramePointerOffset,
        output_offset: FramePointerOffset,
        size: Size,
    },
    StoreAbsolute {
        address_to: FramePointerOffset,
        input_offset: FramePointerOffset,
        size: Size,
    },
    StoreConstant {
        address_to: FramePointerOffset,
        constant: Constant,
    },
    CopyRelative {
        input_offset: FramePointerOffset,
        output_offset: FramePointerOffset,
        size: Size,
    },
    IntegerArithmetic {
        input_offset_a: FramePointerOffset,
        input_offset_b: FramePointerOffset,
        output_offset: FramePointerOffset,
        size: Size,
        signed: bool,
        arithmetic_type: ArithmeticType,
    },
    BranchUnConditional(RelativeAddress),
    BranchIf0 {
        offset: FramePointerOffset,
        size: Size,
    },
    Return {
        return_value: Option<FramePointerOffset>,
        to_pop: VariableSize,
    },
    VMExit(VMExitType),
}

impl IRInstruction {
    /*
rax
rbx
rcx
rdx
are reserved for temp in instructions
r15 is reserved for context pointer
    **/
    pub fn to_x86(&self, instructions: &mut Vec<Instruction>) {
        match self {
            IRInstruction::LoadAbsolute { address_from, output_offset, size } => {
                let load_address_mem_operand = MemoryOperand::with_base_displ(Register::RBP, address_from.0 as i64);
                let load_address = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RAX, load_address_mem_operand);
                let address = Register::RAX;
                let load_value = match size {
                    Size::Byte => Instruction::with_reg_mem(Code::Mov_r8_rm8, Register::BL, MemoryOperand::with_base(address)),
                    Size::Short => Instruction::with_reg_mem(Code::Mov_r16_rm16, Register::BX, MemoryOperand::with_base(address)),
                    Size::Int => Instruction::with_reg_mem(Code::Mov_r32_rm32, Register::EBX, MemoryOperand::with_base(address)),
                    Size::Long => Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RBX, MemoryOperand::with_base(address))
                };
                let write_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
                let write_value = match size {
                    Size::Byte => Instruction::with_mem_reg(Code::Mov_rm8_r8, write_memory_operand, Register::BL),
                    Size::Short => Instruction::with_mem_reg(Code::Mov_rm16_r16, write_memory_operand, Register::BX),
                    Size::Int => Instruction::with_mem_reg(Code::Mov_rm32_r32, write_memory_operand, Register::EBX),
                    Size::Long => Instruction::with_mem_reg(Code::Mov_rm64_r64, write_memory_operand, Register::RBX)
                };
                instructions.push(load_address);
                instructions.push(load_value);
                instructions.push(write_value)
            }
            IRInstruction::StoreAbsolute { address_to, input_offset, size } => {
                let write_address_mem_operand = MemoryOperand::with_base_displ(Register::RBP, address_to.0 as i64);
                let write_address_load = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RAX, write_address_mem_operand);
                let input_load_memory_operand = MemoryOperand::with_base_displ(Register::RBP, input_offset.0 as i64);
                let load_value = match size {
                    Size::Byte => Instruction::with_reg_mem(Code::Mov_rm8_r8, Register::BL, input_load_memory_operand),
                    Size::Short => Instruction::with_reg_mem(Code::Mov_rm16_r16, Register::BX, input_load_memory_operand),
                    Size::Int => Instruction::with_reg_mem(Code::Mov_rm32_r32, Register::EBX, input_load_memory_operand),
                    Size::Long => Instruction::with_reg_mem(Code::Mov_rm64_r64, Register::RBX, input_load_memory_operand)
                };

                let write_value = match size {
                    Size::Byte => Instruction::with_mem_reg(Code::Mov_rm8_r8, MemoryOperand::with_base(Register::RAX), Register::BL),
                    Size::Short => Instruction::with_mem_reg(Code::Mov_rm16_r16, MemoryOperand::with_base(Register::RAX), Register::BX),
                    Size::Int => Instruction::with_mem_reg(Code::Mov_rm32_r32, MemoryOperand::with_base(Register::RAX), Register::EBX),
                    Size::Long => Instruction::with_mem_reg(Code::Mov_rm64_r64, MemoryOperand::with_base(Register::RAX), Register::RBX)
                };
                instructions.push(write_address_load);
                instructions.push(load_value);
                instructions.push(write_value);
            }
            IRInstruction::CopyRelative { size, input_offset, output_offset } => {
                let input_load_memory_operand = MemoryOperand::with_base_displ(Register::RBP, input_offset.0 as i64);
                let load_value = match size {
                    Size::Byte => Instruction::with_reg_mem(Code::Mov_rm8_r8, Register::BL, input_load_memory_operand),
                    Size::Short => Instruction::with_reg_mem(Code::Mov_rm16_r16, Register::BX, input_load_memory_operand),
                    Size::Int => Instruction::with_reg_mem(Code::Mov_rm32_r32, Register::EBX, input_load_memory_operand),
                    Size::Long => Instruction::with_reg_mem(Code::Mov_rm64_r64, Register::RBX, input_load_memory_operand)
                };
                let write_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
                let write_value = match size {
                    Size::Byte => Instruction::with_mem_reg(Code::Mov_rm8_r8, write_memory_operand, Register::BL),
                    Size::Short => Instruction::with_mem_reg(Code::Mov_rm16_r16, write_memory_operand, Register::BX),
                    Size::Int => Instruction::with_mem_reg(Code::Mov_rm32_r32, write_memory_operand, Register::EBX),
                    Size::Long => Instruction::with_mem_reg(Code::Mov_rm64_r64, write_memory_operand, Register::RBX)
                };
                instructions.push(load_value);
                instructions.push(write_value);
            },
            IRInstruction::IntegerArithmetic { .. } => todo!(),
            IRInstruction::BranchUnConditional(_) => todo!(),
            IRInstruction::BranchIf0 { .. } => todo!(),
            IRInstruction::VMExit(_) => todo!(),
            IRInstruction::StoreConstant { .. } => todo!(),
            IRInstruction::Return { .. } => todo!(),
        }
    }
}

#[cfg(test)]
pub mod test {
    use iced_x86::{Formatter, Instruction, IntelFormatter};

    use crate::{FramePointerOffset, IRInstruction, Size};

    #[test]
    pub fn test() {}
}