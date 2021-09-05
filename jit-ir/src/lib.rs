#[macro_use]
extern crate memoffset;

use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::NonNull;

use bimap::BiMap;
use iced_x86::{BlockEncoder, BlockEncoderOptions, BlockEncoderResult, Code, Instruction, InstructionBlock, MemoryOperand, Register};

use gc_memory_layout_common::FramePointerOffset;
use jit_common::{JitCodeContext, VMExitData};
use jit_common::SavedRegisters;

pub struct RelativeAddress(isize);

#[derive(Clone, Copy)]
#[derive(Eq, PartialEq)]
#[derive(Debug)]
pub enum Size {
    Byte,
    Short,
    Int,
    Long,
}


pub struct VariableSize(pub usize);

#[derive(Debug)]
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


#[derive(Debug)]
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
#[derive(Debug)]
pub struct IRLabel {
    id: usize,
}

#[derive(Debug)]
pub enum BranchType0 {
    Equal0,
    Less0,
    More0,
    LessEqual0,
    MoreEqual0,
}

#[derive(Debug)]
pub enum BranchType {
    Equal,
    Less,
    More,
    LessEqual,
    MoreEqual,
}

#[derive(Debug)]
pub enum BranchTypeFloat {
    EqualFloat,
    LessFloat,
    MoreFloat,
    LessEqualFloat,
    MoreEqualFloat,
}

#[derive(Debug)]
pub enum FloatSize {
    Float,
    Double,
}

#[derive(Debug)]
pub enum FloatArithmeticType {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
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
        resolved_destination_rel: NonNull<c_void>,
        local_var_and_operand_stack_size: FramePointerOffset,
        return_location: Option<FramePointerOffset>,
    },
    VMExit(VMExitData),
    Label(IRLabel),
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct InstructionCount(usize);

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct AbsolutePositionInCodeRegion(pub *mut c_void);

#[derive(Debug)]
pub struct InstructionSink {
    instructions: Vec<Instruction>,
    ir_indexes: Vec<usize>,
    memory_offset_to_vm_exit: HashMap<InstructionCount, VMExitData>,
    memory_offset_to_vm_return: HashMap<InstructionCount, InstructionCount>,
    current_instruction_count: usize,
}

#[must_use]
pub struct RegistrationGuard {
    before_offset: InstructionCount,
}

pub struct VMExits {
    pub memory_offset_to_vm_exit: HashMap<AbsolutePositionInCodeRegion, VMExitData>,
    pub memory_offset_to_vm_return: HashMap<AbsolutePositionInCodeRegion, Option<AbsolutePositionInCodeRegion>>,
}

impl InstructionSink {
    pub fn new() -> Self {
        Self { instructions: vec![], ir_indexes: vec![], memory_offset_to_vm_exit: HashMap::new(), memory_offset_to_vm_return: HashMap::new(), current_instruction_count: 0 }
    }

    pub fn add_instruction(&mut self, ir_index: usize, instruction: Instruction) {
        self.current_instruction_count += 1;
        self.instructions.push(instruction);
        self.ir_indexes.push(ir_index);
    }

    pub fn register_exit_before(&mut self, vm_exit_type: VMExitData) -> RegistrationGuard {
        let before_offset = InstructionCount(self.current_instruction_count);
        self.memory_offset_to_vm_exit.insert(before_offset, vm_exit_type);
        RegistrationGuard { before_offset }
    }

    pub fn register_exit_after(&mut self, registration_guard: RegistrationGuard) {
        self.memory_offset_to_vm_return.insert(registration_guard.before_offset, InstructionCount(self.current_instruction_count));
    }

