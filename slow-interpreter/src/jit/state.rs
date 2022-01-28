use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::cmp::max;
use std::collections::{HashMap, HashSet};
use std::env::current_exe;
use std::error::Error;
use std::ffi::c_void;
use std::fs::read_to_string;
use std::intrinsics::copy_nonoverlapping;
use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use std::sync::{Arc, MutexGuard};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::thread::{LocalKey, Thread};

use bimap::BiHashMap;
use crossbeam::epoch::Atomic;
use iced_x86::{BlockEncoder, Formatter, InstructionBlock};
use iced_x86::BlockEncoderOptions;
use iced_x86::code_asm::{CodeAssembler, CodeLabel, dword_ptr, eax, qword_ptr, r15, rax, rbp, rsp};
use iced_x86::ConditionCode::l;
use iced_x86::IntelFormatter;
use iced_x86::OpCodeOperandKind::cl;
use itertools::{Either, Itertools};
use memoffset::offset_of;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};
use num::Integer;
use thread_priority::ThreadId;

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, LabelName};
use another_jit_vm_ir::vm_exit_abi::VMExitTypeWithArgs;
use classfile_view::view::HasAccessFlags;
use early_startup::{EXTRA_LARGE_REGION_BASE, EXTRA_LARGE_REGION_SIZE, EXTRA_LARGE_REGION_SIZE_SIZE, LARGE_REGION_BASE, LARGE_REGION_SIZE, LARGE_REGION_SIZE_SIZE, MAX_REGIONS_SIZE_SIZE, MEDIUM_REGION_BASE, MEDIUM_REGION_SIZE, MEDIUM_REGION_SIZE_SIZE, Regions, SMALL_REGION_BASE, SMALL_REGION_SIZE, SMALL_REGION_SIZE_SIZE};
use gc_memory_layout_common::{AllocatedObjectType, ArrayMemoryLayout, FrameHeader, FramePointerOffset, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, MemoryRegions, ObjectMemoryLayout, StackframeMemoryLayout};
use jvmti_jni_bindings::{jdouble, jint, jlong, jobject, jvalue};
use rust_jvm_common::{ByteCodeOffset, JavaThreadId, MethodId};
use rust_jvm_common::compressed_classfile::{CFieldDescriptor, CMethodDescriptor, CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstruction, CompressedInstructionInfo, CompressedLdcW};
use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName, FieldName, MethodName};
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::class_loading::{assert_loaded_class, check_initing_or_inited_class, check_loaded_class_force_loader};
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java::lang::class::JClass;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, NormalObject, Object, ObjectFieldsAndClass};
use crate::jit::{CompiledCodeID, IRInstructionIndex, MethodResolver, NotSupported, ToIR, ToNative, transition_stack_frame, TransitionType};
use crate::jit::state::birangemap::BiRangeMap;
use crate::jit_common::{JitCodeContext, RuntimeTypeInfo};
use crate::jit_common::java_stack::JavaStack;
use crate::jit_common::SavedRegisters;
use crate::jvm_state::JVMState;
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};

thread_local! {
pub static JITSTATE : RefCell<JITedCodeState> = RefCell::new(JITedCodeState::new());
}

//could be own crate
pub mod birangemap;

pub struct JITedCodeState {
    code: *mut c_void,
    current_max_compiled_code_id: CompiledCodeID,
    method_id_to_code: BiHashMap<usize, CompiledCodeID>,
    current_end: *mut c_void,
    // indexed by compiled id:
    function_addresses: BiRangeMap<*mut c_void, CompiledCodeID>,
    function_starts: HashMap<CompiledCodeID, LabelName>,
    opaque: HashSet<CompiledCodeID>,
    current_jit_instr: IRInstructionIndex,
    exits: HashMap<*mut c_void, VMExitTypeWithArgs>,
    labels: HashMap<LabelName, *mut c_void>,
    labeler: Labeler,
    pub top_level_exit_code: *mut c_void,
    address_to_byte_code_offset: HashMap<CompiledCodeID, BiRangeMap<*mut c_void, ByteCodeOffset>>,
    address_to_byte_code_index: HashMap<CompiledCodeID, BiRangeMap<*mut c_void, u16>>,
    address_to_byte_code_compressed_code: HashMap<CompiledCodeID, BiRangeMap<*mut c_void, CInstruction>>,
}

#[derive(Debug, Copy, Clone)]
pub struct Opaque {}

impl JITedCodeState {
    pub fn new() -> Self {
        let thread_id_numeric = thread::current().id().as_u64();
        const BASE_CODE_ADDRESS: usize = 1024 * 1024 * 1024 * 1024;
        const THREAD_CODE_ADDRESS_MULTIPLIER: usize = 1024 * 1024 * 1024 * 2;
        const MAX_CODE_SIZE: usize = 2 * 1024 * 1024 * 1024 - 1;
        let addr = BASE_CODE_ADDRESS + (thread_id_numeric.get() as usize) * THREAD_CODE_ADDRESS_MULTIPLIER;
        let res_code_address = unsafe { mmap(addr as *mut c_void, MAX_CODE_SIZE, ProtFlags::PROT_WRITE | ProtFlags::PROT_EXEC, MapFlags::MAP_ANONYMOUS | MapFlags::MAP_NORESERVE | MapFlags::MAP_PRIVATE, -1, 0).unwrap() } as *mut c_void;

        let mut res = Self {
            code: res_code_address,
            current_max_compiled_code_id: CompiledCodeID(0),
            method_id_to_code: Default::default(),
            function_addresses: BiRangeMap::new(),
            current_end: res_code_address,
            current_jit_instr: IRInstructionIndex(0),
            exits: HashMap::new(),
            labels: HashMap::new(),
            labeler: Labeler { current_label: AtomicU32::new(0) },
            top_level_exit_code: null_mut(),
            address_to_byte_code_offset: HashMap::new(),
            function_starts: HashMap::new(),
            opaque: HashSet::new(),
            address_to_byte_code_index: HashMap::new(),
            address_to_byte_code_compressed_code: HashMap::new(),
        };
        res.top_level_exit_code = res.add_top_level_exit_code();
        res
    }

