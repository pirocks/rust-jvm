#[macro_use]
extern crate memoffset;

use std::collections::HashMap;
use std::ffi::c_void;
use std::ops::Range;

use iced_x86::{Code, Instruction, MemoryOperand, Register};
use iced_x86::CodeSize::Code16;
use rangemap::RangeMap;

use gc_memory_layout_common::FramePointerOffset;
use jit_common::{JitCodeContext, VMExitType};
use jit_common::SavedRegisters;

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
    ReturnNone,
    Return {
        return_value: FramePointerOffset,
        return_value_size: Size,
    },
    Call {
        resolved_destination: FramePointerOffset,
        local_var_and_operand_stack_size: FramePointerOffset,
        return_location: Option<FramePointerOffset>,
    },
    VMExit(VMExitType),
    Label(IRLabel),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct MemoryOffset(usize);

impl MemoryOffset {
    pub fn to_absolute(&self, base: *mut c_void) -> AbsoluteOffsetInCodeRegion {
        AbsoluteOffsetInCodeRegion(unsafe { base.offset(self.0 as isize) })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct AbsoluteOffsetInCodeRegion(pub *mut c_void);

pub struct InstructionSink {
    instructions: Vec<Instruction>,
    memory_offset_to_vm_exit: HashMap<MemoryOffset, VMExitType>,
    memory_offset_to_vm_return: HashMap<MemoryOffset, MemoryOffset>,
    current_memory_offset: usize,
}

#[must_use]
pub struct RegistrationGuard {
    before_offset: MemoryOffset,
}

pub struct VMExits {
    pub memory_offset_to_vm_exit: HashMap<AbsoluteOffsetInCodeRegion, VMExitType>,
    pub memory_offset_to_vm_return: HashMap<AbsoluteOffsetInCodeRegion, AbsoluteOffsetInCodeRegion>,
}

impl InstructionSink {
    pub fn new() -> Self {
        Self { instructions: vec![], memory_offset_to_vm_exit: HashMap::new(), memory_offset_to_vm_return: HashMap::new(), current_memory_offset: 0 }
    }

    pub fn add_instruction(&mut self, instruction: Instruction) {
        self.current_memory_offset += instruction.len();
        self.instructions.push(instruction);
    }

    pub fn register_exit_before(&mut self, vm_exit_type: VMExitType) -> RegistrationGuard {
        let before_offset = MemoryOffset(self.current_memory_offset);
        self.memory_offset_to_vm_exit.insert(before_offset, vm_exit_type);
        RegistrationGuard { before_offset }
    }

    pub fn register_exit_after(&mut self, registration_guard: RegistrationGuard) {
        self.memory_offset_to_vm_return.insert(registration_guard.before_offset, MemoryOffset(self.current_memory_offset));
    }

    pub fn as_slice(&self) -> &[Instruction] {
        self.instructions.as_slice()
    }

    pub fn get_vm_exits_given_installed_address(&self, installed_address: *mut c_void) -> VMExits {
        VMExits {
            memory_offset_to_vm_exit: self.memory_offset_to_vm_exit.iter().map(|(mem_offset, vm_exit)| (mem_offset.to_absolute(installed_address), vm_exit.clone())).collect(),
            memory_offset_to_vm_return: self.memory_offset_to_vm_return.iter().map(|(mem_offset, return_offset)| (mem_offset.to_absolute(installed_address), return_offset.to_absolute(installed_address))).collect(),
        }
    }
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
    pub fn to_x86(&self, instructions: &mut InstructionSink) {
        match self {
            IRInstruction::LoadAbsolute { address_from, output_offset, size } => {
                IRInstruction::load_absolute(instructions, address_from, output_offset, size)
            }
            IRInstruction::StoreAbsolute { address_to, input_offset, size } => {
                IRInstruction::store_absolute(instructions, address_to, input_offset, size);
            }
            IRInstruction::CopyRelative { input_offset, output_offset, signed, input_size, output_size } => {
                IRInstruction::copy_relative(instructions, input_offset, output_offset, signed, input_size, output_size);
            }
            IRInstruction::IntegerArithmetic { input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type } => {
                IRInstruction::integer_arithmetic(instructions, input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type);
            }
            IRInstruction::BranchUnConditional(_) => todo!(),
            IRInstruction::VMExit(exit_type) => {
                match exit_type {
                    VMExitType::CheckCast => todo!(),
                    VMExitType::InstanceOf => todo!(),
                    VMExitType::Throw => todo!(),
                    VMExitType::InvokeDynamic => todo!(),
                    VMExitType::InvokeStaticResolveTarget { .. } => todo!(),
                    VMExitType::InvokeVirtualResolveTarget { .. } => todo!(),
                    VMExitType::InvokeSpecialResolveTarget { .. } => todo!(),
                    VMExitType::InvokeInterfaceResolveTarget { .. } => todo!(),
                    VMExitType::MonitorEnter => todo!(),
                    VMExitType::MonitorExit => todo!(),
                    VMExitType::MultiNewArray => todo!(),
                    VMExitType::ArrayOutOfBounds => todo!(),
                    VMExitType::DebugTestExit => {
                        let restore_old_stack = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RSP, MemoryOperand::with_base_displ(Register::R15, (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,stack_pointer)) as i64));
                        instructions.add_instruction(restore_old_stack);
                        let restore_old_frame = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RBP, MemoryOperand::with_base_displ(Register::R15, (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,frame_pointer)) as i64));
                        instructions.add_instruction(restore_old_frame);
                        //todo should add 1 here to
                        let jmp_to_old = Instruction::with_mem(Code::Jmp_rm64, MemoryOperand::with_base_displ(Register::R15, (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,instruction_pointer)) as i64));
                        let registration_guard = instructions.register_exit_before(exit_type.clone());
                        instructions.add_instruction(jmp_to_old);
                        instructions.register_exit_after(registration_guard);
                    }
                    VMExitType::ExitDueToCompletion => todo!()
                }
            }
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
            }
            IRInstruction::Return { .. } => todo!(),
            IRInstruction::FloatArithmetic { input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type } => {
                IRInstruction::float_arithmetic(instructions, input_offset_a, input_offset_b, output_offset, size, arithmetic_type)
            }
            IRInstruction::BranchIf { .. } => todo!(),
            IRInstruction::BranchIfComparison { .. } => todo!(),
            IRInstruction::Label(_) => todo!(),
            IRInstruction::ReturnNone => {
                let move_sp_to_old_ip = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RSP, MemoryOperand::with_base(Register::RBP));
                instructions.add_instruction(move_sp_to_old_ip);
                let set_to_old_rbp = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RBP, MemoryOperand::with_base_displ(Register::RBP, 8));
                instructions.add_instruction(set_to_old_rbp);
                instructions.add_instruction(Instruction::with(Code::Retnq));
            }
            IRInstruction::Call { local_var_and_operand_stack_size, resolved_destination, return_location } => todo!()
        }
    }

    fn float_arithmetic(instructions: &mut InstructionSink, input_offset_a: &FramePointerOffset, input_offset_b: &FramePointerOffset, output_offset: &FramePointerOffset, size: &FloatSize, arithmetic_type: &FloatArithmeticType) {
        let input_load_memory_operand_a = MemoryOperand::with_base_displ(Register::RBP, input_offset_a.0 as i64);
        let load_value_a = match size {
            FloatSize::Float => Instruction::with_reg_mem(Code::Movss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_a),
            FloatSize::Double => Instruction::with_reg_mem(Code::Movsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_a)
        };
        instructions.add_instruction(load_value_a);
        let input_load_memory_operand_b = MemoryOperand::with_base_displ(Register::RBP, input_offset_b.0 as i64);
        let load_value_b = match size {
            FloatSize::Float => Instruction::with_reg_mem(Code::Movss_xmm_xmmm32, Register::XMM1, input_load_memory_operand_b.clone()),
            FloatSize::Double => Instruction::with_reg_mem(Code::Movsd_xmm_xmmm64, Register::XMM1, input_load_memory_operand_b.clone())
        };
        instructions.add_instruction(load_value_b);
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
        instructions.add_instruction(operation);
        let output_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
        let write_res = match size {
            FloatSize::Float => Instruction::with_mem_reg(Code::Movss_xmmm32_xmm, output_memory_operand, Register::XMM0),
            FloatSize::Double => Instruction::with_mem_reg(Code::Movsd_xmmm64_xmm, output_memory_operand, Register::XMM0)
        };
        instructions.add_instruction(write_res)
    }

    fn integer_arithmetic(instructions: &mut InstructionSink, input_offset_a: &FramePointerOffset, input_offset_b: &FramePointerOffset, output_offset: &FramePointerOffset, size: &Size, signed: &bool, arithmetic_type: &ArithmeticType) {
        let input_load_memory_operand_a = MemoryOperand::with_base_displ(Register::RBP, input_offset_a.0 as i64);
        let load_value_a = match size {
            Size::Byte => Instruction::with_reg_mem(Code::Mov_rm8_r8, Register::AL, input_load_memory_operand_a),
            Size::Short => Instruction::with_reg_mem(Code::Mov_rm16_r16, Register::AX, input_load_memory_operand_a),
            Size::Int => Instruction::with_reg_mem(Code::Mov_rm32_r32, Register::EAX, input_load_memory_operand_a),
            Size::Long => Instruction::with_reg_mem(Code::Mov_rm64_r64, Register::RAX, input_load_memory_operand_a)
        };
        instructions.add_instruction(load_value_a);
        let input_load_memory_operand_b = MemoryOperand::with_base_displ(Register::RBP, input_offset_b.0 as i64);
        let load_value_b = match size {
            Size::Byte => Instruction::with_reg_mem(Code::Mov_rm8_r8, Register::BL, input_load_memory_operand_b.clone()),
            Size::Short => Instruction::with_reg_mem(Code::Mov_rm16_r16, Register::BX, input_load_memory_operand_b.clone()),
            Size::Int => Instruction::with_reg_mem(Code::Mov_rm32_r32, Register::EBX, input_load_memory_operand_b.clone()),
            Size::Long => Instruction::with_reg_mem(Code::Mov_rm64_r64, Register::RBX, input_load_memory_operand_b.clone())
        };
        instructions.add_instruction(load_value_b);
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
        instructions.add_instruction(write_result);
    }

    fn copy_relative(instructions: &mut InstructionSink, input_offset: &FramePointerOffset, output_offset: &FramePointerOffset, signed: &bool, input_size: &Size, output_size: &Size) {
        let input_load_memory_operand = MemoryOperand::with_base_displ(Register::RBP, input_offset.0 as i64);

        let load_to_register = match output_size {
            Size::Byte => Register::BL,
            Size::Short => Register::BX,
            Size::Int => Register::EBX,
            Size::Long => if !*signed && input_size == &Size::Long { Register::EBX } else { Register::RBX }
        };
        let opcode = if *signed {
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
        instructions.add_instruction(load_value);
        instructions.add_instruction(write_value);
    }

    fn store_absolute(instructions: &mut InstructionSink, address_to: &FramePointerOffset, input_offset: &FramePointerOffset, size: &Size) {
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
        instructions.add_instruction(write_address_load);
        instructions.add_instruction(load_value);
        instructions.add_instruction(write_value);
    }

    fn load_absolute(instructions: &mut InstructionSink, address_from: &FramePointerOffset, output_offset: &FramePointerOffset, size: &Size) -> () {
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
        instructions.add_instruction(load_address);
        instructions.add_instruction(load_value);
        instructions.add_instruction(write_value)
    }
}

#[cfg(test)]
pub mod test {
    use iced_x86::{Formatter, Instruction, IntelFormatter};

    use crate::{FramePointerOffset, IRInstruction, Size};

    #[test]
    pub fn test() {}
}