#![feature(step_trait)]
#![feature(box_syntax)]
#![feature(once_cell)]
#![feature(trait_alias)]
#![feature(bound_as_ref)]

use std::collections::{Bound, BTreeMap, HashMap, HashSet};
use std::collections::Bound::{Included, Unbounded};
use std::ffi::c_void;
use std::iter::Step;
use std::ops::{Deref, Range, RangeBounds};
use std::sync::{Arc, RwLock};

use iced_x86::{BlockEncoder, BlockEncoderOptions, code_asm, InstructionBlock};
use iced_x86::code_asm::{al, ax, byte_ptr, CodeAssembler, CodeLabel, dl, dword_ptr, dx, eax, edx, qword_ptr, rax, rbp, rbx, rcx, rdx, rsp, word_ptr};
use itertools::Itertools;

use another_jit_vm::{BaseAddress, MethodImplementationID, NativeInstructionLocation, Register, VMExitEvent, VMState};
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIP, SavedRegistersWithoutIPDiff};
use compiler::{IRInstr, LabelName, RestartPointID};
use gc_memory_layout_common::{MAGIC_1_EXPECTED, MAGIC_2_EXPECTED};
use ir_stack::{FRAME_HEADER_PREV_MAGIC_1_OFFSET, FRAME_HEADER_PREV_MAGIC_2_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET, FRAME_HEADER_PREV_RIP_OFFSET, OPAQUE_FRAME_SIZE};
use rust_jvm_common::opaque_id_table::OpaqueID;

use crate::compiler::{BitwiseLogicType, FloatCompareMode, IRCallTarget, Signed, Size};
use crate::ir_stack::{FRAME_HEADER_END_OFFSET, FRAME_HEADER_IR_METHOD_ID_OFFSET, FRAME_HEADER_METHOD_ID_OFFSET, IRFrameMut, IRStackMut};
use crate::vm_exit_abi::{IRVMExitType, RuntimeVMExitInput};

#[cfg(test)]
pub mod tests;
pub mod compiler;
pub mod vm_exit_abi;
pub mod ir_stack;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct IRMethodID(pub usize);

pub struct IRVMStateInner<'vm_life, ExtraData: 'vm_life> {
    // each IR function is distinct single java methods may many ir methods
    ir_method_id_max: IRMethodID,
    top_level_return_function_id: Option<IRMethodID>,
    current_implementation: HashMap<IRMethodID, MethodImplementationID>,
    implementation_id_to_ir_method_id: HashMap<MethodImplementationID, IRMethodID>,

    frame_sizes_by_ir_method_id: HashMap<IRMethodID, usize>,
    method_ir_offsets_range: HashMap<IRMethodID, BTreeMap<IRInstructNativeOffset, IRInstructIndex>>,
    method_ir_offsets_at_index: HashMap<IRMethodID, HashMap<IRInstructIndex, IRInstructNativeOffset>>,
    _method_ir: HashMap<IRMethodID, Vec<IRInstr>>,
    // index
    opaque_method_to_or_method_id: HashMap<OpaqueID, IRMethodID>,
    // function_ir_mapping: HashMap<IRMethodID, !>,
    handlers: HashMap<IRMethodID, ExitHandlerType<'vm_life, ExtraData>>,
}

impl<'vm_life, ExtraData: 'vm_life> IRVMStateInner<'vm_life, ExtraData> {
    pub fn new() -> Self {
        Self {
            ir_method_id_max: IRMethodID(0),
            top_level_return_function_id: None,
            current_implementation: Default::default(),
            implementation_id_to_ir_method_id: Default::default(),
            frame_sizes_by_ir_method_id: Default::default(),
            method_ir_offsets_range: Default::default(),
            method_ir_offsets_at_index: Default::default(),
            _method_ir: Default::default(),
            opaque_method_to_or_method_id: Default::default(),
            handlers: Default::default(),
        }
    }

    pub fn add_function_ir_offsets(&mut self, current_ir_id: IRMethodID,
                                   new_instruction_offsets: Vec<IRInstructNativeOffset>,
                                   ir_instruct_index_to_assembly_index: Vec<(IRInstructIndex, AssemblyInstructionIndex)>) {
        let mut offsets_range = BTreeMap::new();
        let mut offsets_at_index = HashMap::new();
        for ((i, instruction_offset), (ir_instruction_index, assembly_instruction_index_2)) in new_instruction_offsets.into_iter().enumerate().zip(ir_instruct_index_to_assembly_index.into_iter()) {
            if instruction_offset.0 as u32 == u32::MAX {
                //hack to work around iced generating annoying jumps
                continue;
            }
            let assembly_instruction_index_1 = AssemblyInstructionIndex(i);
            assert_eq!(assembly_instruction_index_1, assembly_instruction_index_2);
            let overwritten = offsets_range.insert(instruction_offset, ir_instruction_index);
            assert!(overwritten.is_none());
            offsets_at_index.entry(ir_instruction_index).or_insert(instruction_offset);
        }
        let indexes = offsets_range.iter().map(|(_, instruct)| *instruct).collect::<HashSet<_>>();
        assert_eq!(indexes.iter().max().unwrap().0 + 1, indexes.len());
        self.method_ir_offsets_range.insert(current_ir_id, offsets_range);
        self.method_ir_offsets_at_index.insert(current_ir_id, offsets_at_index);
    }
}

pub struct IRVMState<'vm_life, ExtraData: 'vm_life> {
    native_vm: VMState<'vm_life, u64, ExtraData>,
    pub inner: RwLock<IRVMStateInner<'vm_life, ExtraData>>,
}


pub type ExitHandlerType<'vm_life, ExtraData> = Arc<dyn for<'r, 's, 't0, 't1> Fn(&'r IRVMExitEvent<'s>, IRStackMut<'t0>, &'t1 IRVMState<'vm_life, ExtraData>, &mut ExtraData) -> IRVMExitAction + 'vm_life>;