    pub fn ip_to_bytecode_pc(&self, instruct_pointer: *mut c_void) -> Result<(u16, ByteCodeOffset), Opaque> {
        //todo track opaque funcitons
        let compiled_code_id = self.function_addresses.get(&instruct_pointer).unwrap();
        if self.opaque.contains(compiled_code_id) {
            return Err(Opaque {});
        }
        let address_to_bytecode_for_this_method = self.address_to_byte_code_offset.get(&compiled_code_id).unwrap();
        let address_to_bytecode_index_for_this_method = self.address_to_byte_code_index.get(&compiled_code_id).unwrap();
        let bytecode_offset = address_to_bytecode_for_this_method.get(&instruct_pointer).unwrap();
        let index_offset = address_to_bytecode_index_for_this_method.get(&instruct_pointer).unwrap();
        Ok((*index_offset, *bytecode_offset))
    }

    pub fn ip_to_bytecode_pcs(&self, instruct_pointer: *mut c_void) -> Result<Vec<ByteCodeOffset>, Opaque> {
        //todo track opaque funcitons
        let compiled_code_id = self.function_addresses.get(&instruct_pointer).unwrap();
        if self.opaque.contains(compiled_code_id) {
            return Err(Opaque {});
        }
        let address_to_bytecode_for_this_method = self.address_to_byte_code_offset.get(&compiled_code_id).unwrap();
        let address_to_bytecode_index_for_this_method = self.address_to_byte_code_index.get(&compiled_code_id).unwrap();
        let address_to_code = self.address_to_byte_code_compressed_code.get(&compiled_code_id).unwrap();
        let bytecode_offset = address_to_bytecode_for_this_method.values().cloned().collect_vec();
        Ok(bytecode_offset)
    }

    pub fn ip_to_methodid(&self) -> MethodId {
        todo!()
    }

    fn add_top_level_exit_code(&mut self) -> *mut c_void {
        let mut labels = vec![];
        let start_label = self.labeler.new_label(&mut labels);
        let exit_label = self.labeler.new_label(&mut labels);
        let nop = CompressedInstruction { offset: ByteCodeOffset(0), instruction_size: 0, info: CompressedInstructionInfo::nop };
        let ir = ToIR {
            labels,
            ir: vec![(ByteCodeOffset(0), IRInstr::Label { 0: IRLabel { name: start_label } }, nop.clone()),
                     (ByteCodeOffset(0), todo!()/*IRInstr::VMExit { before_exit_label: exit_label, after_exit_label: None, exit_type: VMExitTypeWithArgs::TopLevelReturn {} }*/, nop),
            ],
            function_start_label: start_label,
        };

        let current_code_id = self.next_code_id((-1isize) as usize);
        self.opaque.insert(current_code_id);
        self.add_from_ir("top level exit wrapper function".to_string(), current_code_id, ir)
    }

    fn next_code_id(&mut self, method_id: MethodId) -> CompiledCodeID {
        let next_code_id = CompiledCodeID(self.current_max_compiled_code_id.0 + 1);
        self.current_max_compiled_code_id = next_code_id;
        assert!(!self.method_id_to_code.contains_right(&next_code_id));
        self.method_id_to_code.insert(method_id, next_code_id);
        next_code_id
    }


