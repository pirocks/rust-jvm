use iced_x86::{Code, Instruction, MemoryOperand, Register};
use iced_x86::CodeSize::Code16;

use gc_memory_layout_common::FramePointerOffset;

pub struct RelativeAddress(isize);

#[derive(Clone, Copy)]
#[derive(Eq, PartialEq)]
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
    RotateRight,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum VMExitType {
    CheckCast,
    InstanceOf,
    Throw,
    InvokeDynamic,
    InvokeStatic,
    InvokeVirtual,
    InvokeSpecial,
    InvokeInterface,
    MonitorEnter,
    MonitorExit,
    MultiNewArray,
    ArrayOutOfBounds,

}

pub enum Constant {
    Pointer(usize),
    Double(f64),
    Float(f32),
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
}

#[derive(Clone, Copy)]
pub struct IRLabel {
    id: usize,
}

pub enum BranchType0 {
    Equal0,
    Less0,
    More0,
    LessEqual0,
    MoreEqual0,
}

pub enum BranchType {
    Equal,
    Less,
    More,
    LessEqual,
    MoreEqual,
}

pub enum BranchTypeFloat {
    EqualFloat,
    LessFloat,
    MoreFloat,
    LessEqualFloat,
    MoreEqualFloat,
}

pub enum FloatSize {
    Float,
    Double,
}

