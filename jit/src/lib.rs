#![feature(asm)]

extern crate compiler_builtins;

use std::collections::HashMap;
use std::env::current_exe;
use std::ffi::c_void;
use std::mem::{size_of, transmute};
use std::num::NonZeroUsize;
use std::panic::panic_any;
use std::ptr::null_mut;
use std::sync::atomic::{fence, Ordering};

use iced_x86::{BlockEncoder, BlockEncoderOptions, BlockEncoderResult, Encoder, InstructionBlock};
use iced_x86::CC_b::c;
use iced_x86::CC_l::l;
use iced_x86::Code::Adc_r16_rm16;
use iced_x86::ImmSize::Size1;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, StackframeMemoryLayout};
use jit_ir::{ArithmeticType, BranchType, Constant, IRInstruction, IRLabel, Size, VariableSize, VMExitType};
use rust_jvm_common::classfile::{Code, Instruction, InstructionInfo};
use verification::verifier::Frame;

use crate::arrays::{array_load, array_store};
use crate::integer_arithmetic::{binary_and, binary_or, binary_xor, integer_add, integer_div, integer_mul, integer_sub, shift, ShiftDirection};

pub enum JITError {
    NotSupported
}


pub struct JitBlock {
    java_pc_to_ir: HashMap<u16, usize>,
    instructions: Vec<IRInstruction>,
}

impl JitBlock {
    pub fn add_instruction(&mut self, instruction: IRInstruction) {
        todo!()
    }
}

pub struct JitIROutput {
    blocks: Vec<JitBlock>,
}

impl JitIROutput {
    pub fn add_block(&mut self, block: JitBlock) {
        todo!()
    }
}

pub struct JitState {
    memory_layout: StackframeMemoryLayout,
    java_pc: usize,
    next_pc: Option<NonZeroUsize>,
    output: JitIROutput,
}

impl JitState {
    pub fn new_ir_label(&self) -> IRLabel {
        todo!()
    }

    pub fn next_pc(&self) -> usize {
        todo!()
    }
}

const MAX_INTERMEDIATE_VALUE_PADDING: usize = 3;

pub fn code_to_ir(code: &Code, frame_vtypes: HashMap<usize, Frame>) -> Result<JitIROutput, JITError> {
    let mut jit_state = JitState {
        memory_layout: StackframeMemoryLayout::new((code.max_stack + MAX_INTERMEDIATE_VALUE_PADDING) as usize, code.max_locals as usize, frame_vtypes),
        java_pc: 0,
        next_pc: None,
        output: JitIROutput { blocks: vec![] },
    };
    let mut current_instr = None;
    let mut main_block = JitBlock {
        java_pc_to_ir: Default::default(),
        instructions: vec![],
    };
    for future_instr in &code.code {
        if let Some(current_instr) = current_instr.take() {
            jit_state.next_pc = Some(NonZeroUsize::new(future_instr.offset).unwrap());
            jit_state.java_pc = current_instr.offset;
            byte_code_to_ir(current_instr, &mut jit_state, &mut main_block)?;
        }
        jit_state.next_pc = None;
        current_instr = Some(future_instr.clone());
    }
    byte_code_to_ir(current_instr.unwrap(), &mut jit_state, &mut main_block)?;
    jit_state.output.add_block(main_block);
    Ok(jit_state.output)
}