    pub fn ir_to_native(&self, ir: ToIR, base_address: *mut c_void, method_log_info: String) -> ToNative {
        let ToIR { labels: ir_labels, ir, function_start_label } = ir;
        let mut exits = HashMap::new();
        let mut assembler: CodeAssembler = CodeAssembler::new(64).unwrap();
        let mut iced_labels = ir_labels.into_iter().map(|label| (label.name, assembler.create_label())).collect::<HashMap<_, _>>();
        let mut label_instruction_offsets: Vec<(LabelName, u32)> = vec![];
        let mut instruction_index_to_bytecode_offset_start: HashMap<u32, (ByteCodeOffset, CInstruction)> = HashMap::new();
        for (bytecode_offset, ir_instr, cinstruction) in ir {
            let cinstruction: CInstruction = cinstruction;
            instruction_index_to_bytecode_offset_start.insert(assembler.instructions().len() as u32, (bytecode_offset, cinstruction));
            match ir_instr {
                IRInstr::LoadFPRelative { from, to } => {
                    assembler.mov(to.to_native_64(), rbp + from.0).unwrap();
                }
                IRInstr::StoreFPRelative { from, to } => {
                    assembler.mov(qword_ptr(rbp + to.0), from.to_native_64()).unwrap();
                }
                IRInstr::Load { from_address, to } => {
                    assembler.mov(to.to_native_64(), from_address.to_native_64() + 0).unwrap();
                }
                IRInstr::Store { from, to_address } => {
                    assembler.mov(qword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap();
                }
                IRInstr::Add { res, a } => {
                    assembler.add(res.to_native_64(), a.to_native_64()).unwrap();
                }
                IRInstr::Sub { res, to_subtract } => {
                    assembler.sub(res.to_native_64(), to_subtract.to_native_64()).unwrap();
                }
                IRInstr::Div { .. } => todo!(),
                IRInstr::Mod { .. } => todo!(),
                IRInstr::Mul { .. } => todo!(),
                IRInstr::Const32bit { to, const_ } => {
                    assembler.mov(to.to_native_32(), const_).unwrap();
                }
                IRInstr::Const64bit { to, const_ } => {
                    assembler.mov(to.to_native_64(), const_).unwrap();
                }
                IRInstr::BranchToLabel { label } => {
                    let target_location = self.labels.get(&label);
                    match target_location {
                        None => {
                            assembler.jmp(iced_labels[&label]).unwrap();
                        }
                        Some(target_location) => {
                            assembler.jmp(*target_location as u64).unwrap();
                        }
                    }
                }
                IRInstr::BranchEqual { label, a, b } => {
                    assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap();
                    assembler.je(iced_labels[&label]).unwrap();
                }
                IRInstr::BranchNotEqual { a, b, label } => {
                    assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap();
                    assembler.jne(iced_labels[&label]).unwrap()
                }
                /*                IRInstr::VMExit { before_exit_label: exit_label, exit_type, .. } => {
                                    todo!();
                                    let native_stack_pointer = (offset_of!(JitCodeContext, native_saved) + offset_of!(SavedRegisters, stack_pointer)) as i64;
                                    let native_frame_pointer = (offset_of!(JitCodeContext, native_saved) + offset_of!(SavedRegisters, frame_pointer)) as i64;
                                    let native_instruction_pointer = (offset_of!(JitCodeContext, native_saved) + offset_of!(SavedRegisters, instruction_pointer)) as i64;
                                    let java_stack_pointer = (offset_of!(JitCodeContext, java_saved) + offset_of!(SavedRegisters, stack_pointer)) as i64;
                                    let java_frame_pointer = (offset_of!(JitCodeContext, java_saved) + offset_of!(SavedRegisters, frame_pointer)) as i64;
                                    let exit_handler_ip = offset_of!(JitCodeContext, exit_handler_ip) as i64;
                                    if false {
                                        //exit to exit handler
                                        // save_java_stack
                                        assembler.mov(r15 + java_stack_pointer, rsp).unwrap();
                                        // save_java_frame
                                        assembler.mov(r15 + java_frame_pointer, rbp).unwrap();
                                        // restore_old_stack
                                        assembler.mov(rsp, r15 + native_stack_pointer).unwrap();
                                        // restore_old_frame
                                        assembler.mov(rbp, r15 + native_frame_pointer).unwrap();
                                        // call back to exit_handler
                                        assembler.call(qword_ptr(r15 + exit_handler_ip)).unwrap();
                                    }
                                    //exit back to initial run_method
                                    // if false {
                                    // save_java_stack
                                    assembler.mov(r15 + java_stack_pointer, rsp).unwrap();
                                    // save_java_frame
                                    assembler.mov(r15 + java_frame_pointer, rbp).unwrap();
                                    // restore_old_stack
                                    assembler.mov(rsp, r15 + native_stack_pointer).unwrap();
                                    // restore_old_frame
                                    assembler.mov(rbp, r15 + native_frame_pointer).unwrap();
                                    // call_to_old
                                    //todo this clobbers existing data
                                    assembler.call(qword_ptr(r15 + native_instruction_pointer)).unwrap();
                                    exits.insert(exit_label, exit_type);
                                    label_instruction_offsets.push((exit_label, assembler.instructions().len() as u32));
                                    //need noop b/c can't have a label at end
                                    // match exit_type.clone(){
                                    //     VMExitType::ResolveInvokeStatic { method_name, desc, target_class } => {
                                    //
                                    //     }
                                    //     VMExitType::TopLevelReturn { .. } => {
                                    //         todo!()
                                    //     }
                                    // }
                                }
                */                IRInstr::Label(label) => {
                    let iced_label = iced_labels.get_mut(&label.name).unwrap();
                    label_instruction_offsets.push((label.name, assembler.instructions().len() as u32));
                    assembler.set_label(iced_label).unwrap();
                }
                IRInstr::Return { return_val, temp_register_1, temp_register_2, temp_register_3, temp_register_4, frame_size } => {
                    if let Some(return_register) = return_val {
                        assembler.mov(rax, return_register.to_native_64()).unwrap();
                    }
                    let index = temp_register_2.to_native_64();
                    let (div, rem) = frame_size.div_rem(&8);
                    assert_eq!(rem, 0);
                    assembler.mov(index, div as u64).unwrap();
                    let address_base = temp_register_3.to_native_64();
                    assembler.lea(address_base, rbp + size_of::<*mut c_void>() * 2).unwrap(); // skip saved rip and saved rbp
                    let m1 = temp_register_1.to_native_64();
                    let mut branch_back = assembler.create_label();
                    assembler.set_label(&mut branch_back);
                    assembler.mov(m1, -1i64 as u64).unwrap();
                    let address = temp_register_4.to_native_64();
                    assembler.lea(address, address_base + index * size_of::<*mut c_void>()).unwrap();
                    assembler.mov(qword_ptr(address), m1).unwrap();
                    assembler.add(index, -1i32).unwrap();
                    assembler.mov(m1, 0u64).unwrap();
                    assembler.cmp(index, m1).unwrap();
                    assembler.jne(branch_back).unwrap();

                    //rsp is now equal is to prev rbp + 1 qword, so that we can pop the previous rip in ret
                    assembler.mov(rsp, rbp).unwrap();
                    assert_eq!(offset_of!(FrameHeader, prev_rip), 0);
                    //load prev fram pointer
                    assembler.mov(rbp, rbp + offset_of!(FrameHeader, prev_rpb)).unwrap();
                    assembler.ret().unwrap();
                }
                IRInstr::LoadLabel { label, to } => {
                    let iced_label = iced_labels[&label];
                    assembler.lea(to.to_native_64(), dword_ptr(iced_label)).unwrap();
                }
                IRInstr::LoadRBP { to } => {
                    assembler.mov(to.to_native_64(), rbp).unwrap();
                }
                IRInstr::GrowStack { amount } => {
                    assembler.lea(rsp, rsp + amount).unwrap();
                }
                IRInstr::WriteRBP { from } => {
                    assembler.mov(rbp, from.to_native_64()).unwrap();
                }
                IRInstr::LoadSP { to } => {
                    assembler.mov(to.to_native_64(), rsp).unwrap();
                }
                IRInstr::NOP => {
                    // assembler.fnop().unwrap();
                }
                IRInstr::CopyRegister { .. } => todo!(),
                IRInstr::BinaryBitAnd { .. } => todo!(),
                IRInstr::ForwardBitScan { .. } => todo!(),
                IRInstr::IRNewFrame { .. } => todo!(),
                IRInstr::VMExit2 { .. } => todo!(),
                IRInstr::IRCall { .. } => todo!(),
                IRInstr::NPECheck { .. } => todo!(),
                IRInstr::RestartPoint(_) => todo!(),
                IRInstr::DebuggerBreakpoint => {
                    assembler.int3().unwrap();
                }
                IRInstr::Load32 { to, from_address } => {
                    assembler.mov(to.to_native_32(),qword_ptr(from_address.to_native_64())).unwrap();
                }
            }
        }
        let block = InstructionBlock::new(assembler.instructions(), base_address as u64);
        let mut formatted_instructions = String::new();
        let mut formatter = IntelFormatter::default();
        for (i, instruction) in assembler.instructions().iter().enumerate() {
            let mut temp = "".to_string();
            formatter.format(instruction, &mut temp);
            let instruction_info_as_string = &match instruction_index_to_bytecode_offset_start.get(&(i as u32)) {
                Some((_, x)) => x.info.instruction_to_string_without_meta(),
                None => "".to_string(),
            };
            formatted_instructions.push_str(format!("#{}: {:<35}{}\n", i, temp, instruction_info_as_string).as_str());
        }
        eprintln!("{}", format!("{} :\n{}", method_log_info, formatted_instructions));
        let result = BlockEncoder::encode(64, block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS | BlockEncoderOptions::DONT_FIX_BRANCHES).unwrap();
        let mut bytecode_offset_to_address = BiRangeMap::new();
        let mut new_labels: HashMap<LabelName, *mut c_void> = Default::default();
        let mut label_instruction_indexes = label_instruction_offsets.into_iter().peekable();
        let mut current_byte_code_offset = Some((0, ByteCodeOffset(0)));
        let mut current_byte_code_start_address = Some(base_address);
        for (i, native_offset) in result.new_instruction_offsets.iter().enumerate() {
            if *native_offset == u32::MAX {
                continue;
            }
            let current_instruction_address = unsafe { base_address.offset(*native_offset as isize) };
            loop {
                match label_instruction_indexes.peek() {
                    None => break,
                    Some((label, instruction_index)) => {
                        assert!(i <= *instruction_index as usize);
                        if *instruction_index as usize == i {
                            new_labels.insert(*label, current_instruction_address);
                            let _ = label_instruction_indexes.next();
                            continue;
                        }
                    }
                }
                break;
            }
            if i == 0 {
                continue;
            }
            if let Some((new_byte_code_offset, cinstr)) = instruction_index_to_bytecode_offset_start.get(&(i as u32)) {
                assert!(((current_byte_code_start_address.unwrap()) as u64) < ((current_instruction_address) as u64));
                bytecode_offset_to_address.insert_range(current_byte_code_start_address.unwrap()..current_instruction_address, (current_byte_code_offset.unwrap().0, current_byte_code_offset.unwrap().1, cinstr.clone()));
                current_byte_code_offset = Some((i as u16, *new_byte_code_offset));
                current_byte_code_start_address = Some(current_instruction_address);
            }
        }
        assert!(label_instruction_indexes.peek().is_none());
        ToNative {
            code: result.code_buffer,
            new_labels,
            bytecode_offset_to_address,
            exits,
            function_start_label,
        }
    }

    fn add_from_ir(&mut self, method_log_info: String, current_code_id: CompiledCodeID, ir: ToIR) -> *mut c_void {
        let ToNative { code, new_labels, bytecode_offset_to_address, exits, function_start_label } = self.ir_to_native(ir, self.current_end, method_log_info.clone());
        self.function_starts.insert(current_code_id, function_start_label);
        let install_at = self.current_end;
        unsafe {
            self.current_end = install_at.offset(code.len() as isize);
        }
        const TWO_GIG: isize = 2 * 1024 * 1024 * 1024;
        unsafe {
            if self.current_end.offset_from(self.code) > TWO_GIG {
                panic!()
            }
        }
        for (label_name, exit_type) in exits {
            self.exits.insert(new_labels[&label_name], exit_type);
        }
        self.labels.extend(new_labels.into_iter());
        let bytecode_offset_to_address: BiRangeMap<*mut c_void, (_, _, _)> = bytecode_offset_to_address;
        for (address_range, offset) in bytecode_offset_to_address {
            self.address_to_byte_code_index.entry(current_code_id).or_insert(BiRangeMap::new()).insert_range(address_range.clone(), offset.0);
            self.address_to_byte_code_offset.entry(current_code_id).or_insert(BiRangeMap::new()).insert_range(address_range.clone(), offset.1);
            self.address_to_byte_code_compressed_code.entry(current_code_id).or_insert(BiRangeMap::new()).insert_range(address_range, offset.2);
        }
        unsafe { copy_nonoverlapping(code.as_ptr(), install_at as *mut u8, code.len()) }
        unsafe {
            self.function_addresses.insert_range(install_at..(install_at.offset(code.len() as isize)), current_code_id);
        }
        install_at
    }

    pub fn run_method_safe(jit_state: &RefCell<JITedCodeState>, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, methodid: MethodId) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        let res = unsafe {
            let jit_state_ = jit_state.borrow();
            let code_id = *jit_state_.method_id_to_code.get_by_left(&methodid).unwrap();
            drop(jit_state_);
            JITedCodeState::run_method(jit_state, jvm, int_state, methodid, code_id)
        };
        res
    }