pub enum FloatArithmeticType {
    Add,
    Sub,
    Mul,
    Div,
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
    Constant {
        output_offset: FramePointerOffset,
        constant: Constant,
    },
    CopyRelative {
        input_offset: FramePointerOffset,
        output_offset: FramePointerOffset,
        input_size: Size,
        output_size: Size,
        signed: bool,
    },
    IntegerArithmetic {
        input_offset_a: FramePointerOffset,
        input_offset_b: FramePointerOffset,
        output_offset: FramePointerOffset,
        size: Size,
        signed: bool,
        arithmetic_type: ArithmeticType,
    },
    FloatArithmetic {
        input_offset_a: FramePointerOffset,
        input_offset_b: FramePointerOffset,
        output_offset: FramePointerOffset,
        size: FloatSize,
        signed: bool,
        arithmetic_type: FloatArithmeticType,
    },
    BranchUnConditional(IRLabel),
    BranchIf {
        offset: FramePointerOffset,
        size: Size,
        to: IRLabel,
        branch_type: BranchType0,
    },
    BranchIfComparison {
        offset_a: FramePointerOffset,
        offset_b: FramePointerOffset,
        size: Size,
        to: IRLabel,
        branch_type: BranchType,
    },
    Return {
        return_value: Option<FramePointerOffset>,
        to_pop: VariableSize,
    },
    VMExit(VMExitType),
    Label(IRLabel),
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
            IRInstruction::CopyRelative { input_offset, output_offset, signed, input_size, output_size } => {
                let input_load_memory_operand = MemoryOperand::with_base_displ(Register::RBP, input_offset.0 as i64);

                let load_to_register = match output_size {
                    Size::Byte => Register::BL,
                    Size::Short => Register::BX,
                    Size::Int => Register::EBX,
                    Size::Long => if !*signed && input_size == Size::Long { Register::EBX } else { Register::RBX }
                };
                let opcode = if signed {
                    match input_size {
                        Size::Byte => match output_size {
                            Size::Byte => Code::Mov_r8_rm8,
                            Size::Short => Code::Movsx_r16_rm8,
                            Size::Int => Code::Movsx_r32_rm8,
                            Size::Long => Code::Movsx_r64_rm8,
                        }
                        Size::Short => match output_size {
                            Size::Byte => todo!("no easy way to do narrowing signed copy,and unsure if makes sense anyway"),
                            Size::Short => Code::Movsx_r16_rm16,
                            Size::Int => Code::Movsx_r32_rm16,
                            Size::Long => Code::Movsx_r64_rm16,
                        }
                        Size::Int => match output_size {
                            Size::Byte => todo!("no easy way to do narrowing signed copy,and unsure if makes sense anyway"),
                            Size::Short => todo!("no easy way to do narrowing signed copy,and unsure if makes sense anyway"),
                            Size::Int => Code::Mov_r32_rm32,
                            Size::Long => Code::Movsxd_r64_rm32,
                        }
                        Size::Long => match output_size {
                            Size::Byte => todo!("no easy way to do narrowing signed copy,and unsure if makes sense anyway"),
                            Size::Short => todo!("no easy way to do narrowing signed copy,and unsure if makes sense anyway"),
                            Size::Int => todo!("no easy way to do narrowing signed copy,and unsure if makes sense anyway"),
                            Size::Long => Code::Mov_r64_rm64
                        }
                    }
                } else {
                    match input_size {
                        Size::Byte => match output_size {
                            Size::Byte => Code::Mov_r8_rm8,
                            Size::Short => Code::Movzx_r16_rm8,
                            Size::Int => Code::Movzx_r32_rm8,
                            Size::Long => Code::Movzx_r64_rm8,
                        }
                        Size::Short => match output_size {
                            Size::Byte => Code::Mov_r8_rm8,
                            Size::Short => Code::Mov_r16_rm16,
                            Size::Int => Code::Movzx_r32_rm16,
                            Size::Long => Code::Movzx_r64_rm16,
                        }
                        Size::Int => match output_size {
                            Size::Byte => Code::Mov_r8_rm8,
                            Size::Short => Code::Mov_r16_rm16,
                            Size::Int => Code::Mov_r32_rm32,
                            Size::Long => Code::Mov_r32_rm32,//todo need registers to match here
                        }
                        Size::Long => match output_size {
                            Size::Byte => Code::Mov_r8_rm8,
                            Size::Short => Code::Mov_r16_rm16,
                            Size::Int => Code::Mov_r32_rm32,
                            Size::Long => Code::Mov_r64_rm64,
                        }
                    }
                };
                let load_value = Instruction::with_reg_mem(opcode, load_to_register, input_load_memory_operand);
                let write_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
                let write_value = match output_size {
                    Size::Byte => Instruction::with_mem_reg(Code::Mov_rm8_r8, write_memory_operand, Register::BL),
                    Size::Short => Instruction::with_mem_reg(Code::Mov_rm16_r16, write_memory_operand, Register::BX),
                    Size::Int => Instruction::with_mem_reg(Code::Mov_rm32_r32, write_memory_operand, Register::EBX),
                    Size::Long => Instruction::with_mem_reg(Code::Mov_rm64_r64, write_memory_operand, Register::RBX)
                };
                instructions.push(load_value);
                instructions.push(write_value);
            }
            IRInstruction::IntegerArithmetic { input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type } => {
                let input_load_memory_operand_a = MemoryOperand::with_base_displ(Register::RBP, input_offset_a.0 as i64);
                let load_value_a = match size {
                    Size::Byte => Instruction::with_reg_mem(Code::Mov_rm8_r8, Register::AL, input_load_memory_operand_a),
                    Size::Short => Instruction::with_reg_mem(Code::Mov_rm16_r16, Register::AX, input_load_memory_operand_a),
                    Size::Int => Instruction::with_reg_mem(Code::Mov_rm32_r32, Register::EAX, input_load_memory_operand_a),
                    Size::Long => Instruction::with_reg_mem(Code::Mov_rm64_r64, Register::RAX, input_load_memory_operand_a)
                };
                instructions.push(load_value_a);
                let input_load_memory_operand_b = MemoryOperand::with_base_displ(Register::RBP, input_offset_b.0 as i64);
                let load_value_b = match size {
                    Size::Byte => Instruction::with_reg_mem(Code::Mov_rm8_r8, Register::BL, input_load_memory_operand_b.clone()),
                    Size::Short => Instruction::with_reg_mem(Code::Mov_rm16_r16, Register::BX, input_load_memory_operand_b.clone()),
                    Size::Int => Instruction::with_reg_mem(Code::Mov_rm32_r32, Register::EBX, input_load_memory_operand_b.clone()),
                    Size::Long => Instruction::with_reg_mem(Code::Mov_rm64_r64, Register::RBX, input_load_memory_operand_b.clone())
                };
                instructions.push(load_value_b);
                let arithmetic = match arithmetic_type {
                    ArithmeticType::Add => {
                        match size {
                            Size::Byte => Instruction::with_reg_mem(Code::Add_r8_rm8, Register::AL, input_load_memory_operand_b),
                            Size::Short => Instruction::with_reg_mem(Code::Add_r16_rm16, Register::AX, input_load_memory_operand_b),
                            Size::Int => Instruction::with_reg_mem(Code::Add_r32_rm32, Register::EAX, input_load_memory_operand_b),
                            Size::Long => Instruction::with_reg_mem(Code::Add_r64_rm64, Register::RAX, input_load_memory_operand_b)
                        }
                    }
                    ArithmeticType::Sub => {
                        match size {
                            Size::Byte => Instruction::with_reg_mem(Code::Sub_r8_rm8, Register::AL, input_load_memory_operand_b),
                            Size::Short => Instruction::with_reg_mem(Code::Sub_r16_rm16, Register::AX, input_load_memory_operand_b),
                            Size::Int => Instruction::with_reg_mem(Code::Sub_r32_rm32, Register::EAX, input_load_memory_operand_b),
                            Size::Long => Instruction::with_reg_mem(Code::Sub_r64_rm64, Register::RAX, input_load_memory_operand_b)
                        }
                    }
                    ArithmeticType::Mul => {
                        match size {
                            Size::Byte => Instruction::with_mem(if *signed { Code::Imul_rm8 } else { Code::Mul_rm8 }, input_load_memory_operand_b),
                            Size::Short => Instruction::with_mem(if *signed { Code::Imul_rm16 } else { Code::Mul_rm16 }, input_load_memory_operand_b),
                            Size::Int => Instruction::with_mem(if *signed { Code::Imul_rm32 } else { Code::Mul_rm32 }, input_load_memory_operand_b),
                            Size::Long => Instruction::with_mem(if *signed { Code::Imul_rm64 } else { Code::Mul_rm64 }, input_load_memory_operand_b)
                        }
                        //result now in a
                    }
                    ArithmeticType::Div => {
                        match size {
                            Size::Byte => Instruction::with_mem(if *signed { Code::Idiv_rm8 } else { Code::Div_rm8 }, input_load_memory_operand_b),
                            Size::Short => Instruction::with_mem(if *signed { Code::Idiv_rm16 } else { Code::Div_rm16 }, input_load_memory_operand_b),
                            Size::Int => Instruction::with_mem(if *signed { Code::Idiv_rm32 } else { Code::Div_rm32 }, input_load_memory_operand_b),
                            Size::Long => Instruction::with_mem(if *signed { Code::Idiv_rm64 } else { Code::Div_rm64 }, input_load_memory_operand_b)
                        }
                    }
                    ArithmeticType::BinaryAnd => {
                        match size {
                            Size::Byte => Instruction::with_reg_mem(Code::And_r8_rm8, Register::AL, input_load_memory_operand_b),
                            Size::Short => Instruction::with_reg_mem(Code::And_r16_rm16, Register::AX, input_load_memory_operand_b),
                            Size::Int => Instruction::with_reg_mem(Code::And_r32_rm32, Register::EAX, input_load_memory_operand_b),
                            Size::Long => Instruction::with_reg_mem(Code::And_r64_rm64, Register::RAX, input_load_memory_operand_b)
                        }
                    }
                    ArithmeticType::BinaryOr => {
                        match size {
                            Size::Byte => Instruction::with_reg_mem(Code::Or_r8_rm8, Register::AL, input_load_memory_operand_b),
                            Size::Short => Instruction::with_reg_mem(Code::Or_r16_rm16, Register::AX, input_load_memory_operand_b),
                            Size::Int => Instruction::with_reg_mem(Code::Or_r32_rm32, Register::EAX, input_load_memory_operand_b),
                            Size::Long => Instruction::with_reg_mem(Code::Or_r64_rm64, Register::RAX, input_load_memory_operand_b)
                        }
                    }
                    ArithmeticType::BinaryXor => {
                        match size {
                            Size::Byte => Instruction::with_reg_mem(Code::Xor_r8_rm8, Register::AL, input_load_memory_operand_b),
                            Size::Short => Instruction::with_reg_mem(Code::Xor_r16_rm16, Register::AX, input_load_memory_operand_b),
                            Size::Int => Instruction::with_reg_mem(Code::Xor_r32_rm32, Register::EAX, input_load_memory_operand_b),
                            Size::Long => Instruction::with_reg_mem(Code::Xor_r64_rm64, Register::RAX, input_load_memory_operand_b)
                        }
                    }
                    ArithmeticType::LeftShift => {
                        match size {
                            Size::Byte => Instruction::with_reg_reg(if *signed { Code::Sal_rm8_CL } else { Code::Shl_rm8_CL }, Register::AL, Register::BL),
                            Size::Short => Instruction::with_reg_reg(if *signed { Code::Sal_rm16_CL } else { Code::Shl_rm16_CL }, Register::AX, Register::AL),
                            Size::Int => Instruction::with_reg_reg(if *signed { Code::Sal_rm32_CL } else { Code::Shl_rm32_CL }, Register::EAX, Register::EBX),
                            Size::Long => Instruction::with_reg_reg(if *signed { Code::Sal_rm64_CL } else { Code::Shl_rm64_CL }, Register::RAX, Register::RBX),
                        }
                    }
                    ArithmeticType::RightShift => {
                        match size {
                            Size::Byte => Instruction::with_reg_reg(if *signed { Code::Sar_rm8_CL } else { Code::Shr_rm8_CL }, Register::AL, Register::BL),
                            Size::Short => Instruction::with_reg_reg(if *signed { Code::Sar_rm16_CL } else { Code::Shr_rm16_CL }, Register::AX, Register::AL),
                            Size::Int => Instruction::with_reg_reg(if *signed { Code::Sar_rm32_CL } else { Code::Shr_rm32_CL }, Register::EAX, Register::EBX),
                            Size::Long => Instruction::with_reg_reg(if *signed { Code::Sar_rm64_CL } else { Code::Shr_rm64_CL }, Register::RAX, Register::RBX),
                        }
                    }
                    ArithmeticType::RotateRight => {
                        match size {
                            Size::Byte => Instruction::with_reg_reg(if *signed { todo!() } else { Code::Ror_rm8_CL }, Register::AL, Register::BL),
                            Size::Short => Instruction::with_reg_reg(if *signed { todo!() } else { Code::Ror_rm16_CL }, Register::AX, Register::BX),
                            Size::Int => Instruction::with_reg_reg(if *signed { todo!() } else { Code::Ror_rm32_CL }, Register::EAX, Register::EBX),
                            Size::Long => Instruction::with_reg_reg(if *signed { todo!() } else { Code::Ror_rm64_CL }, Register::RAX, Register::RBX),
                        }
                    }
                };
                let output_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
                let write_result = match size {
                    Size::Byte => Instruction::with_mem_reg(Code::Mov_r8_rm8, output_memory_operand, Register::AL),
                    Size::Short => Instruction::with_mem_reg(Code::Mov_r16_rm16, output_memory_operand, Register::AX),
                    Size::Int => Instruction::with_mem_reg(Code::Mov_r32_rm32, output_memory_operand, Register::EAX),
                    Size::Long => Instruction::with_mem_reg(Code::Mov_r64_rm64, output_memory_operand, Register::RAX),
                };
                instructions.push(write_result);
            }
            IRInstruction::BranchUnConditional(_) => todo!(),
            IRInstruction::VMExit(_) => todo!(),
            IRInstruction::Constant { output_offset, constant } => {
                let output_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
                let instruct = match constant {
                    Constant::Pointer(ptr) => {
                        todo!()
                    }
                    Constant::Double(_) => {
                        todo!()
                    }
                    Constant::Float(flt) => {
                        todo!()
                    }
                    Constant::Long(_) => {
                        todo!()
                    }
                    Constant::Int(_) => {
                        todo!()
                    }
                    Constant::Short(_) => {
                        todo!()
                    }
                    Constant::Byte(_) => {
                        todo!()
                    }
                };
            },
            IRInstruction::Return { .. } => todo!(),
            IRInstruction::FloatArithmetic { input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type } => {
                let input_load_memory_operand_a = MemoryOperand::with_base_displ(Register::RBP, input_offset_a.0 as i64);
                let load_value_a = match size {
                    FloatSize::Float => Instruction::with_reg_mem(Code::Movss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_a),
                    FloatSize::Double => Instruction::with_reg_mem(Code::Movsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_a)
                };
                instructions.push(load_value_a);
                let input_load_memory_operand_b = MemoryOperand::with_base_displ(Register::RBP, input_offset_b.0 as i64);
                let load_value_b = match size {
                    FloatSize::Float => Instruction::with_reg_mem(Code::Movss_xmm_xmmm32, Register::XMM1, input_load_memory_operand_b.clone()),
                    FloatSize::Double => Instruction::with_reg_mem(Code::Movsd_xmm_xmmm64, Register::XMM1, input_load_memory_operand_b.clone())
                };
                instructions.push(load_value_b);
                let operation = match arithmetic_type {
                    FloatArithmeticType::Add => match size {
                        FloatSize::Float => Instruction::with_reg_mem(Code::Addss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_b),
                        FloatSize::Double => Instruction::with_reg_mem(Code::Addsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_b)
                    }
                    FloatArithmeticType::Sub => match size {
                        FloatSize::Float => Instruction::with_reg_mem(Code::Subss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_b),
                        FloatSize::Double => Instruction::with_reg_mem(Code::Subsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_b),
                    }
                    FloatArithmeticType::Mul => match size {
                        FloatSize::Float => Instruction::with_reg_mem(Code::Mulss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_b),
                        FloatSize::Double => Instruction::with_reg_mem(Code::Mulsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_b)
                    },
                    FloatArithmeticType::Div => match size {
                        FloatSize::Float => Instruction::with_reg_mem(Code::Divss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_b),
                        FloatSize::Double => Instruction::with_reg_mem(Code::Divsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_b),
                    }
                };
                instructions.push(operation);
                let output_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
                let write_res = match size {
                    FloatSize::Float => Instruction::with_mem_reg(Code::Movss_xmmm32_xmm, output_memory_operand, Register::XMM0),
                    FloatSize::Double => Instruction::with_mem_reg(Code::Movsd_xmmm64_xmm, output_memory_operand, Register::XMM0)
                };
                instructions.push(write_res)
            },
            IRInstruction::BranchIf { .. } => todo!(),
            IRInstruction::BranchIfComparison { .. } => todo!(),
            IRInstruction::Label(_) => todo!()
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