#![feature(asm)]

extern crate compiler_builtins;

use std::ffi::c_void;
use std::mem::transmute;
use std::panic::panic_any;
use std::ptr::null_mut;
use std::sync::atomic::{fence, Ordering};

use iced_x86::{BlockEncoder, BlockEncoderOptions, BlockEncoderResult, Encoder, InstructionBlock};
use iced_x86::ConditionCode::s;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use jit_ir::{Constant, IRInstruction, Size, VariableSize};
use rust_jvm_common::classfile::{Instruction, InstructionInfo};

pub enum JITError {
    NotSupported
}

// pub struct Label{
//     id: usize,
//     bytecode_index: usize,
//     true_index: usize,
// }

pub struct JitState {
    memory_layout: StackframeMemoryLayout,
    pub java_pc: usize,
    output: Vec<IRInstruction>,
}


pub fn byte_code_to_ir(bytecode: &Instruction, current_jit_state: &mut JitState) -> Result<(), JITError> {
    let Instruction { offset, instruction: instruction_info } = bytecode;
    current_jit_state.java_pc = *offset;
    match instruction_info {
        InstructionInfo::aaload => Err(JITError::NotSupported),
        InstructionInfo::aastore => Err(JITError::NotSupported),
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
            current_jit_state.output.push(IRInstruction::Return { return_value: Some(FramePointerOffset(todo!())), to_pop: VariableSize(current_jit_state.memory_layout.full_frame_size()) });
            Ok(())
        }
        InstructionInfo::arraylength => Err(JITError::NotSupported),
        InstructionInfo::astore(_) => Err(JITError::NotSupported),
        InstructionInfo::astore_0 => Err(JITError::NotSupported),
        InstructionInfo::astore_1 => Err(JITError::NotSupported),
        InstructionInfo::astore_2 => Err(JITError::NotSupported),
        InstructionInfo::astore_3 => Err(JITError::NotSupported),
        InstructionInfo::athrow => Err(JITError::NotSupported),
        InstructionInfo::baload => Err(JITError::NotSupported),
        InstructionInfo::bastore => Err(JITError::NotSupported),
        InstructionInfo::bipush(_) => Err(JITError::NotSupported),
        InstructionInfo::caload => Err(JITError::NotSupported),
        InstructionInfo::castore => Err(JITError::NotSupported),
        InstructionInfo::checkcast(_) => Err(JITError::NotSupported),
        InstructionInfo::d2f => Err(JITError::NotSupported),
        InstructionInfo::d2i => Err(JITError::NotSupported),
        InstructionInfo::d2l => Err(JITError::NotSupported),
        InstructionInfo::dadd => Err(JITError::NotSupported),
        InstructionInfo::daload => Err(JITError::NotSupported),
        InstructionInfo::dastore => Err(JITError::NotSupported),
        InstructionInfo::dcmpg => Err(JITError::NotSupported),
        InstructionInfo::dcmpl => Err(JITError::NotSupported),
        InstructionInfo::dconst_0 => Err(JITError::NotSupported),
        InstructionInfo::dconst_1 => Err(JITError::NotSupported),
        InstructionInfo::ddiv => Err(JITError::NotSupported),
        InstructionInfo::dload(_) => Err(JITError::NotSupported),
        InstructionInfo::dload_0 => Err(JITError::NotSupported),
        InstructionInfo::dload_1 => Err(JITError::NotSupported),
        InstructionInfo::dload_2 => Err(JITError::NotSupported),
        InstructionInfo::dload_3 => Err(JITError::NotSupported),
        InstructionInfo::dmul => Err(JITError::NotSupported),
        InstructionInfo::dneg => Err(JITError::NotSupported),
        InstructionInfo::drem => Err(JITError::NotSupported),
        InstructionInfo::dreturn => Err(JITError::NotSupported),
        InstructionInfo::dstore(_) => Err(JITError::NotSupported),
        InstructionInfo::dstore_0 => Err(JITError::NotSupported),
        InstructionInfo::dstore_1 => Err(JITError::NotSupported),
        InstructionInfo::dstore_2 => Err(JITError::NotSupported),
        InstructionInfo::dstore_3 => Err(JITError::NotSupported),
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
        InstructionInfo::faload => Err(JITError::NotSupported),
        InstructionInfo::fastore => Err(JITError::NotSupported),
        InstructionInfo::fcmpg => Err(JITError::NotSupported),
        InstructionInfo::fcmpl => Err(JITError::NotSupported),
        InstructionInfo::fconst_0 => Err(JITError::NotSupported),
        InstructionInfo::fconst_1 => Err(JITError::NotSupported),
        InstructionInfo::fconst_2 => Err(JITError::NotSupported),
        InstructionInfo::fdiv => Err(JITError::NotSupported),
        InstructionInfo::fload(_) => Err(JITError::NotSupported),
        InstructionInfo::fload_0 => Err(JITError::NotSupported),
        InstructionInfo::fload_1 => Err(JITError::NotSupported),
        InstructionInfo::fload_2 => Err(JITError::NotSupported),
        InstructionInfo::fload_3 => Err(JITError::NotSupported),
        InstructionInfo::fmul => Err(JITError::NotSupported),
        InstructionInfo::fneg => Err(JITError::NotSupported),
        InstructionInfo::frem => Err(JITError::NotSupported),
        InstructionInfo::freturn => Err(JITError::NotSupported),
        InstructionInfo::fstore(_) => Err(JITError::NotSupported),
        InstructionInfo::fstore_0 => Err(JITError::NotSupported),
        InstructionInfo::fstore_1 => Err(JITError::NotSupported),
        InstructionInfo::fstore_2 => Err(JITError::NotSupported),
        InstructionInfo::fstore_3 => Err(JITError::NotSupported),
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
        InstructionInfo::iadd => Err(JITError::NotSupported),
        InstructionInfo::iaload => Err(JITError::NotSupported),
        InstructionInfo::iand => Err(JITError::NotSupported),
        InstructionInfo::iastore => Err(JITError::NotSupported),
        InstructionInfo::iconst_m1 => Err(JITError::NotSupported),
        InstructionInfo::iconst_0 => Err(JITError::NotSupported),
        InstructionInfo::iconst_1 => Err(JITError::NotSupported),
        InstructionInfo::iconst_2 => Err(JITError::NotSupported),
        InstructionInfo::iconst_3 => Err(JITError::NotSupported),
        InstructionInfo::iconst_4 => Err(JITError::NotSupported),
        InstructionInfo::iconst_5 => Err(JITError::NotSupported),
        InstructionInfo::idiv => Err(JITError::NotSupported),
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
        InstructionInfo::iload(_) => Err(JITError::NotSupported),
        InstructionInfo::iload_0 => Err(JITError::NotSupported),
        InstructionInfo::iload_1 => Err(JITError::NotSupported),
        InstructionInfo::iload_2 => Err(JITError::NotSupported),
        InstructionInfo::iload_3 => Err(JITError::NotSupported),
        InstructionInfo::imul => Err(JITError::NotSupported),
        InstructionInfo::ineg => Err(JITError::NotSupported),
        InstructionInfo::instanceof(_) => Err(JITError::NotSupported),
        InstructionInfo::invokedynamic(_) => Err(JITError::NotSupported),
        InstructionInfo::invokeinterface(_) => Err(JITError::NotSupported),
        InstructionInfo::invokespecial(_) => Err(JITError::NotSupported),
        InstructionInfo::invokestatic(_) => Err(JITError::NotSupported),
        InstructionInfo::invokevirtual(_) => Err(JITError::NotSupported),
        InstructionInfo::ior => Err(JITError::NotSupported),
        InstructionInfo::irem => Err(JITError::NotSupported),
        InstructionInfo::ireturn => Err(JITError::NotSupported),
        InstructionInfo::ishl => Err(JITError::NotSupported),
        InstructionInfo::ishr => Err(JITError::NotSupported),
        InstructionInfo::istore(_) => Err(JITError::NotSupported),
        InstructionInfo::istore_0 => Err(JITError::NotSupported),
        InstructionInfo::istore_1 => Err(JITError::NotSupported),
        InstructionInfo::istore_2 => Err(JITError::NotSupported),
        InstructionInfo::istore_3 => Err(JITError::NotSupported),
        InstructionInfo::isub => Err(JITError::NotSupported),
        InstructionInfo::iushr => Err(JITError::NotSupported),
        InstructionInfo::ixor => Err(JITError::NotSupported),
        InstructionInfo::jsr(_) => Err(JITError::NotSupported),
        InstructionInfo::jsr_w(_) => Err(JITError::NotSupported),
        InstructionInfo::l2d => Err(JITError::NotSupported),
        InstructionInfo::l2f => Err(JITError::NotSupported),
        InstructionInfo::l2i => Err(JITError::NotSupported),
        InstructionInfo::ladd => Err(JITError::NotSupported),
        InstructionInfo::laload => Err(JITError::NotSupported),
        InstructionInfo::land => Err(JITError::NotSupported),
        InstructionInfo::lastore => Err(JITError::NotSupported),
        InstructionInfo::lcmp => Err(JITError::NotSupported),
        InstructionInfo::lconst_0 => Err(JITError::NotSupported),
        InstructionInfo::lconst_1 => Err(JITError::NotSupported),
        InstructionInfo::ldc(_) => Err(JITError::NotSupported),
        InstructionInfo::ldc_w(_) => Err(JITError::NotSupported),
        InstructionInfo::ldc2_w(_) => Err(JITError::NotSupported),
        InstructionInfo::ldiv => Err(JITError::NotSupported),
        InstructionInfo::lload(_) => Err(JITError::NotSupported),
        InstructionInfo::lload_0 => Err(JITError::NotSupported),
        InstructionInfo::lload_1 => Err(JITError::NotSupported),
        InstructionInfo::lload_2 => Err(JITError::NotSupported),
        InstructionInfo::lload_3 => Err(JITError::NotSupported),
        InstructionInfo::lmul => Err(JITError::NotSupported),
        InstructionInfo::lneg => Err(JITError::NotSupported),
        InstructionInfo::lookupswitch(_) => Err(JITError::NotSupported),
        InstructionInfo::lor => Err(JITError::NotSupported),
        InstructionInfo::lrem => Err(JITError::NotSupported),
        InstructionInfo::lreturn => Err(JITError::NotSupported),
        InstructionInfo::lshl => Err(JITError::NotSupported),
        InstructionInfo::lshr => Err(JITError::NotSupported),
        InstructionInfo::lstore(_) => Err(JITError::NotSupported),
        InstructionInfo::lstore_0 => Err(JITError::NotSupported),
        InstructionInfo::lstore_1 => Err(JITError::NotSupported),
        InstructionInfo::lstore_2 => Err(JITError::NotSupported),
        InstructionInfo::lstore_3 => Err(JITError::NotSupported),
        InstructionInfo::lsub => Err(JITError::NotSupported),
        InstructionInfo::lushr => Err(JITError::NotSupported),
        InstructionInfo::lxor => Err(JITError::NotSupported),
        InstructionInfo::monitorenter => Err(JITError::NotSupported),
        InstructionInfo::monitorexit => Err(JITError::NotSupported),
        InstructionInfo::multianewarray(_) => Err(JITError::NotSupported),
        InstructionInfo::new(_) => Err(JITError::NotSupported),
        InstructionInfo::newarray(_) => Err(JITError::NotSupported),
        InstructionInfo::nop => Err(JITError::NotSupported),
        InstructionInfo::pop => Err(JITError::NotSupported),
        InstructionInfo::pop2 => Err(JITError::NotSupported),
        InstructionInfo::putfield(_) => Err(JITError::NotSupported),
        InstructionInfo::putstatic(_) => Err(JITError::NotSupported),
        InstructionInfo::ret(_) => Err(JITError::NotSupported),
        InstructionInfo::return_ => Err(JITError::NotSupported),
        InstructionInfo::saload => Err(JITError::NotSupported),
        InstructionInfo::sastore => Err(JITError::NotSupported),
        InstructionInfo::sipush(_) => Err(JITError::NotSupported),
        InstructionInfo::swap => Err(JITError::NotSupported),
        InstructionInfo::tableswitch(_) => Err(JITError::NotSupported),
        InstructionInfo::wide(_) => Err(JITError::NotSupported),
        InstructionInfo::EndOfCode => Err(JITError::NotSupported),
    }
}

fn constant(current_jit_state: &mut JitState, constant: Constant) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc } = current_jit_state;
    let null_offset = memory_layout.operand_stack_entry(*java_pc, 0);
    output.push(IRInstruction::StoreConstant {
        address_to: null_offset,
        constant,
    });
    Ok(())
}

fn aload_n(current_jit_state: &mut JitState, variable_index: usize) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc } = current_jit_state;
    let local_var_offset = memory_layout.local_var_entry(*java_pc, variable_index);
    output.push(IRInstruction::StoreAbsolute {
        address_to: memory_layout.operand_stack_entry(*java_pc, todo!()),
        input_offset: local_var_offset,
        size: Size::Long,
    });
    Ok(())
}

fn astore_n(current_jit_state: &mut JitState, variable_index: usize) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc } = current_jit_state;
    let local_var_offset = memory_layout.local_var_entry(*java_pc, variable_index);
    output.push(IRInstruction::LoadAbsolute {
        output_offset: todo!(),
        address_from: todo!(),
        size: Size::Long,
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
        IRInstruction::LoadAbsolute { address_from: FramePointerOffset(10), output_offset: FramePointerOffset(10), size: Size::Long }.to_x86(&mut instructions);
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
}


