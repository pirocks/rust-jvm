#![feature(step_trait)]
#![feature(box_syntax)]
#![feature(once_cell)]
#![feature(trait_alias)]
#![feature(bound_as_ref)]

use std::cell::OnceCell;
use std::collections::{Bound, BTreeMap, HashMap, HashSet};
use std::ffi::c_void;
use std::iter::Step;
use std::ops::{Deref, Range, RangeBounds};
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

use iced_x86::{BlockEncoder, BlockEncoderOptions, ConstantOffsets, InstructionBlock};
use iced_x86::code_asm::CodeAssembler;
use itertools::Itertools;

use another_jit_vm::{BaseAddress, IRMethodID, MethodImplementationID, NativeInstructionLocation, VMExitEvent, VMState};
use another_jit_vm::code_modification::{AssemblerFunctionCallTarget, AssemblerRuntimeModificationTarget, CodeModificationHandle, FunctionCallTarget};
use another_jit_vm::saved_registers_utils::{SavedRegistersWithIPDiff, SavedRegistersWithoutIP, SavedRegistersWithoutIPDiff};
use compiler::{IRInstr, LabelName, RestartPointID};
use gc_memory_layout_common::layout::FRAME_HEADER_END_OFFSET;
use ir_stack::OPAQUE_FRAME_SIZE;
use rust_jvm_common::MethodId;
use rust_jvm_common::opaque_id_table::OpaqueID;

use crate::compiler::{BitwiseLogicType, FloatCompareMode, IRCallTarget, Signed, Size};
use crate::ir_stack::{IRFrameMut, IRStackMut};
use crate::ir_to_native::single_ir_to_native;
use crate::vm_exit_abi::IRVMExitType;
use crate::vm_exit_abi::register_structs::InvokeVirtualResolve;
use crate::vm_exit_abi::runtime_input::RuntimeVMExitInput;

#[cfg(test)]
pub mod tests;
pub mod compiler;
pub mod vm_exit_abi;
pub mod ir_stack;
pub mod ir_to_native;


#[derive(Clone, Copy, Debug)]
pub struct WasException;

pub struct IRVMStateInner<'vm, ExtraData: 'vm> {
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
    pub handler: OnceCell<ExitHandlerType<'vm, ExtraData>>,

    reserved_ir_method_id: HashSet<IRMethodID>,
}

impl<'vm, ExtraData: 'vm> IRVMStateInner<'vm, ExtraData> {
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
            handler: Default::default(),
            reserved_ir_method_id: Default::default(),
        }
    }

    pub fn add_function_ir_offsets(&mut self, current_ir_id: IRMethodID,
                                   new_instruction_offsets: &Vec<IRInstructNativeOffset>,
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
            let overwritten = offsets_range.insert(*instruction_offset, ir_instruction_index);
            assert!(overwritten.is_none());
            offsets_at_index.entry(ir_instruction_index).or_insert(*instruction_offset);
        }
        let indexes = offsets_range.iter().map(|(_, instruct)| *instruct).collect::<HashSet<_>>();
        assert_eq!(indexes.iter().max().unwrap().0 + 1, indexes.len());
        self.method_ir_offsets_range.insert(current_ir_id, offsets_range);
        self.method_ir_offsets_at_index.insert(current_ir_id, offsets_at_index);
    }
}

pub struct IRVMState<'vm, ExtraData: 'vm> {
    native_vm: VMState<'vm, u64, ExtraData>,
    pub inner: RwLock<IRVMStateInner<'vm, ExtraData>>,
}

//todo make this not an arc for perf
pub type ExitHandlerType<'vm, ExtraData> = Arc<dyn for<'r, 's, 't0, 't1> Fn(&'r IRVMExitEvent<'s>, IRStackMut<'t0>, &'t1 IRVMState<'vm, ExtraData>, &mut ExtraData) -> IRVMExitAction + 'vm>;


#[derive(Debug)]
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
    Exception {
        throwable: NonNull<c_void>//todo gc handle this
    },
}

