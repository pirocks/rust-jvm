#![feature(in_band_lifetimes)]
#![feature(step_trait)]
#![feature(box_syntax)]

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::ops::Deref;
use std::sync::RwLock;

use bimap::BiHashMap;
use iced_x86::{BlockEncoder, BlockEncoderOptions, Formatter, InstructionBlock, IntelFormatter};
use iced_x86::code_asm::{CodeAssembler, CodeLabel, qword_ptr, rax, rbp, rbx, rsp};
use itertools::Itertools;

use another_jit_vm::{BaseAddress, MethodImplementationID, NativeInstructionLocation, Register, SavedRegistersWithoutIP, VMExitAction, VMExitEvent, VMState};
use another_jit_vm::stack::OwnedNativeStack;
use compiler::{IRInstr, LabelName, RestartPointID};
use gc_memory_layout_common::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use ir_stack::{FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET, FRAME_HEADER_PREV_RIP_OFFSET, IRStack, OPAQUE_FRAME_SIZE};

use crate::ir_stack::IRStackMut;
use crate::vm_exit_abi::{IRVMExitType, RuntimeVMExitInput};

#[cfg(test)]
pub mod tests;
pub mod compiler;
pub mod vm_exit_abi;


#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRMethodID(pub usize);

pub struct IRVMStateInner {
    // each IR function is distinct single java methods may many ir methods
    ir_method_id_max: IRMethodID,
    top_level_return_function_id: Option<IRMethodID>,
    current_implementation: BiHashMap<IRMethodID, MethodImplementationID>,
    frame_sizes_by_address: HashMap<*const c_void, usize>,
    //todo not used currently
    frame_sizes_by_ir_method_id: HashMap<IRMethodID, usize>,
    method_ir_offsets: HashMap<IRMethodID, BiHashMap<IRInstructNativeOffset, IRInstructIndex>>,
    method_ir: HashMap<IRMethodID, Vec<IRInstr>>,
    // index
    opaque_method_to_or_method_id: HashMap<u64, IRMethodID>,
    // function_ir_mapping: HashMap<IRMethodID, !>,
}

impl IRVMStateInner {
    pub fn new() -> Self {
        Self {
            ir_method_id_max: IRMethodID(0),
            top_level_return_function_id: None,
            current_implementation: Default::default(),
            frame_sizes_by_address: Default::default(),
            frame_sizes_by_ir_method_id: Default::default(),
            method_ir_offsets: Default::default(),
            method_ir: Default::default(),
            opaque_method_to_or_method_id: Default::default(),
        }
    }

    pub fn add_function_ir_offsets(&mut self, current_ir_id: IRMethodID,
                                   new_instruction_offsets: Vec<IRInstructNativeOffset>,
                                   ir_instruct_index_to_assembly_index: Vec<(IRInstructIndex, AssemblyInstructionIndex)>) {
        let mut res = BiHashMap::new();//todo these bihashmaps are dangerous, should assert nothing is ever overwritten
        for ((i, instruction_offset), (ir_instruction_index, assembly_instruction_index_2)) in new_instruction_offsets.into_iter().enumerate().zip(ir_instruct_index_to_assembly_index.into_iter()) {
            let assembly_instruction_index_1 = AssemblyInstructionIndex(i);
            assert_eq!(assembly_instruction_index_1, assembly_instruction_index_2);
            if let Some(ir_instruction_offset) = res.get_by_right(&ir_instruction_index) {
                if *ir_instruction_offset > instruction_offset {
                    res.insert(instruction_offset, ir_instruction_index);
                    panic!("don't expect this to actually be needed")
                }
            } else {
                let overwritten = res.insert(instruction_offset, ir_instruction_index);
                assert!(!overwritten.did_overwrite());
            }
        }
        let indexes = res.iter().map(|(_, instruct)| *instruct).collect::<HashSet<_>>();
        assert_eq!(indexes.iter().max().unwrap().0 + 1, indexes.len());
        self.method_ir_offsets.insert(current_ir_id, res);
    }
}