    fn runtime_type_info(memory_region: &MutexGuard<MemoryRegions>) -> RuntimeTypeInfo {
        unsafe {
            RuntimeTypeInfo {
                small_num_regions: 0,
                medium_num_regions: 0,
                large_num_regions: 0,
                extra_large_num_regions: 0,
                //todo can't do this b/c vecs might be realloced
                small_region_index_to_region_data: memory_region.small_region_types.as_ptr(),
                medium_region_index_to_region_data: memory_region.medium_region_types.as_ptr(),
                large_region_index_to_region_data: memory_region.large_region_types.as_ptr(),
                extra_large_region_index_to_region_data: memory_region.extra_large_region_types.as_ptr(),
                allocated_type_to_vtable: transmute(0xDDDDDDDDusize), //major todo
            }
        }
    }

    #[allow(unknown_lints)]
    #[allow(named_asm_labels)]
    #[allow(unaligned_references)]
    unsafe fn resume_method(jit_state: &RefCell<JITedCodeState>, mut target_ip: *mut c_void, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        loop {
            //todo reacrchited pushing/popping of frames storing sp.
            let java_stack: &mut JavaStack = todo!();//int_state.java_stack();
            let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: as_ptr, status_register } = java_stack.handle_vm_entry();
            let rust_stack: u64 = stack_pointer as u64;
            let rust_frame: u64 = frame_pointer as u64;
            let memory_region: MutexGuard<MemoryRegions> = jvm.gc.memory_region.lock().unwrap();
            let mut jit_code_context = JitCodeContext {
                native_saved: SavedRegisters {
                    stack_pointer: todo!(),
                    frame_pointer: todo!(),
                    instruction_pointer: todo!(),
                    status_register,
                },
                java_saved: SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: target_ip, status_register },
                exit_handler_ip: null_mut(),
                runtime_type_info: Self::runtime_type_info(&memory_region),
            };
            drop(memory_region);
            eprint!("going in sp:{:?} fp:{:?} ip: {:?}", jit_code_context.java_saved.stack_pointer, jit_code_context.java_saved.frame_pointer, jit_code_context.java_saved.instruction_pointer);
            let mut jit_state_ = jit_state.borrow();
            let compiled_code_id = jit_state_.function_addresses.get(&target_ip).unwrap();
            let method_id = jit_state_.method_id_to_code.get_by_right(compiled_code_id).unwrap();
            let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(*method_id).unwrap();
            let view = rc.view();
            let method_view = view.method_view_i(method_i);
            let string_pool = &jvm.string_pool;
            eprintln!("@ {:?}:{:?}", view.name().unwrap_object_name().0.to_str(string_pool), method_view.name().0.to_str(string_pool));
            drop(jit_state_);
            let jit_context_pointer = &jit_code_context as *const JitCodeContext as u64;
            ///pub struct FrameHeader {
            //     pub prev_rip: *mut c_void,
            //     pub prev_rpb: *mut c_void,
            //     pub frame_info_ptr: *mut FrameInfo,
            //     pub debug_ptr: *mut c_void,
            //     pub magic_part_1: u64,
            //     pub magic_part_2: u64,
            // }
            let old_java_ip: *mut c_void = todo!();
            /*asm!(
            "push rbx",
            "push rbp",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            // technically these need only be saved on windows:
            // "push xmm*",
            //todo perhaps should use pusha/popa here, b/c this must be really slow
            "push rsp",
            // store old stack pointer into context
            "mov [{0} + {old_stack_pointer_offset}],rsp",
            // store old frame pointer into context
            "mov [{0} + {old_frame_pointer_offset}],rbp",
            // store exit instruction pointer into context
            "lea r15, [rip+__rust_jvm_internal_after_call]",
            "mov [{0} + {old_rip_offset}],r15",
            "mov r15,{0}",
            // load java frame pointer
            "mov rbp, [{0} + {new_frame_pointer_offset}]",
            // load java stack pointer
            "mov rsp, [{0} + {new_stack_pointer_offset}]",
            // jump to jitted code
            "jmp [{0} + {new_rip_offset}]",
            //
            "__rust_jvm_internal_after_call:",
            // gets old java ip from call back to here in java
            "pop {1}",
            "pop rsp",
            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbp",
            "pop rbx",
            in(reg) jit_context_pointer,
            out(reg) old_java_ip,
            old_stack_pointer_offset = const 0,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,stack_pointer)),
            old_frame_pointer_offset = const 8,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,frame_pointer)),
            old_rip_offset = const 16,//(offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,instruction_pointer)),
            new_stack_pointer_offset = const 32,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,stack_pointer)),
            new_frame_pointer_offset = const 40,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,frame_pointer)),
            new_rip_offset = const 48,//(offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,instruction_pointer))
            );*/
            todo!();
            jit_code_context.java_saved.instruction_pointer = old_java_ip;
            //major todo java_stack is mutably borrowed multiple times here b/c recursive exits
            java_stack.saved_registers = Some(jit_code_context.java_saved.clone());
            //todo exception handling
            let exit_type = jit_state.borrow().exits.get(&old_java_ip).unwrap().clone();
            let (method_name_str, class_name_str) = (|| {
                let current_frame = int_state.current_frame();
                let frame_view = current_frame.frame_view(jvm);
                let methodid = frame_view.method_id().unwrap_or(usize::MAX);
                let (rc, method_i) = match jvm.method_table.read().unwrap().try_lookup(methodid) {
                    Some(x) => x,
                    None => return ("unknown".to_string(), "unknown".to_string()),
                };
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let method_name_str = method_view.name().0.to_str(&jvm.string_pool);
                let class_name_str = view.name().unwrap_name().0.to_str(&jvm.string_pool);
                (method_name_str, class_name_str)
            })();

