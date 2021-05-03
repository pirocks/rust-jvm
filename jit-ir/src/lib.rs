use iced_x86::{Code, Instruction, MemoryOperand, Register};

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
    Pointer(usize)
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
    **/
    pub fn to_x86(&self, instructions: &mut Vec<Instruction>) {
        match self {
            IRInstruction::LoadAbsolute { address_from, output_offset, size } => {
                let mem = MemoryOperand::with_base_displ(Register::RBP, address_from.0 as i64);
                let load_address = Instruction::with_reg_mem(Code::Movq_rm64_mm, Register::RAX, mem);
                let load_value = match size {
                    Size::Byte => todo!(),
                    Size::Short => todo!(),
                    Size::Int => Instruction::with_reg_mem(Code::Movd_rm32_mm, Register::EBX, MemoryOperand::with_base(Register::RAX)),
                    Size::Long => Instruction::with_reg_mem(Code::Movq_rm64_mm, Register::RBX, MemoryOperand::with_base(Register::RAX))
                };
                // let write_value = Instruction::with_reg_mem();
                instructions.push(load_address);
                instructions.push(load_value);
                // instructions.push(write_value)
            }
            IRInstruction::StoreAbsolute { .. } => {}
            IRInstruction::CopyRelative { .. } => {}
            IRInstruction::IntegerArithmetic { .. } => {}
            IRInstruction::BranchUnConditional(_) => {}
            IRInstruction::BranchIf0 { .. } => {}
            IRInstruction::VMExit(_) => {}
            IRInstruction::StoreConstant { .. } => {}
            IRInstruction::Return { .. } => {}
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