pub mod ir_stack;
// thread_local! {
//     static THIS_THREAD_STACK: RefCell<IRStack> = RefCell::new(IRStack::new());
//     static THIS_THREAD_STACK_POINTER: RefCell<Option<*const c_void>> = RefCell::new(None);
//     static THIS_THREAD_FRAME_POINTER: RefCell<Option<*const c_void>> = RefCell::new(None);
// }

pub struct IRVMState<'vm_life, ExtraData: 'vm_life> {
    native_vm: VMState<'vm_life, u64, (ExtraData, &'vm_life Self)/*(Arc<JavaThread<'vm_life>>, JavaStackPosition, &'vm_life JVMState<'vm_life>)*/>,
    pub inner: RwLock<IRVMStateInner>,
    // this should just not be per thread
    // stack: &'static LocalKey<RefCell<OwnedIRStack>>,
    // current_frame_pointer: &'static LocalKey<RefCell<Option<*const c_void>>>,
    // current_stack_pointer: &'static LocalKey<RefCell<Option<*const c_void>>>,
}

impl<'vm_life, ExtraData: 'vm_life> IRVMState<'vm_life, ExtraData> {
    pub fn lookup_opaque_ir_method_id(&self, opaque_id: u64) -> IRMethodID {
        let mut guard = self.inner.write().unwrap();
        match guard.opaque_method_to_or_method_id.get(&opaque_id) {
            None => {
                guard.ir_method_id_max.0 += 1;
                let new_ir_method_id = guard.ir_method_id_max;
                guard.opaque_method_to_or_method_id.insert(opaque_id, new_ir_method_id);
                guard.frame_sizes_by_ir_method_id.insert(new_ir_method_id, OPAQUE_FRAME_SIZE);
                drop(guard);
                return self.lookup_opaque_ir_method_id(opaque_id);
            }
            Some(ir_method_id) => {
                *ir_method_id
            }
        }
    }

    pub fn lookup_ir_method_id_pointer(&self, ir_method_id: IRMethodID) -> *const c_void {
        let guard = self.inner.read().unwrap();
        let current_implementation = &guard.current_implementation;
        let ir_method_implementation = *current_implementation.get_by_left(&ir_method_id).unwrap();
        drop(guard);
        self.native_vm.lookup_method_addresses(ir_method_implementation).start
    }

    pub fn get_top_level_return_ir_method_id(&self) -> IRMethodID {
        self.inner.read().unwrap().top_level_return_function_id.unwrap()
    }

    pub fn init_top_level_exit_id(&self, ir_method_id: IRMethodID) {
        let mut guard = self.inner.write().unwrap();
        assert!(guard.top_level_return_function_id.is_none());
        guard.top_level_return_function_id = Some(ir_method_id);
    }

    pub fn lookup_location_of_ir_instruct(&self, ir_method_id: IRMethodID, ir_instruct_index: IRInstructIndex) -> NativeInstructionLocation {
        let read_guard = self.inner.read().unwrap();
        let method_ir_offsets_for_this_method = read_guard.method_ir_offsets.get(&ir_method_id).unwrap();
        let offset = *method_ir_offsets_for_this_method.get_by_right(&ir_instruct_index).unwrap();
        let func_start = self.lookup_ir_method_id_pointer(ir_method_id);
        unsafe { NativeInstructionLocation(func_start.offset(offset.0 as isize)) }
    }

    pub fn new(ir_exit_handler: Box<dyn Fn(&IRVMExitEvent, IRStackMut, &'vm_life Self, &mut ExtraData) -> VMExitAction<u64> + 'vm_life>) -> Self {
        let exit_handler: Box<dyn (Fn(&VMExitEvent, &mut OwnedNativeStack, &mut (ExtraData, &'vm_life IRVMState<'vm_life,ExtraData>)) -> VMExitAction<u64>) + 'vm_life> = box move |vm_exit_event: &VMExitEvent, owned_native_stack: &mut OwnedNativeStack, (extra_data, self_): &mut (ExtraData, &'vm_life Self)| {
            Self::vm_handler_to_ir_handler(ir_exit_handler.deref(), &vm_exit_event, owned_native_stack, extra_data, self_)
        };
        let native_vm = VMState::new(exit_handler);
        Self {
            native_vm,
            inner: RwLock::new(IRVMStateInner::new()),
        }
    }

    fn vm_handler_to_ir_handler(ir_exit_handler: &dyn Fn(&IRVMExitEvent, IRStackMut, &'vm_life IRVMState<'vm_life, ExtraData>, &mut ExtraData) -> VMExitAction<u64>, vm_exit_event: &VMExitEvent, owned_native_stack: &mut OwnedNativeStack, extra_data: &mut ExtraData, self_: &'vm_life IRVMState<'vm_life, ExtraData>) -> VMExitAction<u64> {
        let ir_method_id = *self_.inner.read().unwrap().current_implementation.get_by_right(&vm_exit_event.method).unwrap();
        let exit_input = RuntimeVMExitInput::from_register_state(&vm_exit_event.saved_guest_registers);
        let exiting_frame_position_rbp = vm_exit_event.saved_guest_registers.saved_registers_without_ip.rbp;
        let exiting_stack_pointer = vm_exit_event.saved_guest_registers.saved_registers_without_ip.rsp;
        let event = IRVMExitEvent {
            inner: vm_exit_event,
            ir_method: ir_method_id,
            exit_type: exit_input,
            exiting_frame_position_rbp,
        };
        let mut ir_stack = IRStack { native: owned_native_stack };
        let ir_stack_mut = IRStackMut::new(&mut ir_stack, exiting_frame_position_rbp);
        ir_exit_handler(&event, ir_stack_mut, self_, extra_data)
    }

    pub fn run_method(&'vm_life self, method_id: IRMethodID, ir_stack: &mut IRStack, extra_data: ExtraData, frame_pointer: *const c_void, stack_pointer: *const c_void) -> u64 {
        //int_state: &mut InterpreterStateGuard<'vm_life, 'l>
        // let extra_data = (int_state.thread.clone(), JavaStackPosition::Frame { frame_pointer }, int_state.jvm);
        let inner_read_guard = self.inner.read().unwrap();
        let current_implementation = *inner_read_guard.current_implementation.get_by_left(&method_id).unwrap();
        //todo for now we launch with zeroed registers, in future we may need to map values to stack or something

        unsafe { ir_stack.native.validate_frame_pointer(frame_pointer); }
        let mut initial_registers = SavedRegistersWithoutIP::new_with_all_zero();
        initial_registers.rbp = frame_pointer as *mut c_void;
        initial_registers.rsp = stack_pointer as *mut c_void;
        drop(inner_read_guard);
        let res = self.native_vm.launch_vm(ir_stack.native,current_implementation, initial_registers, (extra_data, self));
        res
    }

    fn debug_print_instructions(assembler: &CodeAssembler, offsets: &Vec<IRInstructNativeOffset>, base_address: BaseAddress) {
        let mut formatted_instructions = String::new();
        let mut formatter = IntelFormatter::default();
        for (i, instruction) in assembler.instructions().iter().enumerate() {
            unsafe { formatted_instructions.push_str(format!("{:?}:", base_address.0.offset(offsets[i].0 as isize)).as_ref()) }
            formatter.format(instruction, &mut formatted_instructions);
            formatted_instructions.push('\n');
        }
        // eprintln!("{}", formatted_instructions);
    }

    pub fn add_function(&'vm_life self, instructions: Vec<IRInstr>, frame_size: usize, handler: Box<dyn Fn(&IRVMExitEvent, IRStackMut, &Self, &mut ExtraData) -> VMExitAction<u64> + 'vm_life>) -> (IRMethodID, HashMap<RestartPointID, IRInstructIndex>) {
        let mut inner_guard = self.inner.write().unwrap();
        let current_ir_id = inner_guard.ir_method_id_max;
        inner_guard.ir_method_id_max.0 += 1;
        let (code_assembler, assembly_index_to_ir_instruct_index, restart_points) = add_function_from_ir(instructions);
        let base_address = self.native_vm.get_new_base_address();
        let block = InstructionBlock::new(code_assembler.instructions(), base_address.0 as u64);
        let result = BlockEncoder::encode(code_assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
        let new_instruction_offsets = result.new_instruction_offsets.into_iter().map(|new_instruction_offset| IRInstructNativeOffset(new_instruction_offset as usize)).collect_vec();
        Self::debug_print_instructions(&code_assembler, &new_instruction_offsets, base_address);
        inner_guard.add_function_ir_offsets(current_ir_id, new_instruction_offsets, assembly_index_to_ir_instruct_index);
        inner_guard.frame_sizes_by_ir_method_id.insert(current_ir_id, frame_size);
        let code = result.code_buffer;
        let method_implementation_id = self.native_vm.add_method_implementation(code, base_address, box move |vm_exit_event, owned_native_stack,(extra_data, ir_vm_state)| {
            Self::vm_handler_to_ir_handler(handler.deref(), &vm_exit_event, owned_native_stack,extra_data, ir_vm_state)
        });
        inner_guard.current_implementation.insert(current_ir_id, method_implementation_id);
        (current_ir_id, restart_points)
    }
}