            let java_stack: &mut JavaStack = todo!()/*int_state.java_stack()*/;
            eprintln!("going out sp:{:?} fp:{:?} ip:{:?} {} {} {:?}", java_stack.saved_registers.unwrap().stack_pointer, java_stack.saved_registers.unwrap().frame_pointer, java_stack.saved_registers.unwrap().instruction_pointer, class_name_str, method_name_str, todo!()/*exit_type*/);
            target_ip = match JITedCodeState::handle_exit(jit_state, todo!()/*exit_type*/, jvm, int_state, methodid, old_java_ip) {
                None => {
                    return Ok(None);
                }
                Some(target_ip) => target_ip,
            };
        }
    }

    #[allow(named_asm_labels)]
    pub unsafe fn run_method(jitstate: &RefCell<JITedCodeState>, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        let target_ip = jitstate.borrow().function_addresses.get_reverse(&compiled_id).unwrap().start;
        drop(jitstate.borrow_mut());
        JITedCodeState::resume_method(jitstate, target_ip, jvm, int_state, methodid, compiled_id)
    }

    #[allow(unaligned_references)]
    fn handle_exit(jitstate: &RefCell<JITedCodeState>, exit_type: VMExitTypeWithArgs, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, methodid: usize, old_java_ip: *mut c_void) -> Option<*mut c_void> {
        // int_state.debug_print_stack_trace(jvm);
        todo!()
        /*        match exit_type {
                    VMExitType::ResolveInvokeStatic { method_name, desc, target_class } => {
                        let save = int_state.java_stack().saved_registers;
                        let inited_class = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                        int_state.java_stack().saved_registers = save;
                        let method_view = inited_class.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap();
                        let to_call_function_method_id = jvm.method_table.write().unwrap().get_method_id(inited_class.clone(), method_view.method_i());
                        if method_view.is_native() {
                            match run_native_method(jvm, int_state, inited_class.clone(), method_view.method_i(), todo!()) {
                                Ok(Some(res)) => int_state.current_frame_mut().push(res),
                                Ok(None) => {}
                                Err(WasException {}) => todo!(),
                            };
                            return Some(old_java_ip);
                        } else {
                            jitstate.borrow_mut().add_function(method_view.code_attribute().unwrap(), to_call_function_method_id, MethodResolver { jvm, loader: int_state.current_loader() });
                            let (current_function_rc, current_function_method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
                            let method_view = current_function_rc.unwrap_class_class().class_view.method_view_i(current_function_method_i);
                            let code = method_view.code_attribute().unwrap();
                            Self::recompile_method_and_restart(jitstate, methodid, jvm, int_state, code, old_java_ip, TransitionType::ResolveCalls).unwrap();
                            todo!()
                        }
                    }
                    VMExitType::TopLevelReturn { .. } => {
                        int_state.set_function_return(true);
                        None
                    }
                    VMExitType::ResolveInvokeSpecial { method_name, desc, target_class } => {
                        let save = int_state.java_stack().saved_registers;
                        let inited_class = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                        int_state.java_stack().saved_registers = save;
                        let method_view = inited_class.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap();
                        let to_call_function_method_id = jvm.method_table.write().unwrap().get_method_id(inited_class.clone(), method_view.method_i());
                        jitstate.borrow_mut().add_function(method_view.code_attribute().unwrap(), to_call_function_method_id, MethodResolver { jvm, loader: int_state.current_loader() });
                        let (current_function_rc, current_function_method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
                        let method_view = current_function_rc.unwrap_class_class().class_view.method_view_i(current_function_method_i);
                        let code = method_view.code_attribute().unwrap();
                        Self::recompile_method_and_restart(jitstate, methodid, jvm, int_state, code, old_java_ip, TransitionType::ResolveCalls).unwrap();
                        todo!()
                    }
                    VMExitType::Todo { .. } => {
                        todo!()
                    }
                    VMExitType::RunNativeStatic { method_name, desc, target_class } => {
                        let method_name_str = method_name.0.to_str(&jvm.string_pool);
                        dbg!(method_name_str);
                        let args = setup_args_from_current_frame(jvm, int_state, &desc, false);
                        let save = int_state.java_stack().saved_registers;
                        let inited = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                        let method_i = inited.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap().method_i();
                        int_state.java_stack().saved_registers = save;
                        let res = run_native_method(jvm, int_state, inited, method_i, args).unwrap();
                        match res {
                            None => {}
                            Some(_) => {
                                todo!()
                            }
                        }
                        int_state.java_stack().saved_registers = save;
                        let jit_state = jitstate.borrow();
                        let code_id = jit_state.method_id_to_code.get_by_left(&methodid).unwrap();
                        let address_to_bytecode_for_this_method = jit_state.address_to_byte_code_offset.get(&code_id).unwrap();
                        let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                        let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + 3); // call static is 3 bytes
                        let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                        Some(restart_address)
                    }
                    VMExitType::PutStatic { target_type, target_class, name, frame_pointer_offset_of_to_put } => {
                        let save = int_state.java_stack().saved_registers;
                        let target_class_rc = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                        int_state.java_stack().saved_registers = save;
                        // let _target_type_rc = assert_loaded_class(jvm, target_type.clone());
                        target_class_rc.static_vars().insert(name, int_state.raw_read_at_frame_pointer_offset(frame_pointer_offset_of_to_put, target_type.to_runtime_type().unwrap()));
                        //todo dup
                        let jitstate_borrow = jitstate.borrow();
                        let code_id = jitstate_borrow.method_id_to_code.get_by_left(&methodid).unwrap();
                        let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                        let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                        let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + 3); // put static is 3 bytes
                        let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                        Some(restart_address)
                    }
                    VMExitType::AllocateVariableSizeArrayANewArray { target_type_sub_type, len_offset, res_write_offset } => {
                        // dbg!(int_state.get_java_stack().saved_registers.unwrap().frame_pointer);
                        // dbg!(int_state.get_java_stack().saved_registers.unwrap().stack_pointer);
                        let before_stack = int_state.java_stack().saved_registers.unwrap().stack_pointer;
                        let save = int_state.java_stack().saved_registers;
                        let inited_target_type_rc = check_initing_or_inited_class(jvm, int_state, target_type_sub_type).unwrap();
                        int_state.java_stack().saved_registers = save;
                        assert_eq!(before_stack, int_state.java_stack().saved_registers.unwrap().stack_pointer);
                        // dbg!(int_state.get_java_stack().saved_registers.unwrap().frame_pointer);
                        // dbg!(int_state.get_java_stack().saved_registers.unwrap().stack_pointer);
                        let array_len = int_state.raw_read_at_frame_pointer_offset(len_offset, RuntimeType::IntType).unwrap_int() as usize;
                        let allocated_object_type = runtime_class_to_allocated_object_type(&inited_target_type_rc, int_state.current_loader(), Some(array_len), jvm.thread_state.get_current_thread().java_tid);
                        let mut memory_region = jvm.gc.memory_region.lock().unwrap();
                        let mut region_data = memory_region.find_or_new_region_for(allocated_object_type);
                        let allocation = region_data.get_allocation();
                        let to_write = jvalue { l: allocation.as_ptr() as jobject };
                        int_state.raw_write_at_frame_pointer_offset(res_write_offset, to_write);
                        let jitstate_borrow = jitstate.borrow();
                        let code_id = jitstate_borrow.method_id_to_code.get_by_left(&methodid).unwrap();
                        let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                        let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                        let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + 3); // anewarray is 3 bytes
                        let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                        Some(restart_address)
                    }
                    VMExitType::InitClass { target_class } => {
                        let saved = int_state.java_stack().saved_registers;
                        let inited_target_type_rc = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                        int_state.java_stack().saved_registers = saved;
                        let (current_function_rc, current_function_method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
                        let method_view = current_function_rc.unwrap_class_class().class_view.method_view_i(current_function_method_i);
                        let code = method_view.code_attribute().unwrap();
                        let instruct_pointer = int_state.java_stack().saved_registers().instruction_pointer;
                        assert_eq!(instruct_pointer, old_java_ip);
                        let mut jit_state = jitstate.borrow_mut();
                        let compiled_code_id = jit_state.function_addresses.get(&instruct_pointer).unwrap();
                        let compiled_code = jit_state.address_to_byte_code_offset.get(&compiled_code_id).unwrap();
                        // problem here is that a function call overwrites the old old ip
                        let return_to_byte_code_offset = *compiled_code.get(&instruct_pointer).unwrap();
                        let new_base_address = jit_state.add_function(code, methodid, MethodResolver { jvm, loader: int_state.current_loader() });
                        let new_code_id = *jit_state.function_addresses.get(&new_base_address).unwrap();
                        let start_byte_code_addresses = jit_state.address_to_byte_code_offset.get(&new_code_id).unwrap().get_reverse(&return_to_byte_code_offset).unwrap().clone();
                        let restart_execution_at = start_byte_code_addresses.start;
                        Some(restart_execution_at)
                    }
                    VMExitType::NeedNewRegion { .. } => todo!(),
                    VMExitType::Allocate { ptypeview, loader, res, bytecode_size } => {
                        let save = int_state.java_stack().saved_registers;
                        let rc = check_loaded_class_force_loader(jvm, int_state, &ptypeview, loader).unwrap();
                        int_state.java_stack().saved_registers = save;
                        let allocated = match rc.deref() {
                            RuntimeClass::Array(_) => todo!(),
                            RuntimeClass::Object(obj) => JavaValue::new_object(jvm, rc).unwrap(),
                            _ => panic!(),
                        };
                        int_state.raw_write_at_frame_pointer_offset(res, jvalue { l: allocated.raw_ptr_usize() as jobject });
                        let jitstate_borrow = jitstate.borrow();
                        let code_id = jitstate_borrow.method_id_to_code.get_by_left(&methodid).unwrap();
                        let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                        let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                        let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + bytecode_size);
                        let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                        Some(restart_address)
                    }
                    VMExitType::Throw { .. } => todo!(),
                    VMExitType::LoadString { string, res } => {
                        let string = JString::from_rust(jvm, int_state, string).unwrap();
                        todo!()
                    }
                    VMExitType::LoadClass { class_type, res, bytecode_size } => {
                        let save = int_state.java_stack().saved_registers;
                        let class = JClass::from_type(jvm, int_state, class_type).unwrap();
                        int_state.java_stack().saved_registers = save;
                        dbg!(save.unwrap().frame_pointer);
                        let to_write = jvalue { l: class.object().raw_ptr_usize() as jobject };
                        unsafe { dbg!(to_write.l) };
                        int_state.raw_write_at_frame_pointer_offset(res, to_write);
                        let jitstate_borrow = jitstate.borrow();
                        let code_id = jitstate_borrow.method_id_to_code.get_by_left(&methodid).unwrap();
                        let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                        let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                        let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + bytecode_size); //size of ldc
                        let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                        Some(restart_address)
                    }
                    VMExitType::MonitorEnter { .. } => {
                        todo!()
                    }
                    VMExitType::InvokeSpecialNative { .. } => {
                        todo!()
                    }
                    VMExitType::MonitorExit { .. } => {
                        todo!()
                    }
                    VMExitType::NPE { .. } => {
                        todo!()
                    }
                    VMExitType::Trace { values } => {
                        let save = int_state.java_stack().saved_registers;
                        let frame_pointer = save.unwrap().frame_pointer;
                        unsafe {
                            for (name, value) in values {
                                let ptr = frame_pointer.offset(value.0 as isize) as *mut *const c_void;
                                println!("{} {:?}", name, ptr.read())
                            }
                        }
                        let jitstate_borrow = jitstate.borrow();
                        let code_id = jitstate_borrow.method_id_to_code.get_by_left(&methodid).unwrap();
                        let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                        let next_java_ip: *mut c_void = address_to_bytecode_for_this_method.keys().map(|key| key.start).sorted().filter(|pointer| pointer > &old_java_ip).next().unwrap();
                        let restart_bytecode_offset = address_to_bytecode_for_this_method.get(&next_java_ip).unwrap();
                        let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                        Some(restart_address)
                    }
                }
        */
    }
}

