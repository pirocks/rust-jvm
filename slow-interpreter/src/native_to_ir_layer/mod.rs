use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ptr::null_mut;
use std::sync::RwLock;

use iced_x86::{BlockEncoder, BlockEncoderOptions, Formatter, InstructionBlock, IntelFormatter};
use iced_x86::code_asm::{CodeAssembler, qword_ptr, rbp};
use libc::{MAP_ANONYMOUS, MAP_GROWSDOWN, MAP_NORESERVE, MAP_PRIVATE, PROT_READ, PROT_WRITE, select};
use memoffset::offset_of;

use another_jit_vm::{Method, MethodImplementationID, SavedRegistersWithoutIP, VMState};
use classfile_parser::code::InstructionTypeNum::new;

use crate::gc_memory_layout_common::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use crate::jit::ir::IRInstr;
use crate::method_table::MethodId;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct IRMethodID(usize);

pub struct IRVMStateInner {
    // each IR function is distinct single java methods may many ir methods
    current_implementation: HashMap<IRMethodID, MethodImplementationID>,
    // function_ir_mapping: HashMap<IRMethodID, !>,
}

pub struct IRVMState {
    native_vm: VMState<u64>,
    inner: RwLock<IRVMStateInner>,
}

// IR knows about stack so we should have a stack
// will have IR instruct for new frame, so IR also knows about frames
pub struct IRStack {
    mmaped_top: *mut c_void,
}


pub struct UnPackedIRFrameHeader {
    prev_rip: *mut c_void,
    prev_rbp: *mut c_void,
    ignored_java_data1: Option<*mut c_void>,
    ignored_java_data2: Option<*mut c_void>,
    magic_1: u64,
    magic_2: u64,
    // as above but grows down
    // magic_2: *mut c_void,
    // magic_1: *mut c_void,
    // ignored_java_data2: *mut c_void,
    // ignored_java_data1: *mut c_void,
    // prev_rbp: *mut c_void,
    // prev_rip: *mut c_void,
}

pub const FRAME_HEADER_PREV_RIP_OFFSET: usize = 0;
pub const FRAME_HEADER_PREV_RBP_OFFSET: usize = 8;
pub const FRAME_HEADER_PREV_MAGIC_1_OFFSET: usize = 32;
pub const FRAME_HEADER_PREV_MAGIC_2_OFFSET: usize = 40;

impl IRStack {
    pub fn new() -> Self {
        pub const MAX_STACK: usize = 1024 * 1024 * 1024;
        let mmaped_top = unsafe { libc::mmap(null_mut(), MAX_STACK, PROT_READ | PROT_WRITE, MAP_NORESERVE | MAP_PRIVATE | MAP_ANONYMOUS | MAP_GROWSDOWN, -1, 0) };
        Self {
            mmaped_top
        }
    }

    unsafe fn read_frame_ir_header(frame_pointer: *const c_void) -> UnPackedIRFrameHeader {
        let rip_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_RIP_OFFSET as isize)) as *const *mut c_void;
        let rbp_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_RBP_OFFSET as isize)) as *const *mut c_void;
        let magic1_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_MAGIC_1_OFFSET as isize)) as *const u64;
        let magic2_ptr = frame_pointer.offset(-(FRAME_HEADER_PREV_MAGIC_2_OFFSET as isize)) as *const u64;
        let magic_1 = magic1_ptr.read();
        let magic_2 = magic2_ptr.read();
        assert_eq!(magic_1, MAGIC_1_EXPECTED);
        assert_eq!(magic_2, MAGIC_2_EXPECTED);
        UnPackedIRFrameHeader {
            prev_rip: rip_ptr.read(),
            prev_rbp: rbp_ptr.read(),
            ignored_java_data1: None,
            ignored_java_data2: None,
            magic_1,
            magic_2,
        }
    }
}


//iters up from position on stack
pub struct IRStackIter {
    current_rbp: *mut c_void,
    top: *mut c_void,
}

impl Iterator for IRStackIter {
    type Item = IRStackEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let frame_pointer = self.current_rbp;
        let ir_stack_read = unsafe { IRStack::read_frame_ir_header(frame_pointer) };
        self.current_rbp = ir_stack_read.prev_rbp;
        Some(IRStackEntry {
            rbp: frame_pointer
        })
    }
}