fn vm_exit_handler<'vm_life, 'l, ExtraData>(ir_vm_state: &'vm_life IRVMState<'vm_life, ExtraData>, vm_exit_event: &VMExitEvent, extra_data: &mut ExtraData/*int_state: &mut InterpreterStateGuard<'vm_life, 'l>*/, ir_exit_handler: &(dyn Fn(&IRVMExitEvent, &mut ExtraData) -> VMExitAction<u64> + 'vm_life)) -> VMExitAction<u64> {
    let implementation_id = vm_exit_event.method;
    let exit_address = vm_exit_event.saved_guest_registers.rip;
    let exit_method_base_address = vm_exit_event.method_base_address;
    let offset = unsafe { exit_address.offset_from(exit_method_base_address) };
    if offset < 0 {
        panic!()
    }
    assert!(offset < 1024 * 1024);// methods over a megabyte prob aren't a thing

    // let offset = IRInstructNativeOffset(offset as usize);
    let inner_read_guard = ir_vm_state.inner.read().unwrap();
    let ir_method_id = *inner_read_guard.current_implementation.get_by_right(&implementation_id).unwrap();

    drop(inner_read_guard);
    // let method_offsets = inner_read_guard.method_ir_offsets.get(&ir_method_id).unwrap();
    // dbg!(method_offsets);
    // dbg!(offset);
    // let ir_instruct_index = method_offsets.get_by_left(&offset).unwrap();

    let exit_type = RuntimeVMExitInput::from_register_state(&vm_exit_event.saved_guest_registers);

    let ir_vm_exit_event = IRVMExitEvent {
        inner: &vm_exit_event,
        ir_method: ir_method_id,
        exit_type,
        exiting_frame_position_rbp: vm_exit_event.saved_guest_registers.saved_registers_without_ip.rbp,
    };

    ir_exit_handler(&ir_vm_exit_event, extra_data)
}


fn add_function_from_ir(instructions: Vec<IRInstr>) -> (CodeAssembler, Vec<(IRInstructIndex, AssemblyInstructionIndex)>, HashMap<RestartPointID, IRInstructIndex>) {
    let mut assembler = CodeAssembler::new(64).unwrap();
    let mut ir_instruct_index_to_assembly_instruction_index = Vec::new();
    let mut labels = HashMap::new();
    let mut restart_points = HashMap::new();
    for (i, instruction) in instructions.into_iter().enumerate() {
        let assembly_instruction_index_start = AssemblyInstructionIndex(assembler.instructions().len());
        let ir_instruction_index = IRInstructIndex(i);
        single_ir_to_native(&mut assembler, instruction, &mut labels, &mut restart_points, ir_instruction_index);
        let assembly_instruction_index_end = AssemblyInstructionIndex(assembler.instructions().len());
        assert!(!(assembly_instruction_index_start..assembly_instruction_index_end).is_empty());
        for assembly_index in assembly_instruction_index_start..assembly_instruction_index_end {
            ir_instruct_index_to_assembly_instruction_index.push((ir_instruction_index, assembly_index));
        }
    }
    (assembler, ir_instruct_index_to_assembly_instruction_index, restart_points)
}

fn single_ir_to_native(assembler: &mut CodeAssembler, instruction: IRInstr, labels: &mut HashMap<LabelName, CodeLabel>,
                       restart_points: &mut HashMap<RestartPointID, IRInstructIndex>, ir_instr_index: IRInstructIndex) {
    match instruction {
        IRInstr::LoadFPRelative { from, to } => {
            //stack grows down
            assembler.mov(to.to_native_64(), rbp - from.0).unwrap();
        }
        IRInstr::StoreFPRelative { from, to } => {
            assembler.mov(qword_ptr(rbp - to.0), from.to_native_64()).unwrap();
        }
        IRInstr::Load { .. } => todo!(),
        IRInstr::Store { from, to_address } => {
            assembler.mov(qword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap()
        }
        IRInstr::CopyRegister { .. } => todo!(),
        IRInstr::Add { a, res } => {
            assembler.add(res.to_native_64(), a.to_native_64()).unwrap()
        }
        IRInstr::Sub { .. } => todo!(),
        IRInstr::Div { .. } => todo!(),
        IRInstr::Mod { .. } => todo!(),
        IRInstr::Mul { .. } => todo!(),
        IRInstr::BinaryBitAnd { .. } => todo!(),
        IRInstr::ForwardBitScan { .. } => todo!(),
        IRInstr::Const32bit { .. } => todo!(),
        IRInstr::Const64bit { const_, to } => {
            assembler.mov(to.to_native_64(), const_).unwrap();
        }
        IRInstr::BranchToLabel { label } => {
            let code_label = labels.entry(label).or_insert_with(|| assembler.create_label());
            assembler.jmp(code_label.clone()).unwrap();
        }
        IRInstr::LoadLabel { .. } => todo!(),
        IRInstr::LoadRBP { .. } => todo!(),
        IRInstr::WriteRBP { .. } => todo!(),
        IRInstr::BranchEqual { .. } => todo!(),
        IRInstr::BranchNotEqual { a, b, label, } => {
            let code_label = labels.entry(label).or_insert_with(|| assembler.create_label());
            assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap();
            assembler.jne(code_label.clone()).unwrap();
        }
        IRInstr::Return { return_val, temp_register_1, temp_register_2, temp_register_3, temp_register_4, frame_size } => {
            if let Some(return_register) = return_val {
                assert_ne!(temp_register_1.to_native_64(), rax);
                assert_ne!(temp_register_2.to_native_64(), rax);
                assert_ne!(temp_register_3.to_native_64(), rax);
                assert_ne!(temp_register_4.to_native_64(), rax);
                assembler.mov(rax, return_register.to_native_64()).unwrap();
            }
            //rsp is now equal is to prev rbp qword, so that we can pop the previous rip in ret
            assembler.mov(temp_register_1.to_native_64(), rsp).unwrap();
            assembler.sub(temp_register_1.to_native_64(), rbp).unwrap();
            assembler.mov(temp_register_2.to_native_64(), frame_size as u64).unwrap();
            assembler.cmp(temp_register_1.to_native_64(), temp_register_2.to_native_64()).unwrap();
            let mut skip_assert = assembler.create_label();
            assembler.jne(skip_assert).unwrap();

            assembler.int3().unwrap();
            assembler.mov(temp_register_2.to_native_64(), 0u64).unwrap();
            assembler.mov(temp_register_2.to_native_64(), qword_ptr(temp_register_2.to_native_64())).unwrap();

            assembler.set_label(&mut skip_assert).unwrap();
            assembler.mov(rsp, rbp).unwrap();
            //load prev fram pointer
            assembler.mov(rbp, rbp - FRAME_HEADER_PREV_RBP_OFFSET).unwrap();
            assembler.ret().unwrap();
            // todo!("{:?}",frame_size)
        }
        IRInstr::VMExit2 { exit_type } => {
            gen_vm_exit(assembler, exit_type);
        }
        IRInstr::GrowStack { .. } => todo!(),
        IRInstr::LoadSP { .. } => todo!(),
        IRInstr::WithAssembler { .. } => todo!(),
        IRInstr::FNOP => todo!(),
        IRInstr::Label(label) => {
            let label_name = label.name;
            let code_label = labels.entry(label_name).or_insert_with(|| assembler.create_label());
            assembler.set_label(code_label).unwrap();
            assembler.nop().unwrap();
        }
        IRInstr::IRNewFrame {
            current_frame_size: _,
            temp_register: _,
            return_to_rip: _
        } => {
            todo!()
        }
        // IRInstr::VMExit { .. } => panic!("legacy"),
        IRInstr::IRCall { current_frame_size, new_frame_size, temp_register_1, temp_register_2, target_address } => {
            let return_to_rip = temp_register_2.to_native_64();
            let temp_register = temp_register_1.to_native_64();
            let mut after_call_label = assembler.create_label();
            assembler.lea(return_to_rip, qword_ptr(after_call_label.clone())).unwrap();
            assembler.mov(temp_register, MAGIC_1_EXPECTED).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_1_OFFSET) as u64, temp_register).unwrap();
            assembler.mov(temp_register, MAGIC_2_EXPECTED).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_MAGIC_2_OFFSET) as u64, temp_register).unwrap();

            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RBP_OFFSET) as u64, rbp).unwrap();
            assembler.mov(rbp - (current_frame_size + FRAME_HEADER_PREV_RIP_OFFSET) as u64, return_to_rip).unwrap();
            assembler.mov(temp_register, target_address as u64).unwrap();
            assembler.sub(rsp, new_frame_size as i32).unwrap();
            assembler.jmp(temp_register).unwrap();
            assembler.set_label(&mut after_call_label).unwrap();
        }
        IRInstr::NPECheck { temp_register, npe_exit_type, possibly_null } => {
            let mut after_exit_label = assembler.create_label();
            assembler.xor(temp_register.to_native_64(), temp_register.to_native_64()).unwrap();
            assembler.cmp(temp_register.to_native_64(), possibly_null.to_native_64()).unwrap();
            assembler.jne(after_exit_label).unwrap();
            gen_vm_exit(assembler, npe_exit_type);
            assembler.nop_1(rax).unwrap();
            assembler.set_label(&mut after_exit_label).unwrap();
        }
        IRInstr::RestartPoint(restart_point_id) => {
            assembler.nop_1(rbx).unwrap();
            restart_points.insert(restart_point_id, ir_instr_index);
        }
    }
}

fn gen_vm_exit(assembler: &mut CodeAssembler, exit_type: IRVMExitType) {
    let mut before_exit_label = assembler.create_label();
    let mut after_exit_label = assembler.create_label();
    let registers = vec![Register(1), Register(2), Register(3), Register(4), Register(5)];
    exit_type.gen_assembly(assembler, &mut after_exit_label, registers.clone());
    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers.into_iter().collect());
}


//index is an index, offset is a byte offset from method start

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct IRInstructIndex(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct IRInstructNativeOffset(usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct AssemblyInstructionIndex(usize);

impl std::iter::Step for AssemblyInstructionIndex {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Some(end.0 - start.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(AssemblyInstructionIndex(start.0 + count))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(AssemblyInstructionIndex(start.0 - count))
    }
}


pub struct IRVMExitEvent<'l> {
    pub inner: &'l VMExitEvent,
    pub ir_method: IRMethodID,
    pub exit_type: RuntimeVMExitInput,
    exiting_frame_position_rbp: *mut c_void,
}