impl<'vm, ExtraData: 'vm> IRVMState<'vm, ExtraData> {
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
        let ir_instruct_index = *native_offsets_to_index.range(Bound::Unbounded..Bound::Included(native_offset)).last().unwrap().1;
        (ir_method_id, ir_instruct_index)
    }

    pub fn lookup_ir_method_id_pointer(&self, ir_method_id: IRMethodID) -> NonNull<c_void> {
        let guard = self.inner.read().unwrap();
        let current_implementation = &guard.current_implementation;
        let ir_method_implementation = *current_implementation.get(&ir_method_id).unwrap();
        drop(guard);
        NonNull::new(self.native_vm.lookup_method_addresses(ir_method_implementation).start as *mut c_void).unwrap()
    }

    pub fn get_top_level_return_ir_method_id(&self) -> IRMethodID {
        self.inner.read().unwrap().top_level_return_function_id.unwrap()
    }

    pub fn get_top_level_return_ir_pointer(&self) -> NonNull<c_void> {
        let top_level_ir_method_id = self.inner.read().unwrap().top_level_return_function_id.unwrap();
        self.lookup_ir_method_id_pointer(top_level_ir_method_id)
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
        unsafe { NativeInstructionLocation(func_start.as_ptr().offset(offset.0 as isize)) }
    }

    pub fn new() -> Self {
        let native_vm = VMState::new();
        Self {
            native_vm,
            inner: RwLock::new(IRVMStateInner::new()),
        }
    }

    //todo should take a frame or some shit b/c needs to run on a frame for nested invocation to work
    pub fn run_method<'g, 'l, 'f>(&'g self, ir_method_id: IRMethodID, ir_stack_frame: &mut IRFrameMut<'l>, extra_data: &'f mut ExtraData) -> Result<u64, NonNull<c_void>> {
        let inner_read_guard = self.inner.read().unwrap();
        let current_implementation = *inner_read_guard.current_implementation.get(&ir_method_id).unwrap();
        //todo for now we launch with zeroed registers, in future we may need to map values to stack or something

        unsafe { ir_stack_frame.ir_stack.native.validate_frame_pointer(ir_stack_frame.ptr); }
        assert_eq!(ir_stack_frame.downgrade().ir_method_id().unwrap(), ir_method_id);
        let mut initial_registers = SavedRegistersWithoutIP::new_with_all_zero();
        initial_registers.rbp = ir_stack_frame.ptr;
        initial_registers.rsp = unsafe { ir_stack_frame.ptr.sub(ir_stack_frame.downgrade().frame_size(self)) };
        assert!(initial_registers.rbp > initial_registers.rsp);
        drop(inner_read_guard);
        let ir_stack = &mut ir_stack_frame.ir_stack;
        let mut launched_vm = self.native_vm.launch_vm(&ir_stack.native, current_implementation, initial_registers, extra_data);
        while let Some(vm_exit_event) = launched_vm.next() {
            let exit_input = RuntimeVMExitInput::from_register_state(&vm_exit_event.saved_guest_registers);
            let exiting_frame_position_rbp = vm_exit_event.saved_guest_registers.saved_registers_without_ip.rbp;
            let exiting_stack_pointer = vm_exit_event.saved_guest_registers.saved_registers_without_ip.rsp;
            let event = IRVMExitEvent {
                inner: &vm_exit_event,
                exit_type: exit_input,
            };
            let ir_stack_mut = IRStackMut::new(ir_stack, exiting_frame_position_rbp as *mut c_void, exiting_stack_pointer as *mut c_void);
            let read_guard = self.inner.read().unwrap();

            let handler = read_guard.handler.get().unwrap().clone();
            drop(read_guard);
            match (handler.deref())(&event, ir_stack_mut, self, launched_vm.extra) {
                IRVMExitAction::ExitVMCompletely { return_data: return_value } => {
                    let mut vm_exit_event = vm_exit_event;
                    vm_exit_event.indicate_okay_to_drop();
                    return Ok(return_value);
                }
                IRVMExitAction::RestartAtIndex { index: _ } => {
                    todo!()
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
                IRVMExitAction::Exception { throwable } => {
                    let mut vm_exit_event = vm_exit_event;
                    vm_exit_event.indicate_okay_to_drop();
                    return Err(throwable);
                }
            }
        }
        panic!("should be unreachable")
    }

    #[allow(unused_variables)]
    fn debug_print_instructions(assembler: &CodeAssembler, offsets: &Vec<IRInstructNativeOffset>, base_address: BaseAddress, ir_index_to_assembly_index: &Vec<(IRInstructIndex, AssemblyInstructionIndex)>, ir: &Vec<IRInstr>) {
        // use iced_x86::IntelFormatter;
        // use iced_x86::Formatter;
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

    pub fn reserve_method_id(&self) -> IRMethodID {
        let mut inner_guard = self.inner.write().unwrap();
        let current_ir_id = inner_guard.ir_method_id_max;
        inner_guard.ir_method_id_max.0 += 1;
        inner_guard.reserved_ir_method_id.insert(current_ir_id);
        current_ir_id
    }

    pub fn add_function(&'vm self, instructions: Vec<IRInstr>, frame_size: usize, ir_method_id: IRMethodID, code_modification_handle: CodeModificationHandle) -> (IRMethodID, HashMap<RestartPointID, IRInstructIndex>, HashMap<MethodId, Vec<FunctionCallTarget>>) {
        assert!(frame_size >= FRAME_HEADER_END_OFFSET);
        let mut inner_guard = self.inner.write().unwrap();
        let (code_assembler, assembly_index_to_ir_instruct_index, restart_points, call_modification_points) = add_function_from_ir(&instructions);
        let base_address = self.native_vm.get_new_base_address();
        let block = InstructionBlock::new(code_assembler.instructions(), base_address.0 as u64);
        let result = BlockEncoder::encode(64, block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS | BlockEncoderOptions::RETURN_CONSTANT_OFFSETS/*| BlockEncoderOptions::DONT_FIX_BRANCHES*/).unwrap();//issue here is probably that labels aren't being defined but are being jumped to.
        let new_instruction_offsets = result.new_instruction_offsets.into_iter().map(|new_instruction_offset| IRInstructNativeOffset(new_instruction_offset as usize)).collect_vec();
        Self::debug_print_instructions(&code_assembler, &new_instruction_offsets, base_address, &assembly_index_to_ir_instruct_index, &instructions);
        inner_guard.add_function_ir_offsets(ir_method_id, &new_instruction_offsets, assembly_index_to_ir_instruct_index);
        inner_guard.frame_sizes_by_ir_method_id.insert(ir_method_id, frame_size);
        let code = result.code_buffer;
        let method_implementation_id = self.native_vm.add_method_implementation(code, base_address, code_modification_handle);
        inner_guard.current_implementation.insert(ir_method_id, method_implementation_id);
        inner_guard.implementation_id_to_ir_method_id.insert(method_implementation_id, ir_method_id);
        let was_present = inner_guard.reserved_ir_method_id.remove(&ir_method_id);
        assert!(was_present);
        let mut function_call_targets: HashMap<usize, Vec<FunctionCallTarget>> = HashMap::new();
        for call_modification_point in call_modification_points {
            match call_modification_point {
                AssemblerFunctionCallTarget { method_id, modification_target } => {
                    unsafe {
                        match modification_target {
                            AssemblerRuntimeModificationTarget::MovQ { instruction_number } => {
                                let constant_offset: &ConstantOffsets = &result.constant_offsets[instruction_number];
                                assert!(constant_offset.has_immediate());
                                let raw_ptr = base_address.0.offset(new_instruction_offsets[instruction_number].0 as isize)
                                    .offset(constant_offset.immediate_offset() as isize);
                                function_call_targets.entry(method_id).or_default().push(FunctionCallTarget(raw_ptr as *mut *const c_void));
                            }
                        }
                    }
                }
            }
        }

        (ir_method_id, restart_points, function_call_targets)
    }
}


fn add_function_from_ir(instructions: &Vec<IRInstr>) -> (CodeAssembler, Vec<(IRInstructIndex, AssemblyInstructionIndex)>, HashMap<RestartPointID, IRInstructIndex>, Vec<AssemblerFunctionCallTarget>) {
    let mut assembler = CodeAssembler::new(64).unwrap();
    let mut ir_instruct_index_to_assembly_instruction_index = Vec::new();
    let mut labels = HashMap::new();
    let mut restart_points = HashMap::new();
    let mut assembler_function_call_modification_points = vec![];
    for (i, instruction) in instructions.iter().enumerate() {
        let assembly_instruction_index_start = AssemblyInstructionIndex(assembler.instructions().len());
        let ir_instruction_index = IRInstructIndex(i);
        let assembler_function_call_modification_point = single_ir_to_native(&mut assembler, instruction, &mut labels, &mut restart_points, ir_instruction_index);
        assembler_function_call_modification_points.extend(assembler_function_call_modification_point.into_iter());
        let assembly_instruction_index_end = AssemblyInstructionIndex(assembler.instructions().len());
        assert!(!(assembly_instruction_index_start..assembly_instruction_index_end).is_empty());
        for assembly_index in assembly_instruction_index_start..assembly_instruction_index_end {
            ir_instruct_index_to_assembly_instruction_index.push((ir_instruction_index, assembly_index));
        }
    }
    (assembler, ir_instruct_index_to_assembly_instruction_index, restart_points, assembler_function_call_modification_points)
}

fn gen_vm_exit(assembler: &mut CodeAssembler, exit_type: &IRVMExitType) {
    let mut before_exit_label = assembler.create_label();
    let mut after_exit_label = assembler.create_label();
    let registers = exit_type.registers_to_save();
    exit_type.gen_assembly(assembler, &mut after_exit_label, &registers);
    VMState::<u64, ()>::gen_vm_exit(assembler, &mut before_exit_label, &mut after_exit_label, registers);
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

impl Step for AssemblyInstructionIndex {
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
    pub exit_type: RuntimeVMExitInput,
}