pub fn byte_code_to_ir(bytecode: &Instruction, current_jit_state: &mut JitState, main_block: &mut JitBlock) {
    let Instruction { offset, instruction: instruction_info } = bytecode;
    current_jit_state.java_pc = *offset;
    let java_pc = current_jit_state.java_pc;
    match instruction_info {
        InstructionInfo::aaload => {
            array_load(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::aastore => {
            array_store(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::aconst_null => {
            constant(current_jit_state, Constant::Pointer(0))
        }
        InstructionInfo::aload(variable_index) => {
            aload_n(current_jit_state, *variable_index as usize)
        }
        InstructionInfo::aload_0 => {
            aload_n(current_jit_state, 0)
        }
        InstructionInfo::aload_1 => {
            aload_n(current_jit_state, 1)
        }
        InstructionInfo::aload_2 => {
            aload_n(current_jit_state, 2)
        }
        InstructionInfo::aload_3 => {
            aload_n(current_jit_state, 3)
        }
        InstructionInfo::anewarray(_) => Err(JITError::NotSupported),
        InstructionInfo::areturn => {
            main_block.add_instruction(IRInstruction::Return {
                return_value: Some(current_jit_state.memory_layout.operand_stack_entry(java_pc, 0)),
                to_pop: VariableSize(current_jit_state.memory_layout.full_frame_size()),
            });
            Ok(())
        }
        InstructionInfo::arraylength => {
            let layout: ArrayMemoryLayout = todo!();
            layout.len_entry();
            todo!();
            Ok(())
        }
        InstructionInfo::astore(variable_index) => {
            astore_n(current_jit_state, *variable_index as usize)
        }
        InstructionInfo::astore_0 => {
            astore_n(current_jit_state, 0)
        }
        InstructionInfo::astore_1 => {
            astore_n(current_jit_state, 1)
        }
        InstructionInfo::astore_2 => {
            astore_n(current_jit_state, 2)
        }
        InstructionInfo::astore_3 => {
            astore_n(current_jit_state, 3)
        }
        InstructionInfo::athrow => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::Throw));
            Ok(())
        }
        InstructionInfo::baload => {
            array_load(current_jit_state, main_block, Size::Byte)
        }
        InstructionInfo::bastore => {
            array_store(current_jit_state, main_block, Size::Byte)
        }
        InstructionInfo::bipush(_) => Err(JITError::NotSupported),
        InstructionInfo::caload => {
            array_load(current_jit_state, main_block, Size::Short)
        }
        InstructionInfo::castore => { Err(JITError::NotSupported) }
        InstructionInfo::checkcast(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::CheckCast));
            Ok(())
        }
        InstructionInfo::d2f => Err(JITError::NotSupported),
        InstructionInfo::d2i => Err(JITError::NotSupported),
        InstructionInfo::d2l => Err(JITError::NotSupported),
        InstructionInfo::dadd => Err(JITError::NotSupported),
        InstructionInfo::daload => {
            array_load(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::dastore => {
            array_store(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::dcmpg => Err(JITError::NotSupported),
        InstructionInfo::dcmpl => Err(JITError::NotSupported),
        InstructionInfo::dconst_0 => {
            constant(current_jit_state, Constant::Double(0f64))
        }
        InstructionInfo::dconst_1 => {
            constant(current_jit_state, Constant::Double(1f64))
        }
        InstructionInfo::ddiv => Err(JITError::NotSupported),
        InstructionInfo::dload(n) => {
            store_n(current_jit_state, *n as usize, Size::Long)
        }
        InstructionInfo::dload_0 => {
            store_n(current_jit_state, 0, Size::Long)
        }
        InstructionInfo::dload_1 => {
            store_n(current_jit_state, 1, Size::Long)
        }
        InstructionInfo::dload_2 => {
            store_n(current_jit_state, 2, Size::Long)
        }
        InstructionInfo::dload_3 => {
            store_n(current_jit_state, 3, Size::Long)
        }
        InstructionInfo::dmul => Err(JITError::NotSupported),
        InstructionInfo::dneg => Err(JITError::NotSupported),
        InstructionInfo::drem => Err(JITError::NotSupported),
        InstructionInfo::dreturn => Err(JITError::NotSupported),
        InstructionInfo::dstore(n) => {
            store_n(current_jit_state, *n as usize, Size::Long)
        }
        InstructionInfo::dstore_0 => {
            store_n(current_jit_state, 0, Size::Long)
        }
        InstructionInfo::dstore_1 => {
            store_n(current_jit_state, 1, Size::Long)
        }
        InstructionInfo::dstore_2 => {
            store_n(current_jit_state, 2, Size::Long)
        }
        InstructionInfo::dstore_3 => {
            store_n(current_jit_state, 3, Size::Long)
        }
        InstructionInfo::dsub => Err(JITError::NotSupported),
        InstructionInfo::dup => Err(JITError::NotSupported),
        InstructionInfo::dup_x1 => Err(JITError::NotSupported),
        InstructionInfo::dup_x2 => Err(JITError::NotSupported),
        InstructionInfo::dup2 => Err(JITError::NotSupported),
        InstructionInfo::dup2_x1 => Err(JITError::NotSupported),
        InstructionInfo::dup2_x2 => Err(JITError::NotSupported),
        InstructionInfo::f2d => Err(JITError::NotSupported),
        InstructionInfo::f2i => Err(JITError::NotSupported),
        InstructionInfo::f2l => Err(JITError::NotSupported),
        InstructionInfo::fadd => Err(JITError::NotSupported),
        InstructionInfo::faload => {
            array_load(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::fastore => {
            array_store(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::fcmpg => Err(JITError::NotSupported),
        InstructionInfo::fcmpl => Err(JITError::NotSupported),
        InstructionInfo::fconst_0 => {
            constant(current_jit_state, Constant::Float(0.0f32))
        }
        InstructionInfo::fconst_1 => {
            constant(current_jit_state, Constant::Float(1.0f32))
        }
        InstructionInfo::fconst_2 => {
            constant(current_jit_state, Constant::Float(2.0f32))
        }
        InstructionInfo::fdiv => Err(JITError::NotSupported),
        InstructionInfo::fload(n) => {
            load_n(current_jit_state, *n as usize, Size::Int)
        }
        InstructionInfo::fload_0 => {
            load_n(current_jit_state, 0, Size::Int)
        }
        InstructionInfo::fload_1 => {
            load_n(current_jit_state, 1, Size::Int)
        }
        InstructionInfo::fload_2 => {
            load_n(current_jit_state, 2, Size::Int)
        }
        InstructionInfo::fload_3 => {
            load_n(current_jit_state, 3, Size::Int)
        }
        InstructionInfo::fmul => Err(JITError::NotSupported),
        InstructionInfo::fneg => Err(JITError::NotSupported),
        InstructionInfo::frem => Err(JITError::NotSupported),
        InstructionInfo::freturn => Err(JITError::NotSupported),
        InstructionInfo::fstore(n) => {
            store_n(current_jit_state, *n as usize, Size::Int)
        }
        InstructionInfo::fstore_0 => {
            store_n(current_jit_state, 0, Size::Int)
        }
        InstructionInfo::fstore_1 => {
            store_n(current_jit_state, 1, Size::Int)
        }
        InstructionInfo::fstore_2 => {
            store_n(current_jit_state, 2, Size::Int)
        }
        InstructionInfo::fstore_3 => {
            store_n(current_jit_state, 3, Size::Int)
        }
        InstructionInfo::fsub => Err(JITError::NotSupported),
        InstructionInfo::getfield(_) => Err(JITError::NotSupported),
        InstructionInfo::getstatic(_) => Err(JITError::NotSupported),
        InstructionInfo::goto_(_) => Err(JITError::NotSupported),
        InstructionInfo::goto_w(_) => Err(JITError::NotSupported),
        InstructionInfo::i2b => Err(JITError::NotSupported),
        InstructionInfo::i2c => Err(JITError::NotSupported),
        InstructionInfo::i2d => Err(JITError::NotSupported),
        InstructionInfo::i2f => Err(JITError::NotSupported),
        InstructionInfo::i2l => Err(JITError::NotSupported),
        InstructionInfo::i2s => Err(JITError::NotSupported),
        InstructionInfo::iadd => {
            integer_add(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::iaload => {
            array_load(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::iand => {
            binary_and(current_jit_state, Size::Int)
        }
        InstructionInfo::iastore => {
            array_store(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::iconst_m1 => {
            constant(current_jit_state, Constant::Int(-1))
        }
        InstructionInfo::iconst_0 => {
            constant(current_jit_state, Constant::Int(0))
        }
        InstructionInfo::iconst_1 => {
            constant(current_jit_state, Constant::Int(1))
        }
        InstructionInfo::iconst_2 => {
            constant(current_jit_state, Constant::Int(2))
        }
        InstructionInfo::iconst_3 => {
            constant(current_jit_state, Constant::Int(3))
        }
        InstructionInfo::iconst_4 => {
            constant(current_jit_state, Constant::Int(4))
        }
        InstructionInfo::iconst_5 => {
            constant(current_jit_state, Constant::Int(5))
        }
        InstructionInfo::idiv => {
            integer_div(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::if_acmpeq(_) => Err(JITError::NotSupported),
        InstructionInfo::if_acmpne(_) => Err(JITError::NotSupported),
        InstructionInfo::if_icmpeq(_) => Err(JITError::NotSupported),
        InstructionInfo::if_icmpne(_) => Err(JITError::NotSupported),
        InstructionInfo::if_icmplt(_) => Err(JITError::NotSupported),
        InstructionInfo::if_icmpge(_) => Err(JITError::NotSupported),
        InstructionInfo::if_icmpgt(_) => Err(JITError::NotSupported),
        InstructionInfo::if_icmple(_) => Err(JITError::NotSupported),
        InstructionInfo::ifeq(_) => Err(JITError::NotSupported),
        InstructionInfo::ifne(_) => Err(JITError::NotSupported),
        InstructionInfo::iflt(_) => Err(JITError::NotSupported),
        InstructionInfo::ifge(_) => Err(JITError::NotSupported),
        InstructionInfo::ifgt(_) => Err(JITError::NotSupported),
        InstructionInfo::ifle(_) => Err(JITError::NotSupported),
        InstructionInfo::ifnonnull(_) => Err(JITError::NotSupported),
        InstructionInfo::ifnull(_) => Err(JITError::NotSupported),
        InstructionInfo::iinc(_) => Err(JITError::NotSupported),
        InstructionInfo::iload(n) => {
            load_n(current_jit_state, *n as usize, Size::Int)
        }
        InstructionInfo::iload_0 => {
            load_n(current_jit_state, 0, Size::Int)
        }
        InstructionInfo::iload_1 => {
            load_n(current_jit_state, 1, Size::Int)
        }
        InstructionInfo::iload_2 => {
            load_n(current_jit_state, 2, Size::Int)
        }
        InstructionInfo::iload_3 => {
            load_n(current_jit_state, 3, Size::Int)
        }
        InstructionInfo::imul => {
            let instruct = IRInstruction::IntegerArithmetic {
                input_offset_a: current_jit_state.memory_layout.operand_stack_entry(java_pc, 1),
                input_offset_b: current_jit_state.memory_layout.operand_stack_entry(java_pc, 0),
                output_offset: current_jit_state.memory_layout.operand_stack_entry(java_pc, 1),
                size: Size::Int,
                signed: true,
                arithmetic_type: ArithmeticType::Mul,
            };
            main_block.add_instruction(instruct);
            Ok(())
        }
        InstructionInfo::ineg => {
            todo!()
        }
        InstructionInfo::instanceof(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::InstanceOf));
            Ok(())
        }
        InstructionInfo::invokedynamic(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeDynamic));
            Ok(())
        }
        InstructionInfo::invokeinterface(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeInterface));
            Ok(())
        }
        InstructionInfo::invokespecial(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeSpecial));
            Ok(())
        }
        InstructionInfo::invokestatic(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeStatic));
            Ok(())
        }
        InstructionInfo::invokevirtual(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeVirtual));
            Ok(())
        }
        InstructionInfo::ior => {
            binary_or(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::irem => Err(JITError::NotSupported),
        InstructionInfo::ireturn => Err(JITError::NotSupported),
        InstructionInfo::ishl => {
            shift(current_jit_state, main_block, java_pc, Size::Int, ShiftDirection::ArithmeticLeft)
        }
        InstructionInfo::ishr => {
            shift(current_jit_state, main_block, java_pc, Size::Int, ShiftDirection::ArithmeticRight)
        }
        InstructionInfo::istore(n) => {
            store_n(current_jit_state, *n as usize, Size::Int)
        }
        InstructionInfo::istore_0 => {
            store_n(current_jit_state, 0, Size::Int)
        }
        InstructionInfo::istore_1 => {
            store_n(current_jit_state, 1, Size::Int)
        }
        InstructionInfo::istore_2 => {
            store_n(current_jit_state, 2, Size::Int)
        }
        InstructionInfo::istore_3 => {
            store_n(current_jit_state, 3, Size::Int)
        }
        InstructionInfo::isub => {
            integer_sub(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::iushr => {
            shift(current_jit_state, main_block, java_pc, Size::Int, ShiftDirection::LogicalRight)
        }
        InstructionInfo::ixor => {
            binary_xor(current_jit_state, main_block, Size::Int)
        }
        InstructionInfo::jsr(_) => Err(JITError::NotSupported),
        InstructionInfo::jsr_w(_) => Err(JITError::NotSupported),
        InstructionInfo::l2d => Err(JITError::NotSupported),
        InstructionInfo::l2f => Err(JITError::NotSupported),
        InstructionInfo::l2i => Err(JITError::NotSupported),
        InstructionInfo::ladd => {
            integer_add(current_jit_state, main_block, Size::Long)
        },
        InstructionInfo::laload => {
            array_load(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::land => {
            binary_and(current_jit_state, Size::Long)
        }
        InstructionInfo::lastore => {
            array_store(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::lcmp => Err(JITError::NotSupported),
        InstructionInfo::lconst_0 => {
            constant(current_jit_state, Constant::Long(0))
        }
        InstructionInfo::lconst_1 => {
            constant(current_jit_state, Constant::Long(1))
        }
        InstructionInfo::ldc(_) => Err(JITError::NotSupported),
        InstructionInfo::ldc_w(_) => Err(JITError::NotSupported),
        InstructionInfo::ldc2_w(_) => Err(JITError::NotSupported),
        InstructionInfo::ldiv => {
            integer_div(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::lload(n) => {
            load_n(current_jit_state, *n as usize, Size::Long)
        }
        InstructionInfo::lload_0 => {
            load_n(current_jit_state, 0, Size::Long)
        }
        InstructionInfo::lload_1 => {
            load_n(current_jit_state, 1, Size::Long)
        }
        InstructionInfo::lload_2 => {
            load_n(current_jit_state, 2, Size::Long)
        }
        InstructionInfo::lload_3 => {
            load_n(current_jit_state, 3, Size::Long)
        }
        InstructionInfo::lmul => {
            integer_mul(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::lneg => Err(JITError::NotSupported),
        InstructionInfo::lookupswitch(_) => Err(JITError::NotSupported),
        InstructionInfo::lor => {
            binary_or(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::lrem => Err(JITError::NotSupported),
        InstructionInfo::lreturn => Err(JITError::NotSupported),
        InstructionInfo::lshl => {
            shift(current_jit_state, main_block, java_pc, Size::Long, ShiftDirection::ArithmeticLeft)
        }
        InstructionInfo::lshr => {
            shift(current_jit_state, main_block, java_pc, Size::Long, ShiftDirection::ArithmeticRight)
        }
        InstructionInfo::lstore(n) => {
            store_n(current_jit_state, *n as usize, Size::Long)
        }
        InstructionInfo::lstore_0 => {
            store_n(current_jit_state, 0, Size::Long)
        }
        InstructionInfo::lstore_1 => {
            store_n(current_jit_state, 1, Size::Long)
        }
        InstructionInfo::lstore_2 => {
            store_n(current_jit_state, 2, Size::Long)
        }
        InstructionInfo::lstore_3 => {
            store_n(current_jit_state, 3, Size::Long)
        }
        InstructionInfo::lsub => {
            integer_sub(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::lushr => {
            shift(current_jit_state, main_block, java_pc, Size::Long, ShiftDirection::LogicalRight)
        }
        InstructionInfo::lxor => {
            binary_xor(current_jit_state, main_block, Size::Long)
        }
        InstructionInfo::monitorenter => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::MonitorEnter));
            Ok(())
        }
        InstructionInfo::monitorexit => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::MonitorExit));
            Ok(())
        }
        InstructionInfo::multianewarray(_) => {
            main_block.add_instruction(IRInstruction::VMExit(VMExitType::MultiNewArray));
            Ok(())
        }
        InstructionInfo::new(_) => Err(JITError::NotSupported),
        InstructionInfo::newarray(_) => Err(JITError::NotSupported),
        InstructionInfo::nop => {
            Ok(())
        }
        InstructionInfo::pop => {
            Ok(())
        }
        InstructionInfo::pop2 => {
            Ok(())
        }
        InstructionInfo::putfield(_) => Err(JITError::NotSupported),
        InstructionInfo::putstatic(_) => Err(JITError::NotSupported),
        InstructionInfo::ret(_) => Err(JITError::NotSupported),
        InstructionInfo::return_ => Err(JITError::NotSupported),
        InstructionInfo::saload => {
            array_load(current_jit_state, main_block, Size::Short)
        }
        InstructionInfo::sastore => {
            array_store(current_jit_state, main_block, Size::Short)
        }
        InstructionInfo::sipush(_) => Err(JITError::NotSupported),
        InstructionInfo::swap => {
            swap(current_jit_state, main_block)
        },
        InstructionInfo::tableswitch(_) => Err(JITError::NotSupported),
        InstructionInfo::wide(_) => Err(JITError::NotSupported),
        InstructionInfo::EndOfCode => Err(JITError::NotSupported),
    }?
}

fn swap(current_jit_state: &mut JitState, main_block: &mut JitBlock) -> Result<(), JITError> {
    let a = current_jit_state.memory_layout.operand_stack_entry(current_jit_state.java_pc, 0);
    let b = current_jit_state.memory_layout.operand_stack_entry(current_jit_state.java_pc, 1);
    let temp = current_jit_state.memory_layout.safe_temp_location(current_jit_state.java_pc, 0);
    let copy_to_temp = IRInstruction::CopyRelative {
        input_offset: a,
        output_offset: temp,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    main_block.add_instruction(copy_to_temp);
    let b_to_a = IRInstruction::CopyRelative {
        input_offset: b,
        output_offset: a,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    main_block.add_instruction(b_to_a);
    let temp_to_b = IRInstruction::CopyRelative {
        input_offset: temp,
        output_offset: b,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    main_block.add_instruction(temp_to_b);
    Ok(())
}

pub mod arrays;
pub mod integer_arithmetic;

fn constant(current_jit_state: &mut JitState, constant: Constant) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc, .. } = current_jit_state;
    let null_offset = memory_layout.operand_stack_entry(*java_pc, 0);//todo this is wrong
    output.push(IRInstruction::Constant {
        output_offset: null_offset,
        constant,
    });
    Ok(())
}

fn aload_n(current_jit_state: &mut JitState, variable_index: usize) -> Result<(), JITError> {
    load_n(current_jit_state, variable_index, Size::Long)
}

fn load_n(current_jit_state: &mut JitState, variable_index: usize, size: Size) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc, next_pc } = current_jit_state;
    let local_var_offset = memory_layout.local_var_entry(*java_pc, variable_index);
    output.push(IRInstruction::CopyRelative {
        input_offset: local_var_offset,
        output_offset: memory_layout.operand_stack_entry(*next_pc.unwrap(), 0),
        input_size: size,
        output_size: size,
        signed: false,
    });
    Ok(())
}

fn astore_n(current_jit_state: &mut JitState, variable_index: usize) -> Result<(), JITError> {
    store_n(current_jit_state, variable_index, Size::Long)
}

fn store_n(current_jit_state: &mut JitState, variable_index: usize, size: Size) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc, next_pc } = current_jit_state;
    let local_var_offset = memory_layout.local_var_entry(*java_pc, variable_index);
    output.push(IRInstruction::CopyRelative {
        input_offset: memory_layout.operand_stack_entry(*java_pc.unwrap(), 0),
        output_offset: local_var_offset,
        input_size: size,
        output_size: size,
        signed: false,
    });
    Ok(())
}

pub struct JITedCode {
    code: Vec<CodeRegion>,
}

struct CodeRegion {
    raw: *mut c_void,
}

const MAX_CODE_SIZE: usize = 1_000_000;

impl JITedCode {
    pub unsafe fn add_code_region(&mut self, instructions: &[iced_x86::Instruction]) -> usize {
        let prot_flags = ProtFlags::PROT_EXEC | ProtFlags::PROT_WRITE | ProtFlags::PROT_READ;
        let flags = MapFlags::MAP_ANONYMOUS | MapFlags::MAP_NORESERVE | MapFlags::MAP_PRIVATE;
        let mmap_addr = mmap(transmute(0x1000000usize), MAX_CODE_SIZE, prot_flags, flags, -1, 0).unwrap();
        let rip_start = mmap_addr as u64;

        let block = InstructionBlock::new(instructions, rip_start as u64);
        let BlockEncoderResult { mut code_buffer, .. } = BlockEncoder::encode(64, block, BlockEncoderOptions::NONE).unwrap();
        let len_before = self.code.len();

        if code_buffer.len() > MAX_CODE_SIZE {
            panic!("exceeded max code size");
        }

        libc::memcpy(mmap_addr, code_buffer.as_ptr() as *const c_void, code_buffer.len());

        self.code.push(CodeRegion {
            raw: mmap_addr as *mut c_void
        });
        fence(Ordering::SeqCst);
        // __clear_cache();//todo should use this
        return len_before;
    }

    pub unsafe fn run_jitted_coded(&self, id: usize) {
        let as_ptr = self.code[id].raw;
        let as_num = as_ptr as u64;
        let rust_stack: u64 = 0xdeadbeaf;
        let rust_frame: u64 = 0xdeadbeaf;
        let jit_code_context = JitCodeContext {
            previous_stack: 0xdeaddeaddeaddead
        };
        let jit_context_pointer = &jit_code_context as *const JitCodeContext as u64;
        asm!(
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        // technically these need only be saved on windows
        //todo perhaps should use pusha/popa here, b/c this must be really slow
        // "push xmm6",
        // "push xmm7",
        // "push xmm8",
        // "push xmm9",
        // "push xmm10",
        // "push xmm11",
        // "push xmm12",
        // "push xmm13",
        // "push xmm14",
        // "push xmm15",
        "push rsp",
        //todo need to setup rsp and frame pointer for java stack
        "nop",
        // load java frame pointer
        "mov rbp, {1}",
        // store old stack pointer into context
        "mov [{3}],rsp",
        // load java stack pointer
        "mov rsp, {2}",
        // load context pointer into r15
        "mov r15,{3}",
        // jump to jitted code
        "jmp {0}",
        "pop rsp",
        // "pop xmm15",
        // "pop xmm14",
        // "pop xmm13",
        // "pop xmm12",
        // "pop xmm11",
        // "pop xmm10",
        // "pop xmm9",
        // "pop xmm8",
        // "pop xmm7",
        // "pop xmm6",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        in(reg) as_num,
        in(reg) rust_frame,
        in(reg) rust_stack,
        in(reg) jit_context_pointer
        );

        todo!("need to get return val")
    }
}

#[repr(C)]
pub struct JitCodeContext {
    previous_stack: u64,
}


#[cfg(test)]
pub mod test {
    use iced_x86::{Formatter, Instruction, InstructionBlock, IntelFormatter};

    use gc_memory_layout_common::FramePointerOffset;
    use jit_ir::{IRInstruction, Size};

    use crate::JITedCode;

    #[test]
    pub fn test() {
        let mut instructions: Vec<Instruction> = vec![];
        IRInstruction::LoadAbsolute { address_from: FramePointerOffset(10), output_offset: FramePointerOffset(10), size: Size::Int }.to_x86(&mut instructions);
        let mut formatter = IntelFormatter::new();
        let mut res = String::new();
        for instruction in &instructions {
            formatter.format(instruction, &mut res);
            res.push_str("\n")
        }
        println!("{}", res);
        let mut jitted_code = JITedCode {
            code: vec![]
        };
        let id = unsafe { jitted_code.add_code_region(instructions.as_slice()) };
        unsafe { jitted_code.run_jitted_coded(id); }
    }

    #[test]
    pub fn test2() {}
}