pub enum IRVMExitAction {
    ExitVMCompletely {
        return_data: u64
    },
    RestartAtIndex {
        index: IRInstructIndex
    },
    RestartAtPtr {
        //todo this leaks pointer abstractions to java section. fix
        ptr: *const c_void
    },
    RestartAtIRestartPoint {
        restart_point: RestartPointID
    },
    RestartWithRegisterState {
        //todo major abstraction leak
        diff: SavedRegistersWithIPDiff
    },
}

impl<'vm_life, ExtraData: 'vm_life> IRVMState<'vm_life, ExtraData> {
    pub fn lookup_opaque_ir_method_id(&self, opaque_id: OpaqueID) -> IRMethodID {
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

    pub fn lookup_ip(&self, ip: *const c_void) -> (IRMethodID, IRInstructIndex) {
        let implementation_id = self.native_vm.lookup_ip(ip);
        let method_start = self.native_vm.lookup_method_addresses(implementation_id).start;
        let native_offset = IRInstructNativeOffset(unsafe { method_start.offset_from(ip).abs() } as usize);
        let guard = self.inner.read().unwrap();
        let ir_method_id = *guard.implementation_id_to_ir_method_id.get(&implementation_id).unwrap();
        let native_offsets_to_index = guard.method_ir_offsets_range.get(&ir_method_id).unwrap();
        let ir_instruct_index = *native_offsets_to_index.range(Unbounded..Included(native_offset)).last().unwrap().1;
        (ir_method_id, ir_instruct_index)
    }

    pub fn lookup_ir_method_id_pointer(&self, ir_method_id: IRMethodID) -> *const c_void {
        let guard = self.inner.read().unwrap();
        let current_implementation = &guard.current_implementation;
        let ir_method_implementation = *current_implementation.get(&ir_method_id).unwrap();
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
        let method_ir_offsets_for_this_method = read_guard.method_ir_offsets_at_index.get(&ir_method_id).unwrap();
        let offset = *method_ir_offsets_for_this_method.get(&ir_instruct_index).unwrap();
        let func_start = self.lookup_ir_method_id_pointer(ir_method_id);
        unsafe { NativeInstructionLocation(func_start.offset(offset.0 as isize)) }
    }

    pub fn new() -> Self {
        let native_vm = VMState::new();
        Self {
            native_vm,
            inner: RwLock::new(IRVMStateInner::new()),
        }
    }

    //todo should take a frame or some shit b/c needs to run on a frame for nested invocation to work
    pub fn run_method<'g, 'l, 'f>(&'g self, method_id: IRMethodID, ir_stack_frame: &mut IRFrameMut<'l>, extra_data: &'f mut ExtraData) -> u64 {
        let inner_read_guard = self.inner.read().unwrap();
        let current_implementation = *inner_read_guard.current_implementation.get(&method_id).unwrap();
        //todo for now we launch with zeroed registers, in future we may need to map values to stack or something

        unsafe { ir_stack_frame.ir_stack.native.validate_frame_pointer(ir_stack_frame.ptr); }
        assert_eq!(ir_stack_frame.downgrade().ir_method_id().unwrap(), method_id);
        let mut initial_registers = SavedRegistersWithoutIP::new_with_all_zero();
        initial_registers.rbp = ir_stack_frame.ptr;
        initial_registers.rsp = unsafe { ir_stack_frame.ptr.sub(ir_stack_frame.downgrade().frame_size(self)) };
        assert!(initial_registers.rbp > initial_registers.rsp);
        drop(inner_read_guard);
        let ir_stack = &mut ir_stack_frame.ir_stack;
        let mut launched_vm = self.native_vm.launch_vm(&ir_stack.native, current_implementation, initial_registers, extra_data);
        while let Some(vm_exit_event) = launched_vm.next() {
            let ir_method_id = *self.inner.read().unwrap().implementation_id_to_ir_method_id.get(&vm_exit_event.method).unwrap();
            let implementation_id = *self.inner.read().unwrap().current_implementation.get(&method_id).unwrap();
            let current_method_start = self.native_vm.lookup_method_addresses(implementation_id).start;
            let exit_input = RuntimeVMExitInput::from_register_state(&vm_exit_event.saved_guest_registers);
            let exiting_frame_position_rbp = vm_exit_event.saved_guest_registers.saved_registers_without_ip.rbp;
            let exiting_stack_pointer = vm_exit_event.saved_guest_registers.saved_registers_without_ip.rsp;
            if ir_method_id == self.get_top_level_return_ir_method_id() {
                assert!(exiting_frame_position_rbp >= exiting_stack_pointer);
            } else {
                assert!(exiting_frame_position_rbp > exiting_stack_pointer);
            }
            let function_start = self.lookup_ir_method_id_pointer(ir_method_id);
            let rip = vm_exit_event.saved_guest_registers.rip;
            let ir_instruct_native_offset = unsafe { IRInstructNativeOffset(rip.offset_from(function_start).abs() as usize) };
            let read_guard = self.inner.read().unwrap();
            let method_native_offsets_to_index = read_guard.method_ir_offsets_range.get(&ir_method_id).unwrap();
            let (_, ir_instr_index) = method_native_offsets_to_index.range((Bound::Included(ir_instruct_native_offset), Unbounded)).next().unwrap();
            let ir_instr_index = *ir_instr_index;
            drop(read_guard);
            let event = IRVMExitEvent {
                inner: &vm_exit_event,
                ir_method: ir_method_id,
                exit_type: exit_input,
                _exiting_frame_position_rbp: exiting_frame_position_rbp,
                exit_ir_instr: ir_instr_index,
            };
            // let mmaped_top = ir_stack.native.mmaped_top;
            let ir_stack_mut = IRStackMut::new(ir_stack, exiting_frame_position_rbp as *mut c_void, exiting_stack_pointer as *mut c_void);
            let read_guard = self.inner.read().unwrap();

            let handler = read_guard.handlers.get(&ir_method_id).unwrap().clone();
            // ir_stack_mut.debug_print_stack_strace(self);
            drop(read_guard);
            match (handler.deref())(&event, ir_stack_mut, self, launched_vm.extra) {
                IRVMExitAction::ExitVMCompletely { return_data: return_value } => {
                    let mut vm_exit_event = vm_exit_event;
                    vm_exit_event.indicate_okay_to_drop();
                    return return_value;
                }
                IRVMExitAction::RestartAtIndex { index } => {
                    let read_guard = self.inner.read().unwrap();
                    let address_offsets = read_guard.method_ir_offsets_at_index.get(&ir_method_id).unwrap();
                    let address_offset = address_offsets.get(&index).unwrap();
                    let target_return_rip = unsafe { current_method_start.offset(address_offset.0 as isize) };
                    launched_vm.return_to(vm_exit_event, SavedRegistersWithIPDiff { rip: Some(target_return_rip), saved_registers_without_ip: SavedRegistersWithoutIPDiff::no_change() });
                }
                IRVMExitAction::RestartAtPtr { ptr } => {
                    launched_vm.return_to(vm_exit_event, SavedRegistersWithIPDiff { rip: Some(ptr), saved_registers_without_ip: SavedRegistersWithoutIPDiff::no_change() })
                }
                IRVMExitAction::RestartAtIRestartPoint { restart_point: _ } => {
                    todo!()
                }
                IRVMExitAction::RestartWithRegisterState { diff } => {
                    launched_vm.return_to(vm_exit_event, diff)
                }
            }
        }
        panic!("should be unreachable")
    }

    #[allow(unused_variables)]
    fn debug_print_instructions(assembler: &CodeAssembler, offsets: &Vec<IRInstructNativeOffset>, base_address: BaseAddress, ir_index_to_assembly_index: &Vec<(IRInstructIndex, AssemblyInstructionIndex)>, ir: &Vec<IRInstr>) {
        // let mut formatted_instructions = String::new();
        // let mut formatter = IntelFormatter::default();
        // let mut assembly_index_to_ir_index = HashMap::new();
        // for (ir_instruct_index, assembly_instruct_index) in ir_index_to_assembly_index.iter() {
        //     assembly_index_to_ir_index.insert(*assembly_instruct_index, *ir_instruct_index);
        // }
        // for (i, instruction) in assembler.instructions().iter().enumerate() {
        //     let mut temp = String::new();
        //     formatter.format(instruction, &mut temp);
        //     let instruction_info_as_string = &match assembly_index_to_ir_index.get(&AssemblyInstructionIndex(i)) {
        //         Some(ir_instruct_index) => {
        //             ir[ir_instruct_index.0].debug_string()
        //         }
        //         None => "".to_string(),
        //     };
        //     unsafe { formatted_instructions.push_str(format!("{:?}: {:<35}{}\n", base_address.0.offset(offsets[i].0 as isize), temp, instruction_info_as_string).as_str()); }
        // }
        // eprintln!("{}", formatted_instructions);
    }

    pub fn add_function(&'vm_life self, instructions: Vec<IRInstr>, frame_size: usize, handler: ExitHandlerType<'vm_life, ExtraData>) -> (IRMethodID, HashMap<RestartPointID, IRInstructIndex>) {
        assert!(frame_size >= FRAME_HEADER_END_OFFSET);
        let mut inner_guard = self.inner.write().unwrap();
        let current_ir_id = inner_guard.ir_method_id_max;
        inner_guard.ir_method_id_max.0 += 1;
        inner_guard.handlers.insert(current_ir_id, handler);
        let (code_assembler, assembly_index_to_ir_instruct_index, restart_points) = add_function_from_ir(&instructions);
        let base_address = self.native_vm.get_new_base_address();
        let block = InstructionBlock::new(code_assembler.instructions(), base_address.0 as u64);
        let result = BlockEncoder::encode(64, block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS /*| BlockEncoderOptions::DONT_FIX_BRANCHES*/).unwrap();//issue here is probably that labels aren't being defined but are being jumped to.
        let new_instruction_offsets = result.new_instruction_offsets.into_iter().map(|new_instruction_offset| IRInstructNativeOffset(new_instruction_offset as usize)).collect_vec();
        Self::debug_print_instructions(&code_assembler, &new_instruction_offsets, base_address, &assembly_index_to_ir_instruct_index, &instructions);
        inner_guard.add_function_ir_offsets(current_ir_id, new_instruction_offsets, assembly_index_to_ir_instruct_index);
        inner_guard.frame_sizes_by_ir_method_id.insert(current_ir_id, frame_size);
        let code = result.code_buffer;
        let method_implementation_id = self.native_vm.add_method_implementation(code, base_address);
        inner_guard.current_implementation.insert(current_ir_id, method_implementation_id);
        inner_guard.implementation_id_to_ir_method_id.insert(method_implementation_id, current_ir_id);
        (current_ir_id, restart_points)
    }
}