pub fn runtime_class_to_allocated_object_type(ref_type: &RuntimeClass, loader: LoaderName, arr_len: Option<usize>, thread_id: JavaThreadId) -> AllocatedObjectType {
    match ref_type {
        RuntimeClass::Byte => panic!(),
        RuntimeClass::Boolean => panic!(),
        RuntimeClass::Short => panic!(),
        RuntimeClass::Char => panic!(),
        RuntimeClass::Int => panic!(),
        RuntimeClass::Long => panic!(),
        RuntimeClass::Float => panic!(),
        RuntimeClass::Double => panic!(),
        RuntimeClass::Void => panic!(),
        RuntimeClass::Array(arr) => {
            let primitive_type = match arr.sub_class.deref() {
                RuntimeClass::Byte => CompressedParsedDescriptorType::ByteType,
                RuntimeClass::Boolean => CompressedParsedDescriptorType::BooleanType,
                RuntimeClass::Short => CompressedParsedDescriptorType::ShortType,
                RuntimeClass::Char => CompressedParsedDescriptorType::CharType,
                RuntimeClass::Int => CompressedParsedDescriptorType::IntType,
                RuntimeClass::Long => CompressedParsedDescriptorType::LongType,
                RuntimeClass::Float => CompressedParsedDescriptorType::FloatType,
                RuntimeClass::Double => CompressedParsedDescriptorType::DoubleType,
                RuntimeClass::Void => panic!(),
                RuntimeClass::Object(_) | RuntimeClass::Array(_) => {
                    return AllocatedObjectType::ObjectArray {
                        thread: thread_id,
                        sub_type: arr.sub_class.cpdtype().unwrap_ref_type().clone(),
                        len: arr_len.unwrap() as i32,
                        sub_type_loader: loader,
                    };
                }
                RuntimeClass::Top => panic!(),
            };
            AllocatedObjectType::PrimitiveArray { thread: thread_id, primitive_type, len: arr_len.unwrap() as i32 }
        }
        RuntimeClass::Object(class_class) => AllocatedObjectType::Class {
            thread: thread_id,
            name: class_class.class_view.name().unwrap_name(),
            loader,
            size: class_class.recursive_num_fields * size_of::<jlong>(),
        },
        RuntimeClass::Top => panic!(),
    }
}

