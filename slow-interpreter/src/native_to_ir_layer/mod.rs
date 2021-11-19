use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::RwLock;

use bimap::BiHashMap;
use iced_x86::{BlockEncoder, BlockEncoderOptions, Formatter, InstructionBlock, IntelFormatter};
use iced_x86::CC_b::c;
use iced_x86::code_asm::{CodeAssembler, CodeLabel, qword_ptr, rax, rbp};
use itertools::Itertools;
use libc::{MAP_ANONYMOUS, MAP_GROWSDOWN, MAP_NORESERVE, MAP_PRIVATE, PROT_READ, PROT_WRITE, select};
use memoffset::offset_of;

use another_jit_vm::{BaseAddress, Method, MethodImplementationID, SavedRegistersWithoutIP, VMExitAction, VMExitEvent, VMState};
use classfile_parser::code::InstructionTypeNum::new;

use crate::gc_memory_layout_common::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use crate::jit::ir::IRInstr;
use crate::jit::LabelName;
use crate::method_table::MethodId;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct IRMethodID(usize);

pub struct IRVMStateInner {
    // each IR function is distinct single java methods may many ir methods
    ir_method_id_max: IRMethodID,
    current_implementation: BiHashMap<IRMethodID, MethodImplementationID>,
    method_ir_offsets: HashMap<IRMethodID, BiHashMap<IRInstructOffset, IRInstructIndex>>,
    method_ir: HashMap<IRMethodID, Vec<IRInstr>>,// index
    // function_ir_mapping: HashMap<IRMethodID, !>,
}

impl IRVMStateInner {
    pub fn new() -> Self {
        Self {
            ir_method_id_max: IRMethodID(0),
            current_implementation: Default::default(),
            method_ir_offsets: Default::default(),
            method_ir: Default::default(),
        }
    }

    pub fn add_function_ir_offsets(&mut self, current_ir_id: IRMethodID, new_instruction_offsets: Vec<IRInstructOffset>, assembly_index_to_ir_instruct_index: HashMap<AssemblyInstructionIndex, IRInstructIndex>) {
        let mut res = BiHashMap::new();
        for (i, instruction_offset) in new_instruction_offsets.into_iter().enumerate() {
            let assembly_instruction_index = AssemblyInstructionIndex(i);
            let ir_instruction_index = assembly_index_to_ir_instruct_index.get(&assembly_instruction_index).unwrap();
            res.insert(instruction_offset, *ir_instruction_index);
        }
        self.method_ir_offsets.insert(current_ir_id, res);
    }
}

pub struct IRVMState<'vm_state_life> {
    native_vm: VMState<'vm_state_life, u64>,
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

impl IRVMState<'vm_state_life> {
    pub fn new() -> Self {
        Self {
            native_vm: VMState::new(),
            inner: RwLock::new(IRVMStateInner::new()),
        }
    }

    pub fn run_method(&self, method_id: IRMethodID) -> u64 {
        let inner_read_guard = self.inner.read().unwrap();
        let current_implementation = *inner_read_guard.current_implementation.get_by_left(&method_id).unwrap();
        //todo for now we launch with zeroed registers, in future we may need to map values to stack or something

        self.native_vm.launch_vm(current_implementation, SavedRegistersWithoutIP::new_with_all_zero())
    }

    fn debug_print_instructions(assembler: &CodeAssembler) {
        let mut formatted_instructions = String::new();
        let mut formatter = IntelFormatter::default();
        for (i, instruction) in assembler.instructions().iter().enumerate() {
            formatter.format(instruction, &mut formatted_instructions);
        }
        eprintln!("{}", formatted_instructions);
    }

    pub fn add_function(&'vm_state_life self, instructions: Vec<IRInstr>, ir_exit_handler: Box<dyn Fn(&IRVMExitEvent) -> VMExitAction<u64> + 'vm_state_life>) -> IRMethodID {
        let mut inner_guard = self.inner.write().unwrap();
        let current_ir_id = inner_guard.ir_method_id_max;
        inner_guard.ir_method_id_max.0 += 1;
        let mut assembler = CodeAssembler::new(64).unwrap();
        let (code_assembler, assembly_index_to_ir_instruct_index) = add_function_from_ir(instructions);
        Self::debug_print_instructions(&assembler);
        let base_address = self.native_vm.get_new_base_address();
        let block = InstructionBlock::new(assembler.instructions(), base_address.0 as u64);
        let result = BlockEncoder::encode(assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
        let new_instruction_offsets = result.new_instruction_offsets.into_iter().map(|new_instruction_offset| IRInstructOffset(new_instruction_offset as usize)).collect_vec();
        inner_guard.add_function_ir_offsets(current_ir_id, new_instruction_offsets, assembly_index_to_ir_instruct_index);
        let vm_exit_handler: Box<dyn Fn(&VMExitEvent) -> VMExitAction<u64> + 'vm_state_life> = box move |vm_exit_event: &VMExitEvent| {
            vm_exit_handler(self, vm_exit_event, ir_exit_handler.deref())
        };
        let code = result.code_buffer;

        let method_implementation_id = self.native_vm.add_method_implementation(Method {
            code,
            exit_handler: vm_exit_handler,
        }, base_address);
        inner_guard.current_implementation.insert(current_ir_id, method_implementation_id);
        current_ir_id
    }
}