    pub fn fully_compiled(self, install_to: *mut c_void, max_len: usize) -> (VMExits, IRIndexToNative, usize) {
        let InstructionSink {
            instructions,
            current_instruction_count,
            ir_indexes,
            memory_offset_to_vm_return,
            memory_offset_to_vm_exit
        } = self;
        let block = InstructionBlock::new(instructions.as_slice(), install_to as u64);
        dbg!(&instructions);
        let BlockEncoderResult {
            rip,
            code_buffer,
            reloc_infos,
            new_instruction_offsets,
            constant_offsets
        } = BlockEncoder::encode(64, block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
        dbg!(code_buffer.len());
        dbg!(&new_instruction_offsets);
        dbg!(reloc_infos.len());
        if code_buffer.len() > max_len {
            todo!()
        }
        unsafe { libc::memcpy(install_to, code_buffer.as_ptr() as *const c_void, code_buffer.len()); }
        const SIZE_OF_CALL_INSTRUCTION: isize = 4isize;
        let vmexits = VMExits {
            memory_offset_to_vm_exit: memory_offset_to_vm_exit.into_iter()
                .map(|(ir_offset, exit_type)| {
                    (unsafe { AbsolutePositionInCodeRegion(install_to.offset(new_instruction_offsets[ir_offset.0] as isize + SIZE_OF_CALL_INSTRUCTION)) }, exit_type)
                })
                .collect(),
            memory_offset_to_vm_return: memory_offset_to_vm_return.into_iter()
                .map(|(ir_offset_exit, ir_offset_return)| unsafe {
                    (AbsolutePositionInCodeRegion(install_to.offset(new_instruction_offsets[ir_offset_exit.0] as isize + SIZE_OF_CALL_INSTRUCTION)),
                     Some(AbsolutePositionInCodeRegion(install_to.offset(new_instruction_offsets[ir_offset_return.0] as isize - SIZE_OF_CALL_INSTRUCTION))))
                })
                .collect(),
        };
        dbg!(&ir_indexes);
        dbg!(&new_instruction_offsets);
        let inner = ir_indexes.into_iter().zip(new_instruction_offsets.into_iter())
            .map(|(ir_index, new_offset)| (ir_index, unsafe { install_to.offset(new_offset as isize) }))
            .collect::<BiMap<_, _>>();
        dbg!(&inner);
        (vmexits, IRIndexToNative { inner }, code_buffer.len())
    }
}


pub struct IRIndexToNative {
    pub inner: BiMap<usize, *mut c_void>,
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
    pub fn to_x86(&self, ir_instruction_index: usize, instructions: &mut InstructionSink) {
        match self {
            IRInstruction::LoadAbsolute { address_from, output_offset, size } => {
                IRInstruction::load_absolute(ir_instruction_index, instructions, address_from, output_offset, size)
            }
            IRInstruction::StoreAbsolute { address_to, input_offset, size } => {
                IRInstruction::store_absolute(ir_instruction_index, instructions, address_to, input_offset, size);
            }
            IRInstruction::CopyRelative { input_offset, output_offset, signed, input_size, output_size } => {
                IRInstruction::copy_relative(ir_instruction_index, instructions, input_offset, output_offset, signed, input_size, output_size);
            }
            IRInstruction::IntegerArithmetic { input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type } => {
                IRInstruction::integer_arithmetic(ir_instruction_index, instructions, input_offset_a, input_offset_b, output_offset, size, signed, arithmetic_type);
            }
            IRInstruction::BranchUnConditional(_) => todo!(),
            IRInstruction::VMExit(exit_type) => {
                let native_stack_pointer = (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,stack_pointer)) as i64;
                let native_frame_pointer = (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,frame_pointer)) as i64;
                let native_instruction_pointer = (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,instruction_pointer)) as i64;
                let java_stack_pointer = (offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,stack_pointer)) as i64;
                let java_frame_pointer = (offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,frame_pointer)) as i64;
                // let java_instruction_pointer = (offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,instruction_pointer)) as i64;
                let save_java_stack = Instruction::with_mem_reg(Code::Mov_rm64_r64, MemoryOperand::with_base_displ(Register::R15, java_stack_pointer), Register::RSP);
                instructions.add_instruction(ir_instruction_index, save_java_stack);
                let save_java_frame = Instruction::with_mem_reg(Code::Mov_rm64_r64, MemoryOperand::with_base_displ(Register::R15, java_frame_pointer), Register::RBP);
                instructions.add_instruction(ir_instruction_index, save_java_frame);

                let restore_old_stack = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RSP, MemoryOperand::with_base_displ(Register::R15, native_stack_pointer));
                instructions.add_instruction(ir_instruction_index, restore_old_stack);
                let restore_old_frame = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RBP, MemoryOperand::with_base_displ(Register::R15, native_frame_pointer));
                instructions.add_instruction(ir_instruction_index, restore_old_frame);
                let call_to_old = Instruction::with_mem(Code::Call_rm64, MemoryOperand::with_base_displ(Register::R15, native_instruction_pointer));
                let registration_guard = instructions.register_exit_before(exit_type.clone());
                instructions.add_instruction(ir_instruction_index, call_to_old);
                instructions.register_exit_after(registration_guard);
                match exit_type {
                    VMExitData::CheckCast => todo!(),
                    VMExitData::InstanceOf => todo!(),
                    VMExitData::Throw => todo!(),
                    VMExitData::InvokeDynamic => todo!(),
                    VMExitData::InvokeStaticResolveTarget { .. } => {}
                    VMExitData::InvokeVirtualResolveTarget { .. } => todo!(),
                    VMExitData::InvokeSpecialResolveTarget { .. } => todo!(),
                    VMExitData::InvokeInterfaceResolveTarget { .. } => todo!(),
                    VMExitData::MonitorEnter => todo!(),
                    VMExitData::MonitorExit => todo!(),
                    VMExitData::MultiNewArray => todo!(),
                    VMExitData::ArrayOutOfBounds => todo!(),
                    VMExitData::DebugTestExit => {}
                    VMExitData::ExitDueToCompletion => {}
                    VMExitData::DebugTestExitValue { .. } => todo!()
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
                    Constant::Int(constant) => {
                        instructions.add_instruction(ir_instruction_index, Instruction::try_with_mem_i32(Code::Mov_rm32_imm32, output_memory_operand, *constant).expect("wat"));
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
                IRInstruction::float_arithmetic(ir_instruction_index, instructions, input_offset_a, input_offset_b, output_offset, size, arithmetic_type)
            }
            IRInstruction::BranchIf { .. } => todo!(),
            IRInstruction::BranchIfComparison { .. } => todo!(),
            IRInstruction::Label(_) => todo!(),
            IRInstruction::ReturnNone => {
                let move_sp_to_old_ip = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RSP, MemoryOperand::with_base(Register::RBP));
                instructions.add_instruction(ir_instruction_index, move_sp_to_old_ip);
                let set_to_old_rbp = Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RBP, MemoryOperand::with_base_displ(Register::RBP, 8));
                instructions.add_instruction(ir_instruction_index, set_to_old_rbp);
                instructions.add_instruction(ir_instruction_index, Instruction::with(Code::Retnq));
            }
            IRInstruction::Call { resolved_destination_rel, local_var_and_operand_stack_size, return_location } => todo!()
        }
    }

    fn float_arithmetic(ir_instruction_index: usize, instructions: &mut InstructionSink, input_offset_a: &FramePointerOffset, input_offset_b: &FramePointerOffset, output_offset: &FramePointerOffset, size: &FloatSize, arithmetic_type: &FloatArithmeticType) {
        let input_load_memory_operand_a = MemoryOperand::with_base_displ(Register::RBP, input_offset_a.0 as i64);
        let load_value_a = match size {
            FloatSize::Float => Instruction::with_reg_mem(Code::Movss_xmm_xmmm32, Register::XMM0, input_load_memory_operand_a),
            FloatSize::Double => Instruction::with_reg_mem(Code::Movsd_xmm_xmmm64, Register::XMM0, input_load_memory_operand_a)
        };
        instructions.add_instruction(ir_instruction_index, load_value_a);
        let input_load_memory_operand_b = MemoryOperand::with_base_displ(Register::RBP, input_offset_b.0 as i64);
        let load_value_b = match size {
            FloatSize::Float => Instruction::with_reg_mem(Code::Movss_xmm_xmmm32, Register::XMM1, input_load_memory_operand_b.clone()),
            FloatSize::Double => Instruction::with_reg_mem(Code::Movsd_xmm_xmmm64, Register::XMM1, input_load_memory_operand_b.clone())
        };
        instructions.add_instruction(ir_instruction_index, load_value_b);
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
        instructions.add_instruction(ir_instruction_index, operation);
        let output_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
        let write_res = match size {
            FloatSize::Float => Instruction::with_mem_reg(Code::Movss_xmmm32_xmm, output_memory_operand, Register::XMM0),
            FloatSize::Double => Instruction::with_mem_reg(Code::Movsd_xmmm64_xmm, output_memory_operand, Register::XMM0)
        };
        instructions.add_instruction(ir_instruction_index, write_res)
    }

    fn integer_arithmetic(ir_instruction_index: usize, instructions: &mut InstructionSink, input_offset_a: &FramePointerOffset, input_offset_b: &FramePointerOffset, output_offset: &FramePointerOffset, size: &Size, signed: &bool, arithmetic_type: &ArithmeticType) {
        let input_load_memory_operand_a = MemoryOperand::with_base_displ(Register::RBP, input_offset_a.0 as i64);
        let load_value_a = match size {
            Size::Byte => Instruction::with_reg_mem(Code::Mov_r8_rm8, Register::AL, input_load_memory_operand_a),
            Size::Short => Instruction::with_reg_mem(Code::Mov_r16_rm16, Register::AX, input_load_memory_operand_a),
            Size::Int => Instruction::with_reg_mem(Code::Mov_r32_rm32, Register::EAX, input_load_memory_operand_a),
            Size::Long => Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RAX, input_load_memory_operand_a)
        };
        instructions.add_instruction(ir_instruction_index, load_value_a);
        let input_load_memory_operand_b = MemoryOperand::with_base_displ(Register::RBP, input_offset_b.0 as i64);
        let load_value_b = match size {
            Size::Byte => Instruction::with_reg_mem(Code::Mov_r8_rm8, Register::BL, input_load_memory_operand_b.clone()),
            Size::Short => Instruction::with_reg_mem(Code::Mov_r16_rm16, Register::BX, input_load_memory_operand_b.clone()),
            Size::Int => Instruction::with_reg_mem(Code::Mov_r32_rm32, Register::EBX, input_load_memory_operand_b.clone()),
            Size::Long => Instruction::with_reg_mem(Code::Mov_r64_rm64, Register::RBX, input_load_memory_operand_b.clone())
        };
        instructions.add_instruction(ir_instruction_index, load_value_b);
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
        instructions.add_instruction(ir_instruction_index, arithmetic);
        let output_memory_operand = MemoryOperand::with_base_displ(Register::RBP, output_offset.0 as i64);
        let write_result = match size {
            Size::Byte => Instruction::with_mem_reg(Code::Mov_rm8_r8, output_memory_operand, Register::AL),
            Size::Short => Instruction::with_mem_reg(Code::Mov_rm16_r16, output_memory_operand, Register::AX),
            Size::Int => Instruction::with_mem_reg(Code::Mov_rm32_r32, output_memory_operand, Register::EAX),
            Size::Long => Instruction::with_mem_reg(Code::Mov_rm64_r64, output_memory_operand, Register::RAX),
        };
        instructions.add_instruction(ir_instruction_index, write_result);
    }

    fn copy_relative(ir_instruction_index: usize, instructions: &mut InstructionSink, input_offset: &FramePointerOffset, output_offset: &FramePointerOffset, signed: &bool, input_size: &Size, output_size: &Size) {
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
        instructions.add_instruction(ir_instruction_index, load_value);
        instructions.add_instruction(ir_instruction_index, write_value);
    }

    fn store_absolute(ir_instruction_index: usize, instructions: &mut InstructionSink, address_to: &FramePointerOffset, input_offset: &FramePointerOffset, size: &Size) {
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
        instructions.add_instruction(ir_instruction_index, write_address_load);
        instructions.add_instruction(ir_instruction_index, load_value);
        instructions.add_instruction(ir_instruction_index, write_value);
    }

    fn load_absolute(ir_instruction_index: usize, instructions: &mut InstructionSink, address_from: &FramePointerOffset, output_offset: &FramePointerOffset, size: &Size) -> () {
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
        instructions.add_instruction(ir_instruction_index, load_address);
        instructions.add_instruction(ir_instruction_index, load_value);
        instructions.add_instruction(ir_instruction_index, write_value)
    }
}

#[cfg(test)]
pub mod test {
    use iced_x86::{Formatter, Instruction, IntelFormatter};

    use crate::{FramePointerOffset, IRInstruction, Size};

    #[test]
    pub fn test() {}
}