pub struct Labeler {
    current_label: AtomicU32,
}

impl Labeler {
    pub fn new() -> Self {
        Self {
            current_label: AtomicU32::new(0)
        }
    }

    pub fn new_label(&self, labels_vec: &mut Vec<IRLabel>) -> LabelName {
        let current_label = self.current_label.fetch_add(1, Ordering::SeqCst);
        let res = LabelName(current_label);
        labels_vec.push(IRLabel { name: res });
        res
    }
}

#[derive(Debug)]
pub struct NaiveStackframeLayout {
    pub(crate) max_locals: u16,
    pub(crate) max_stack: u16,
    pub(crate) stack_depth: HashMap<u16, u16>,
}

impl NaiveStackframeLayout {
    pub fn from_stack_depth(stack_depth: HashMap<u16, u16>, max_locals: u16, max_stack: u16) -> Self {
        Self { max_locals, max_stack, stack_depth }
    }

    pub fn new(instructions: &Vec<&CInstruction>, max_locals: u16, max_stack: u16) -> Self {
        todo!()
        /*let mut stack_depth = HashMap::new();
        let mut current_depth = 0;
        for (i, instruct) in instructions.iter().enumerate() {
            stack_depth.insert(i as u16, current_depth);
            match &instruct.info {
                CompressedInstructionInfo::invokestatic { descriptor, .. } => {
                    current_depth -= descriptor.arg_types.len() as u16;
                    match &descriptor.return_type {
                        CompressedParsedDescriptorType::VoidType => {}
                        _ => {
                            current_depth += 1;
                        }
                    }
                }
                CompressedInstructionInfo::return_ => {}
                CompressedInstructionInfo::ireturn => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::aload_0 |
                CompressedInstructionInfo::aload_1 |
                CompressedInstructionInfo::aload_2 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                    current_depth -= 1;
                    current_depth -= descriptor.arg_types.len() as u16;
                }
                CompressedInstructionInfo::iconst_0 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::putfield { name, desc, target_class } => {
                    current_depth -= 2;
                }
                CompressedInstructionInfo::getfield { name, desc, target_class } => {}
                CompressedInstructionInfo::aconst_null => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::iconst_2 {} |
                CompressedInstructionInfo::iconst_1 {} => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::putstatic { name, desc, target_class } => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::anewarray(_) => {}
                CompressedInstructionInfo::new(_) => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::dup => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::ldc(Either::Left(_)) => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::ifnull(_) |
                CompressedInstructionInfo::ifnonnull(_) => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::athrow => {}
                CompressedInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
                    current_depth -= 1;
                    current_depth -= descriptor.arg_types.len() as u16;
                    match descriptor.return_type {
                        CompressedParsedDescriptorType::VoidType => {}
                        _ => {
                            current_depth += 1;
                        }
                    }
                }
                CompressedInstructionInfo::monitorexit |
                CompressedInstructionInfo::monitorenter => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::astore_1 |
                CompressedInstructionInfo::astore_2 |
                CompressedInstructionInfo::astore_3 |
                CompressedInstructionInfo::istore_3 |
                CompressedInstructionInfo::istore_2 => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::iload_3 |
                CompressedInstructionInfo::iload_2 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::ifeq(_) |
                CompressedInstructionInfo::ifne(_) => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::isub => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::bipush(_) => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::if_icmpeq(_) |
                CompressedInstructionInfo::if_icmpne(_) => {
                    current_depth -= 2;
                }
                CompressedInstructionInfo::goto_(_) => {}
                todo => todo!("{:?}", todo)
            }
        }
        Self {
            max_locals,
            max_stack,
            stack_depth,
        }*/
    }
}