fn vm_exit_handler(ir_vm_state: &'vm_state_life IRVMState<'vm_state_life>, vm_exit_event: &VMExitEvent, ir_exit_handler: &(dyn Fn(&IRVMExitEvent) -> VMExitAction<u64> + 'vm_state_life)) -> VMExitAction<u64> {
    let implementation_id = vm_exit_event.method;
    let exit_address = vm_exit_event.saved_guest_registers.rip;
    let exit_method_base_address = vm_exit_event.method_base_address;
    let offset = unsafe { exit_method_base_address.offset_from(exit_address) };
    if offset < 0 {
        panic!()
    }
    let offset = IRInstructOffset(offset as usize);
    assert!(offset.0 < 1024 * 1024);// methods over a megabyte prob aren't a thing
    let inner_read_guard = ir_vm_state.inner.read().unwrap();
    let ir_method_id = *inner_read_guard.current_implementation.get_by_right(&implementation_id).unwrap();
    let ir_instruct_index = inner_read_guard.method_ir_offsets.get(&ir_method_id).unwrap().get_by_left(&offset).unwrap();


    let ir_vm_exit_event = IRVMExitEvent {
        inner: &vm_exit_event,
        ir_method: ir_method_id,
        ir_instruct: *ir_instruct_index,
    };

    ir_exit_handler(&ir_vm_exit_event)
}


fn add_function_from_ir(instructions: Vec<IRInstr>) -> (CodeAssembler, HashMap<AssemblyInstructionIndex, IRInstructIndex>) {
    let mut assembler = CodeAssembler::new(64).unwrap();
    let mut res = HashMap::new();
    let mut labels = HashMap::new();
    for (i, instruction) in instructions.into_iter().enumerate() {
        let assembly_instruction_index = AssemblyInstructionIndex(assembler.instructions().len());
        let ir_instruction_index = IRInstructIndex(i);
        res.insert(assembly_instruction_index, ir_instruction_index);
        single_ir_to_native(&mut assembler, instruction, &mut labels);
    }
    (assembler, res)
}

fn single_ir_to_native(assembler: &mut CodeAssembler, instruction: IRInstr, labels: &mut HashMap<LabelName,CodeLabel>) {
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
        IRInstr::BranchToLabel { label } => {
            let code_label = labels.entry(label).or_insert_with(||assembler.create_label());
            assembler.jmp(code_label.clone()).unwrap();
        },
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
        IRInstr::Label(label) => {
            let label_name = label.name;
            let code_label = labels.entry(label_name).or_insert_with(||assembler.create_label());
            assembler.set_label(code_label);
            assembler.nop().unwrap();
        },
        IRInstr::IRNewFrame {
            current_frame_size,
            temp_register,
            return_to_rip
        } => {
            let return_to_rip = return_to_rip.to_native_64();
            let temp_register = temp_register.to_native_64();
            assembler.mov(temp_register, MAGIC_1_EXPECTED).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_1_OFFSET) as u64, temp_register).unwrap();
            assembler.mov(temp_register, MAGIC_2_EXPECTED).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_2_OFFSET) as u64, temp_register).unwrap();

            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RBP_OFFSET) as u64, rbp).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RIP_OFFSET) as u64, return_to_rip).unwrap();
        }
    }
}


//index is an index, offset is a byte offset from method start

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct IRInstructIndex(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct IRInstructOffset(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct AssemblyInstructionIndex(usize);


pub struct IRVMExitEvent<'l> {
    pub inner: &'l VMExitEvent,
    pub ir_method: IRMethodID,
    pub ir_instruct: IRInstructIndex,
}