pub struct IRStackEntry {
    rbp: *mut c_void,
}

impl IRVMState {
    pub fn new() -> Self {
        Self {
            native_vm: VMState::new(),
            inner: RwLock::new(IRVMStateInner {
                current_implementation: Default::default(),
                // function_ir_mapping: Default::default(),
            }),
        }
    }

    pub fn run_method(&self, method_id: IRMethodID) -> u64 {
        let inner_read_guard = self.inner.read().unwrap();
        let current_implemntation = inner_read_guard.current_implementation.get(&method_id).unwrap();
        //todo for now we launch with zeroed registers, in future we may need to map values to stack or something

        self.native_vm.launch_vm(*current_implemntation, SavedRegistersWithoutIP::new_with_all_zero())
    }

    fn debug_print_instructions(block: &InstructionBlock) {
        let mut formatted_instructions = String::new();
        let mut formatter = IntelFormatter::default();
        for (i, instruction) in assembler.instructions().iter().enumerate() {
            formatter.format(instruction, &mut formatted_instructions);
        }
        eprintln!("{}", format!("{} :\n{}", method_log_info, formatted_instructions));
    }

    pub fn add_function(&self, instructions: Vec<IRInstr>) -> IRMethodID {
        let mut inner_guard = self.inner.write().unwrap();
        let next_id = MethodId(inner_guard.current_implementation.len());
        let mut assembler = CodeAssembler::new(64).unwrap();
        Self::add_function_from_ir(&mut assembler, instructions);
        let block = InstructionBlock::new(assembler.instructions(), base_address as u64);
        Self::debug_print_instructions(&block);

        let result = BlockEncoder::encode(assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();

        self.native_vm.add_method_implementation(Method {
            code,
            exit_handler,
        });
        inner_guard.current_implementation.get(&)
        todo!()
    }

    fn add_function_from_ir(assembler: &mut CodeAssembler, instructions: Vec<IRInstr>) {
        for (i, instruction) in instructions.into_iter().enumerate() {
            match instruction {
                IRInstr::LoadFPRelative { from, to } => {
                    //stack grows down
                    assembler.mov(to.to_native_64(), rbp - from.0).unwrap();
                }
                IRInstr::StoreFPRelative { from, to } => {
                    assembler.mov(qword_ptr(rbp - to.0), from.to_native_64()).unwrap();
                }
                IRInstr::Load { .. } => todo!(),
                IRInstr::Store { .. } => todo!(),
                IRInstr::CopyRegister { .. } => todo!(),
                IRInstr::Add { .. } => todo!(),
                IRInstr::Sub { .. } => todo!(),
                IRInstr::Div { .. } => todo!(),
                IRInstr::Mod { .. } => todo!(),
                IRInstr::Mul { .. } => todo!(),
                IRInstr::BinaryBitAnd { .. } => todo!(),
                IRInstr::ForwardBitScan { .. } => todo!(),
                IRInstr::Const32bit { .. } => todo!(),
                IRInstr::Const64bit { .. } => todo!(),
                IRInstr::BranchToLabel { .. } => todo!(),
                IRInstr::LoadLabel { .. } => todo!(),
                IRInstr::LoadRBP { .. } => todo!(),
                IRInstr::WriteRBP { .. } => todo!(),
                IRInstr::BranchEqual { .. } => todo!(),
                IRInstr::BranchNotEqual { .. } => todo!(),
                IRInstr::Return { .. } => todo!(),
                IRInstr::VMExit { .. } => todo!(),
                IRInstr::GrowStack { .. } => todo!(),
                IRInstr::LoadSP { .. } => todo!(),
                IRInstr::WithAssembler { .. } => todo!(),
                IRInstr::FNOP => todo!(),
                IRInstr::Label(_) => todo!(),
                IRInstr::IRNewFrame {
                    current_frame_size,
                    return_to_rip
                } => {
                    let return_to_rip = return_to_rip.to_native_64();
                    assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_1_OFFSET), MAGIC_1_EXPECTED).unwrap();
                    assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_2_OFFSET), MAGIC_2_EXPECTED).unwrap();

                    assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RBP_OFFSET), rbp).unwrap();
                    assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RIP_OFFSET), return_to_rip).unwrap();
                }
            }
        }
        todo!()
    }

    fn single_ir_to_native(assembler: &mut CodeAssembler) {}
}