impl StackframeMemoryLayout for NaiveStackframeLayout {
    fn local_var_entry(&self, current_count: u16, i: u16) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + i as usize * size_of::<jlong>())
    }

    fn operand_stack_entry(&self, current_count: u16, from_end: u16) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + (self.max_locals + self.stack_depth[&current_count] - from_end) as usize * size_of::<jlong>())
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + (self.max_locals as usize + self.max_stack as usize + 1) * size_of::<jlong>()
        // max stack is maximum depth which means we need 1 one more for size
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}

pub fn setup_args_from_current_frame(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, desc: &CMethodDescriptor, is_virtual: bool) -> Vec<JavaValue<'gc_life>> {
    if is_virtual {
        todo!()
    }
    let java_stack = int_state.java_stack();
    let mut args = vec![];
    for (i, _) in desc.arg_types.iter().enumerate() {
        let current_frame = int_state.current_frame();
        let operand_stack = current_frame.operand_stack(jvm);
        let types_ = operand_stack.types();
        dbg!(&types_);
        let operand_stack_i = types_.len() - 1 - i;
        let jv = operand_stack.get(operand_stack_i as u16, types_[operand_stack_i].clone());
        args.push(jv);
    }
    args
}
/*
IRInstr::WithAssembler {
                                function: box move |assembler: &mut CodeAssembler| {
                                    let _reserved = Register(0);
                                    let small_region_base = Register(1).to_native_64();
                                    let medium_region_base = Register(2).to_native_64();
                                    let large_region_base = Register(3).to_native_64();
                                    let extra_large_region_base = Register(4).to_native_64();
                                    assembler.mov(small_region_base, (SMALL_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.mov(medium_region_base, (MEDIUM_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.mov(large_region_base, (LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.mov(extra_large_region_base, (EXTRA_LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.and(small_region_base, this_pointer).unwrap();
                                    assembler.and(medium_region_base, this_pointer).unwrap();
                                    assembler.and(large_region_base, this_pointer).unwrap();
                                    assembler.and(extra_large_region_base, this_pointer).unwrap();

                                    assembler.mov(region_elemant_size_size.to_native_64(), 1u64).unwrap();

                                    let mut after_size_calc_label = assembler.create_label();
                                    let mask_for_this_pointer = Register(5).to_native_64();
                                    //todo vectorize to get rid off branches
                                    assembler.cmp(small_region_base, 0).unwrap();
                                    assembler.je(after_size_calc_label).unwrap();
                                    assembler.shl(region_elemant_size_size.to_native_64(), SMALL_REGION_SIZE_SIZE as i32).unwrap();
                                    assembler.mov(mask_for_this_pointer, (SMALL_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.xor(this_pointer, mask_for_this_pointer).unwrap();

                                    assembler.cmp(medium_region_base, 0).unwrap();
                                    assembler.je(after_size_calc_label).unwrap();
                                    assembler.shl(region_elemant_size_size.to_native_64(), MEDIUM_REGION_SIZE_SIZE as i32).unwrap();
                                    assembler.mov(mask_for_this_pointer, (MEDIUM_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.xor(this_pointer, mask_for_this_pointer).unwrap();

                                    assembler.cmp(large_region_base, 0).unwrap();
                                    assembler.je(after_size_calc_label).unwrap();
                                    assembler.shl(region_elemant_size_size.to_native_64(), LARGE_REGION_SIZE_SIZE as i32).unwrap();
                                    assembler.mov(mask_for_this_pointer, (LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.xor(this_pointer, mask_for_this_pointer).unwrap();

                                    assembler.cmp(extra_large_region_base, 0).unwrap();
                                    assembler.je(after_size_calc_label).unwrap();
                                    assembler.shl(region_elemant_size_size.to_native_64(), EXTRA_LARGE_REGION_SIZE_SIZE as i32).unwrap();
                                    assembler.mov(mask_for_this_pointer, (EXTRA_LARGE_REGION_BASE << MAX_REGIONS_SIZE_SIZE) as u64).unwrap();
                                    assembler.xor(this_pointer, mask_for_this_pointer).unwrap();


                                    assembler.set_label(&mut after_size_calc_label).unwrap();

                                    let region_index = this_pointer;
                                    assembler.shlx(region_index, this_pointer, region_elemant_size_size.to_native_64()).unwrap();
                                    //todo lookup in r15 the method_table for this variable
                                    // means pointer is not from heap address
                                }
                            };

*/