fn add_function_from_ir(instructions: &Vec<IRInstr>) -> (CodeAssembler, Vec<(IRInstructIndex, AssemblyInstructionIndex)>, HashMap<RestartPointID, IRInstructIndex>) {
    let mut assembler = CodeAssembler::new(64).unwrap();
    let mut ir_instruct_index_to_assembly_instruction_index = Vec::new();
    let mut labels = HashMap::new();
    let mut restart_points = HashMap::new();
    for (i, instruction) in instructions.iter().enumerate() {
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

fn single_ir_to_native(assembler: &mut CodeAssembler, instruction: &IRInstr, labels: &mut HashMap<LabelName, CodeLabel>,
                       restart_points: &mut HashMap<RestartPointID, IRInstructIndex>, ir_instr_index: IRInstructIndex) {
    match instruction {
        IRInstr::LoadFPRelative { from, to, size } => {
            assembler.sub(to.to_native_64(), to.to_native_64()).unwrap();
            match size {
                Size::Byte => {
                    assembler.mov(to.to_native_8(), rbp - from.0).unwrap();
                }
                Size::X86Word => assembler.mov(to.to_native_16(), rbp - from.0).unwrap(),
                Size::X86DWord => assembler.mov(to.to_native_32(), rbp - from.0).unwrap(),
                Size::X86QWord => assembler.mov(to.to_native_64(), rbp - from.0).unwrap()
            }
        }
        IRInstr::StoreFPRelative { from, to, size } => {
            match size {
                Size::Byte => {
                    assembler.mov(byte_ptr(rbp - to.0), from.to_native_8()).unwrap()
                }
                Size::X86Word => assembler.mov(rbp - to.0, from.to_native_16()).unwrap(),
                Size::X86DWord => assembler.mov(rbp - to.0, from.to_native_32()).unwrap(),
                Size::X86QWord => assembler.mov(rbp - to.0, from.to_native_64()).unwrap(),
            }
        }
        IRInstr::Load { to, from_address, size } => {
            assembler.sub(to.to_native_64(), to.to_native_64()).unwrap();
            match size {
                Size::Byte => {
                    assembler.mov(to.to_native_8(), from_address.to_native_64() + 0i32).unwrap();
                }
                Size::X86Word => assembler.mov(to.to_native_16(), from_address.to_native_64() + 0i32).unwrap(),
                Size::X86DWord => assembler.mov(to.to_native_32(), from_address.to_native_64() + 0i32).unwrap(),
                Size::X86QWord => assembler.mov(to.to_native_64(), from_address.to_native_64() + 0i32).unwrap(),
            }
        }
        IRInstr::Store { from, to_address, size } => {
            //todo in future will need to make size actually respected here and not zx
            match size {
                Size::Byte => assembler.mov(byte_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
                Size::X86Word => assembler.mov(word_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
                Size::X86DWord => assembler.mov(dword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
                Size::X86QWord => assembler.mov(qword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap(),
            }
        }
        IRInstr::CopyRegister { .. } => todo!(),
        IRInstr::Add { res, a, size } => {
            match size {
                Size::Byte => assembler.add(res.to_native_8(), a.to_native_8()).unwrap(),
                Size::X86Word => assembler.add(res.to_native_16(), a.to_native_16()).unwrap(),
                Size::X86DWord => assembler.add(res.to_native_32(), a.to_native_32()).unwrap(),
                Size::X86QWord => assembler.add(res.to_native_64(), a.to_native_64()).unwrap(),
            }
        }
        IRInstr::Sub { res, to_subtract, size } => {
            match size {
                Size::Byte => assembler.sub(res.to_native_8(), to_subtract.to_native_8()).unwrap(),
                Size::X86Word => assembler.sub(res.to_native_16(), to_subtract.to_native_16()).unwrap(),
                Size::X86DWord => assembler.sub(res.to_native_32(), to_subtract.to_native_32()).unwrap(),
                Size::X86QWord => assembler.sub(res.to_native_64(), to_subtract.to_native_64()).unwrap(),
            }
        }
        IRInstr::Div { res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed } => {
            div_rem_common(assembler, res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed);
            match size {
                Size::Byte => assembler.mov(res.to_native_8(), al).unwrap(),
                Size::X86Word => assembler.mov(res.to_native_16(), ax).unwrap(),
                Size::X86DWord => assembler.mov(res.to_native_32(), eax).unwrap(),
                Size::X86QWord => assembler.mov(res.to_native_64(), rax).unwrap(),
            }
        }
        IRInstr::Mod { res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed } => {
            div_rem_common(assembler, res, divisor, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed);
            match size {
                Size::Byte => assembler.mov(res.to_native_8(), dl).unwrap(),
                Size::X86Word => assembler.mov(res.to_native_16(), dx).unwrap(),
                Size::X86DWord => assembler.mov(res.to_native_32(), edx).unwrap(),
                Size::X86QWord => assembler.mov(res.to_native_64(), rdx).unwrap(),
            }
        }
        IRInstr::Mul { res, a, must_be_rax, must_be_rbx, must_be_rcx, must_be_rdx, size, signed } => {
            assert_eq!(must_be_rax.0, 0);
            assert_eq!(must_be_rdx.to_native_64(), rdx);
            assert_eq!(must_be_rbx.to_native_64(), rbx);
            assert_eq!(must_be_rcx.to_native_64(), rcx);
            match size {
                Size::Byte => assembler.mov(al, res.to_native_8()).unwrap(),
                Size::X86Word => assembler.mov(ax, res.to_native_16()).unwrap(),
                Size::X86DWord => assembler.mov(eax, res.to_native_32()).unwrap(),
                Size::X86QWord => assembler.mov(rax, res.to_native_64()).unwrap(),
            }
            assembler.mov(rbx, 0u64).unwrap();
            assembler.mov(rcx, 0u64).unwrap();
            assembler.mov(rdx, 0u64).unwrap();
            match signed {
                Signed::Signed => {
                    match size {
                        Size::Byte => assembler.imul(a.to_native_8()).unwrap(),
                        Size::X86Word => assembler.imul(a.to_native_16()).unwrap(),
                        Size::X86DWord => assembler.imul(a.to_native_32()).unwrap(),
                        Size::X86QWord => assembler.imul(a.to_native_64()).unwrap(),
                    }
                }
                Signed::Unsigned => {
                    match size {
                        Size::Byte => assembler.mul(a.to_native_8()).unwrap(),
                        Size::X86Word => assembler.mul(a.to_native_16()).unwrap(),
                        Size::X86DWord => assembler.mul(a.to_native_32()).unwrap(),
                        Size::X86QWord => assembler.mul(a.to_native_64()).unwrap(),
                    }
                }
            }
            match size {
                Size::Byte => assembler.mov(res.to_native_8(), al).unwrap(),
                Size::X86Word => assembler.mov(res.to_native_16(), ax).unwrap(),
                Size::X86DWord => assembler.mov(res.to_native_32(), eax).unwrap(),
                Size::X86QWord => assembler.mov(res.to_native_64(), rax).unwrap(),
            }
        }
        IRInstr::BinaryBitAnd { res, a, size } => {
            match size {
                Size::Byte => assembler.and(res.to_native_8(), a.to_native_8()).unwrap(),
                Size::X86Word => assembler.and(res.to_native_16(), a.to_native_16()).unwrap(),
                Size::X86DWord => assembler.and(res.to_native_32(), a.to_native_32()).unwrap(),
                Size::X86QWord => assembler.and(res.to_native_64(), a.to_native_64()).unwrap(),
            }
        }
        IRInstr::BinaryBitXor { res, a, size } => {
            match size {
                Size::Byte => assembler.xor(res.to_native_8(), a.to_native_8()).unwrap(),
                Size::X86Word => assembler.xor(res.to_native_16(), a.to_native_16()).unwrap(),
                Size::X86DWord => assembler.xor(res.to_native_32(), a.to_native_32()).unwrap(),
                Size::X86QWord => assembler.xor(res.to_native_64(), a.to_native_64()).unwrap(),
            }
        }
        IRInstr::Const32bit { const_, to } => {
            assembler.mov(to.to_native_32(), *const_).unwrap();
        }
        IRInstr::Const64bit { const_, to } => {
            assembler.mov(to.to_native_64(), *const_).unwrap();
        }
        IRInstr::BranchToLabel { label } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            assembler.jmp(*code_label).unwrap();
        }
        IRInstr::LoadLabel { .. } => todo!(),
        IRInstr::LoadRBP { .. } => todo!(),
        IRInstr::WriteRBP { .. } => todo!(),
        IRInstr::BranchEqual { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, a, b, size);
            assembler.je(*code_label).unwrap();
        }
        IRInstr::BranchNotEqual { a, b, label, size, } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, a, b, size);
            assembler.jne(*code_label).unwrap();
        }
        IRInstr::BranchAGreaterEqualB { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, a, b, size);
            assembler.jge(*code_label).unwrap();
        }
        IRInstr::BranchAGreaterB { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, a, b, size);
            assembler.jg(*code_label).unwrap();
        }
        IRInstr::BranchALessB { a, b, label, size } => {
            let code_label = labels.entry(*label).or_insert_with(|| assembler.create_label());
            sized_integer_compare(assembler, a, b, size);
            assembler.jl(*code_label).unwrap();
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
            // assembler.mov(temp_register_1.to_native_64(), rsp).unwrap();
            // assembler.sub(temp_register_1.to_native_64(), rbp).unwrap();
            // assembler.mov(temp_register_2.to_native_64(), *frame_size as u64).unwrap();
            // assembler.cmp(temp_register_1.to_native_64(), temp_register_2.to_native_64()).unwrap();
            // let mut skip_assert = assembler.create_label();
            // assembler.jne(skip_assert).unwrap();
            //
            // assembler.int3().unwrap();
            // assembler.mov(temp_register_2.to_native_64(), 0u64).unwrap();
            // assembler.mov(temp_register_2.to_native_64(), qword_ptr(temp_register_2.to_native_64())).unwrap();

            // assembler.set_label(&mut skip_assert).unwrap();
            //load prev frame pointer
            assembler.mov(temp_register_1.to_native_64(), rbp - FRAME_HEADER_PREV_RIP_OFFSET).unwrap();
            assembler.mov(rbp, rbp - FRAME_HEADER_PREV_RBP_OFFSET).unwrap();
            assembler.add(rsp, *frame_size as i32).unwrap();
            assembler.jmp(temp_register_1.to_native_64()).unwrap();
            // todo!("{:?}",frame_size)
        }
        IRInstr::VMExit2 { exit_type } => {
            gen_vm_exit(assembler, exit_type);
        }
        IRInstr::GrowStack { .. } => todo!(),
        IRInstr::LoadSP { .. } => todo!(),
        IRInstr::NOP => {
            assembler.nop().unwrap();
        }
        IRInstr::Label(label) => {
            let label_name = label.name;
            let code_label = labels.entry(label_name).or_insert_with(|| assembler.create_label());
            assembler.nop().unwrap();
            assembler.set_label(code_label).unwrap();
        }
        IRInstr::IRNewFrame {
            current_frame_size: _,
            temp_register: _,
            return_to_rip: _
        } => {
            todo!()
        }
        // IRInstr::VMExit { .. } => panic!("legacy"),
        IRInstr::IRCall {
            temp_register_1,
            temp_register_2,
            arg_from_to_offsets,
            return_value,
            target_address,
            current_frame_size
        } => {
            let temp_register = temp_register_1.to_native_64();
            let return_to_rbp = temp_register_2.to_native_64();
            let mut after_call_label = assembler.create_label();
            assembler.mov(return_to_rbp, rbp).unwrap();
            assembler.sub(rbp, *current_frame_size as i32).unwrap();
            match target_address {
                IRCallTarget::Constant { new_frame_size, .. } => {
                    assembler.sub(rsp, *new_frame_size as i32).unwrap();
                }
                IRCallTarget::Variable { new_frame_size, .. } => {
                    assembler.sub(rsp, new_frame_size.to_native_64()).unwrap();
                }
            }
            for (from, to) in arg_from_to_offsets {
                assembler.mov(temp_register, return_to_rbp - from.0).unwrap();
                assembler.mov(rbp - to.0, temp_register).unwrap();
            }

            assembler.mov(temp_register, MAGIC_1_EXPECTED).unwrap();
            assembler.mov(rbp - (FRAME_HEADER_PREV_MAGIC_1_OFFSET) as u64, temp_register).unwrap();
            assembler.mov(temp_register, MAGIC_2_EXPECTED).unwrap();
            assembler.mov(rbp - (FRAME_HEADER_PREV_MAGIC_2_OFFSET) as u64, temp_register).unwrap();
            assembler.mov(rbp - (FRAME_HEADER_PREV_RBP_OFFSET) as u64, return_to_rbp).unwrap();
            match target_address {
                IRCallTarget::Constant { method_id, .. } => {
                    assembler.mov(temp_register, *method_id as u64).unwrap();
                }
                IRCallTarget::Variable { method_id, .. } => {
                    assembler.mov(temp_register, method_id.to_native_64()).unwrap();
                }
            }
            assembler.mov(rbp - (FRAME_HEADER_METHOD_ID_OFFSET) as u64, temp_register).unwrap();
            match target_address {
                IRCallTarget::Constant { ir_method_id, .. } => {
                    assembler.mov(temp_register, ir_method_id.0 as u64).unwrap();
                }
                IRCallTarget::Variable { ir_method_id, .. } => {
                    assembler.mov(temp_register, ir_method_id.to_native_64()).unwrap();
                }
            }
            assembler.mov(rbp - (FRAME_HEADER_IR_METHOD_ID_OFFSET) as u64, temp_register).unwrap();

            let return_to_rip = temp_register_2.to_native_64();
            assembler.lea(return_to_rip, qword_ptr(after_call_label.clone())).unwrap();
            assembler.mov(rbp - (FRAME_HEADER_PREV_RIP_OFFSET) as u64, return_to_rip).unwrap();
            match target_address {
                IRCallTarget::Constant { address, .. } => {
                    assembler.mov(temp_register, *address as u64).unwrap();
                }
                IRCallTarget::Variable { address, .. } => {
                    assembler.mov(temp_register, address.to_native_64()).unwrap();
                }
            }
            assembler.jmp(temp_register).unwrap();
            assembler.set_label(&mut after_call_label).unwrap();
            if let Some(return_value) = return_value {
                assembler.mov(rbp - return_value.0, rax).unwrap();
            }
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
            restart_points.insert(*restart_point_id, ir_instr_index);
        }
        IRInstr::DebuggerBreakpoint => {
            assembler.int3().unwrap();
        }
        IRInstr::Const16bit { const_, to } => {
            assembler.mov(to.to_native_32(), *const_ as u32).unwrap()
        }
        IRInstr::ShiftLeft { res, a, cl_aka_register_2, size, signed } => {
            assert_eq!(cl_aka_register_2.to_native_8(), code_asm::cl);
            assembler.mov(code_asm::cl, a.to_native_8()).unwrap();
            match signed {
                BitwiseLogicType::Arithmetic => match size {
                    Size::Byte => assembler.sal(res.to_native_8(), code_asm::cl).unwrap(),
                    Size::X86Word => assembler.sal(res.to_native_16(), code_asm::cl).unwrap(),
                    Size::X86DWord => assembler.sal(res.to_native_32(), code_asm::cl).unwrap(),
                    Size::X86QWord => assembler.sal(res.to_native_64(), code_asm::cl).unwrap(),
                },
                BitwiseLogicType::Logical => match size {
                    Size::Byte => assembler.shl(res.to_native_8(), code_asm::cl).unwrap(),
                    Size::X86Word => assembler.shl(res.to_native_16(), code_asm::cl).unwrap(),
                    Size::X86DWord => assembler.shl(res.to_native_32(), code_asm::cl).unwrap(),
                    Size::X86QWord => assembler.shl(res.to_native_64(), code_asm::cl).unwrap(),
                },
            }
        }
        IRInstr::ShiftRight { res, a, cl_aka_register_2, size, signed } => {
            assert_eq!(cl_aka_register_2.to_native_8(), code_asm::cl);
            assembler.mov(code_asm::cl, a.to_native_8()).unwrap();
            match signed {
                BitwiseLogicType::Arithmetic => match size {
                    Size::Byte => assembler.sar(res.to_native_8(), code_asm::cl).unwrap(),
                    Size::X86Word => assembler.sar(res.to_native_16(), code_asm::cl).unwrap(),
                    Size::X86DWord => assembler.sar(res.to_native_32(), code_asm::cl).unwrap(),
                    Size::X86QWord => assembler.sar(res.to_native_64(), code_asm::cl).unwrap(),
                }
                BitwiseLogicType::Logical => match size {
                    Size::Byte => assembler.shr(res.to_native_8(), code_asm::cl).unwrap(),
                    Size::X86Word => assembler.shr(res.to_native_16(), code_asm::cl).unwrap(),
                    Size::X86DWord => assembler.shr(res.to_native_32(), code_asm::cl).unwrap(),
                    Size::X86QWord => assembler.shr(res.to_native_64(), code_asm::cl).unwrap(),
                }
            }
        }
        IRInstr::BoundsCheck { length, index, size } => {
            let mut not_out_of_bounds = assembler.create_label();
            match size {
                Size::Byte => assembler.cmp(index.to_native_8(), length.to_native_8()).unwrap(),
                Size::X86Word => assembler.cmp(index.to_native_16(), length.to_native_16()).unwrap(),
                Size::X86DWord => assembler.cmp(index.to_native_32(), length.to_native_32()).unwrap(),
                Size::X86QWord => assembler.cmp(index.to_native_64(), length.to_native_64()).unwrap(),
            }
            assembler.jl(not_out_of_bounds.clone()).unwrap();
            assembler.int3().unwrap();//todo
            assembler.set_label(&mut not_out_of_bounds).unwrap();
            assembler.nop().unwrap();
        }
        IRInstr::MulConst { res, a, size, signed } => {
            match signed {
                Signed::Signed => {
                    match size {
                        Size::Byte => todo!(),
                        Size::X86Word => todo!(),
                        Size::X86DWord => todo!(),
                        Size::X86QWord => assembler.imul_3(res.to_native_64(), res.to_native_64(), *a).unwrap(),
                    }
                }
                Signed::Unsigned => {
                    match size {
                        Size::Byte => todo!(),
                        Size::X86Word => todo!(),
                        Size::X86DWord => todo!(),
                        Size::X86QWord => todo!()/*assembler.imul_3(res.to_native_64(), res.to_native_64(), *a).unwrap()*/,
                    }
                }
            }
        }
        IRInstr::LoadFPRelativeDouble { from, to } => {
            assembler.vmovsd(to.to_xmm(), rbp - from.0).unwrap();
        }
        IRInstr::StoreFPRelativeDouble { from, to } => {
            assembler.vmovsd(rbp - to.0, from.to_xmm()).unwrap();
        }
        IRInstr::LoadFPRelativeFloat { from, to } => {
            assembler.movss(to.to_xmm(), rbp - from.0).unwrap();
        }
        IRInstr::StoreFPRelativeFloat { from, to } => {
            assembler.movss(rbp - to.0, from.to_xmm()).unwrap();
        }
        IRInstr::DoubleToIntegerConvert { from, temp, to } => {
            assembler.cvtpd2pi(temp.to_mm(), from.to_xmm()).unwrap();
            assembler.movd(to.to_native_32(), temp.to_mm()).unwrap();
        }
        IRInstr::IntegerToDoubleConvert { to, temp, from } => {
            assembler.movd(temp.to_mm(), from.to_native_32()).unwrap();
            assembler.cvtpi2pd(to.to_xmm(), temp.to_mm()).unwrap()
        }
        IRInstr::DoubleToLongConvert { from, to } => {
            assembler.cvttsd2si(to.to_native_64(), from.to_xmm()).unwrap();
            // assembler.movq(to.to_native_64(), temp.to_mm()).unwrap();
        }
        IRInstr::FloatToIntegerConvert { from, temp, to } => {
            assembler.cvtps2pi(temp.to_mm(), from.to_xmm()).unwrap();
            assembler.movd(to.to_native_32(), temp.to_mm()).unwrap();
        }
        IRInstr::IntegerToFloatConvert { to, temp, from } => {
            assembler.movd(temp.to_mm(), from.to_native_32()).unwrap();
            //todo use cvtsi2ss instead avoids the move to mmx
            assembler.cvtpi2ps(to.to_xmm(), temp.to_mm()).unwrap()
        }
        IRInstr::LongToFloatConvert { to, from } => {
            // assembler.movq(temp.to_mm(), from.to_native_64()).unwrap();
            assembler.cvtsi2ss(to.to_xmm(), from.to_native_64()).unwrap()
        }
        IRInstr::LongToDoubleConvert { to, from } => {
            // assembler.movq(temp.to_mm(), from.to_native_64()).unwrap();
            assembler.cvtsi2sd(to.to_xmm(), from.to_native_64()).unwrap()
        }
        IRInstr::FloatCompare { value1, value2, res, temp1: one, temp2: zero, temp3: m_one, compare_mode } => {
            assembler.xor(res.to_native_64(), res.to_native_64()).unwrap();
            assembler.comiss(value1.to_xmm(), value2.to_xmm()).unwrap();
            float_compare_common(assembler, res, one, zero, m_one, compare_mode);
        }
        IRInstr::DoubleCompare { value1, value2, res, temp1: one, temp2: zero, temp3: m_one, compare_mode } => {
            assembler.xor(res.to_native_64(), res.to_native_64()).unwrap();
            assembler.comisd(value1.to_xmm(), value2.to_xmm()).unwrap();
            float_compare_common(assembler, res, one, zero, m_one, compare_mode);
        }
        IRInstr::IntCompare { res, value1, value2, temp1, temp2, temp3, size } => {
            match size {
                Size::Byte => assembler.cmp(value1.to_native_8(), value2.to_native_8()).unwrap(),
                Size::X86Word => assembler.cmp(value1.to_native_16(), value2.to_native_16()).unwrap(),
                Size::X86DWord => assembler.cmp(value1.to_native_32(), value2.to_native_32()).unwrap(),
                Size::X86QWord => assembler.cmp(value1.to_native_64(), value2.to_native_64()).unwrap(),
            }
            assembler.mov(res.to_native_64(), 0u64).unwrap();
            assembler.mov(temp1.to_native_64(), 1u64).unwrap();
            assembler.mov(temp2.to_native_64(), 0u64).unwrap();
            assembler.mov(temp3.to_native_64(), -1i64).unwrap();
            assembler.cmovg(res.to_native_64(), temp1.to_native_64()).unwrap();
            assembler.cmove(res.to_native_64(), temp2.to_native_64()).unwrap();
            assembler.cmovl(res.to_native_64(), temp3.to_native_64()).unwrap();
        }
        IRInstr::MulFloat { res, a } => {
            assembler.mulps(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::DivFloat { res, divisor } => {
            assembler.divss(res.to_xmm(), divisor.to_xmm()).unwrap();
        }
        IRInstr::AddFloat { res, a } => {
            assembler.addss(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::BinaryBitOr { res, a, size } => {
            match size {
                Size::Byte => assembler.or(res.to_native_8(), a.to_native_8()).unwrap(),
                Size::X86Word => assembler.or(res.to_native_16(), a.to_native_16()).unwrap(),
                Size::X86DWord => assembler.or(res.to_native_32(), a.to_native_32()).unwrap(),
                Size::X86QWord => assembler.or(res.to_native_64(), a.to_native_64()).unwrap(),
            }
        }
        IRInstr::FloatToDoubleConvert { from, to } => {
            assembler.cvtps2pd(to.to_xmm(), from.to_xmm()).unwrap();
        }
        IRInstr::MulDouble { res, a } => {
            assembler.mulpd(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::AddDouble { res, a } => {
            assembler.addpd(res.to_xmm(), a.to_xmm()).unwrap();
        }
        IRInstr::SignExtend { from, to, from_size, to_size } => {
            match from_size {
                Size::Byte => match to_size {
                    Size::Byte => {
                        todo!()
                    }
                    Size::X86Word => assembler.movsx(to.to_native_16(), from.to_native_8()).unwrap(),
                    Size::X86DWord => assembler.movsx(to.to_native_32(), from.to_native_8()).unwrap(),
                    Size::X86QWord => assembler.movsx(to.to_native_64(), from.to_native_8()).unwrap(),
                },
                Size::X86Word => match to_size {
                    Size::Byte => {
                        todo!()
                    }
                    Size::X86Word => {
                        todo!()
                    }
                    Size::X86DWord => assembler.movsx(to.to_native_32(), from.to_native_16()).unwrap(),
                    Size::X86QWord => assembler.movsx(to.to_native_64(), from.to_native_16()).unwrap()
                },
                Size::X86DWord => match to_size {
                    Size::Byte => {
                        todo!()
                    }
                    Size::X86Word => {
                        todo!()
                    }
                    Size::X86DWord => {
                        todo!()
                    }
                    Size::X86QWord => assembler.movsxd(to.to_native_64(), from.to_native_32()).unwrap()
                },
                Size::X86QWord => {
                    todo!()
                }
            };
        }
        IRInstr::ZeroExtend { from, to, from_size, to_size } => {
            match from_size {
                Size::Byte => match to_size {
                    Size::Byte => {
                        todo!()
                    }
                    Size::X86Word => assembler.movzx(to.to_native_16(), from.to_native_8()).unwrap(),
                    Size::X86DWord => assembler.movzx(to.to_native_32(), from.to_native_8()).unwrap(),
                    Size::X86QWord => assembler.movzx(to.to_native_64(), from.to_native_8()).unwrap(),
                },
                Size::X86Word => match to_size {
                    Size::Byte => {
                        todo!()
                    }
                    Size::X86Word => {
                        todo!()
                    }
                    Size::X86DWord => assembler.movzx(to.to_native_32(), from.to_native_16()).unwrap(),
                    Size::X86QWord => assembler.movzx(to.to_native_64(), from.to_native_16()).unwrap()
                },
                Size::X86DWord => match to_size {
                    Size::Byte => {
                        todo!()
                    }
                    Size::X86Word => {
                        todo!()
                    }
                    Size::X86DWord => {
                        todo!()
                    }
                    Size::X86QWord => assembler.mov(to.to_native_32(), from.to_native_32()).unwrap()//mov zeros the upper in register
                },
                Size::X86QWord => {
                    todo!()
                }
            };
        }
    }
}

fn float_compare_common(assembler: &mut CodeAssembler, res: &Register, one: &Register, zero: &Register, m_one: &Register, compare_mode: &FloatCompareMode) {
    assembler.mov(one.to_native_64(), 1u64).unwrap();
    assembler.mov(zero.to_native_64(), 0u64).unwrap();
    assembler.mov(m_one.to_native_64(), -1i64).unwrap();
    assembler.cmovnc(res.to_native_64(), one.to_native_64()).unwrap();
    assembler.cmovc(res.to_native_64(), m_one.to_native_64()).unwrap();
    assembler.cmovz(res.to_native_64(), zero.to_native_64()).unwrap();
    let saved = zero;
    assembler.mov(saved.to_native_64(), res.to_native_64()).unwrap();
    match compare_mode {
        FloatCompareMode::G => {
            assembler.cmovp(res.to_native_64(), one.to_native_64()).unwrap();
        }
        FloatCompareMode::L => {
            assembler.cmovp(res.to_native_64(), m_one.to_native_64()).unwrap();
        }
    }
    assembler.cmovnc(res.to_native_64(), saved.to_native_64()).unwrap();
    assembler.cmovnz(res.to_native_64(), saved.to_native_64()).unwrap();
    assembler.nop().unwrap();
}

fn div_rem_common(assembler: &mut CodeAssembler, res: &Register, divisor: &Register, must_be_rax: &Register, must_be_rbx: &Register, must_be_rcx: &Register, must_be_rdx: &Register, size: &Size, signed: &Signed) {
    assert_eq!(must_be_rax.0, 0);
    assert_eq!(must_be_rdx.to_native_64(), rdx);
    assert_eq!(must_be_rbx.to_native_64(), rbx);
    assert_eq!(must_be_rcx.to_native_64(), rcx);
    assembler.sub(rax, rax).unwrap();
    match size {
        Size::Byte => assembler.mov(al, res.to_native_8()).unwrap(),
        Size::X86Word => assembler.mov(ax, res.to_native_16()).unwrap(),
        Size::X86DWord => assembler.mov(eax, res.to_native_32()).unwrap(),
        Size::X86QWord => assembler.mov(rax, res.to_native_64()).unwrap(),
    }
    assembler.mov(rbx, 0u64).unwrap();
    assembler.mov(rcx, 0u64).unwrap();
    assembler.mov(rdx, 0u64).unwrap();
    match signed {
        Signed::Signed => {
            match size {
                Size::Byte => {
                    // assembler.idiv(divisor.to_native_8()).unwrap()
                    todo!()
                }
                Size::X86Word => {
                    // assembler.idiv(divisor.to_native_16()).unwrap()
                    todo!()
                }
                Size::X86DWord => {
                    assembler.cdq().unwrap();
                    assembler.idiv(divisor.to_native_32()).unwrap()
                }
                Size::X86QWord => {
                    assembler.cqo().unwrap();
                    assembler.idiv(divisor.to_native_64()).unwrap()
                }
            }
        }
        Signed::Unsigned => {
            match size {
                Size::Byte => assembler.div(divisor.to_native_8()).unwrap(),
                Size::X86Word => assembler.div(divisor.to_native_16()).unwrap(),
                Size::X86DWord => assembler.div(divisor.to_native_32()).unwrap(),
                Size::X86QWord => assembler.div(divisor.to_native_64()).unwrap(),
            }
        }
    }
}

fn sized_integer_compare(assembler: &mut CodeAssembler, a: &Register, b: &Register, size: &Size) {
    match size {
        Size::Byte => todo!(),
        Size::X86Word => todo!(),
        Size::X86DWord => assembler.cmp(a.to_native_32(), b.to_native_32()).unwrap(),
        Size::X86QWord => assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap(),
    }
}

fn gen_vm_exit(assembler: &mut CodeAssembler, exit_type: &IRVMExitType) {
    let mut before_exit_label = assembler.create_label();
    let mut after_exit_label = assembler.create_label();
    let registers = vec![Register(1), Register(2), Register(3), Register(4), Register(5), Register(6), Register(7), Register(8), Register(9)];
    exit_type.gen_assembly(assembler, &mut after_exit_label, registers.clone());
    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers.into_iter().collect());
}


//index is an index, offset is a byte offset from method start

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct IRInstructIndex(pub usize);

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct IRInstructNativeOffset(usize);

impl RangeBounds<IRInstructNativeOffset> for Range<Bound<IRInstructNativeOffset>> {
    fn start_bound(&self) -> Bound<&IRInstructNativeOffset> {
        self.start.as_ref()
    }

    fn end_bound(&self) -> Bound<&IRInstructNativeOffset> {
        self.end.as_ref()
    }
}

impl Step for IRInstructNativeOffset {
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Some(end.0 - start.0)
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0 + count))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0 - count))
    }
}

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
    pub exit_ir_instr: IRInstructIndex,
    _exiting_frame_position_rbp: *const c_void,
}

