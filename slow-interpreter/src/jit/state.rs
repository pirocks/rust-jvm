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

use classfile_view::view::HasAccessFlags;
use early_startup::{EXTRA_LARGE_REGION_BASE, EXTRA_LARGE_REGION_SIZE, EXTRA_LARGE_REGION_SIZE_SIZE, LARGE_REGION_BASE, LARGE_REGION_SIZE, LARGE_REGION_SIZE_SIZE, MAX_REGIONS_SIZE_SIZE, MEDIUM_REGION_BASE, MEDIUM_REGION_SIZE, MEDIUM_REGION_SIZE_SIZE, Regions, SMALL_REGION_BASE, SMALL_REGION_SIZE, SMALL_REGION_SIZE_SIZE};
use jvmti_jni_bindings::{jdouble, jint, jlong, jobject, jvalue};
use rust_jvm_common::compressed_classfile::{CFieldDescriptor, CMethodDescriptor, CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstruction, CompressedInstructionInfo, CompressedLdcW};
use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName, FieldName, MethodName};
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::class_loading::{assert_loaded_class, check_initing_or_inited_class, check_loaded_class_force_loader};
use crate::gc_memory_layout_common::{AllocatedObjectType, ArrayMemoryLayout, FramePointerOffset, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, MemoryRegions, ObjectMemoryLayout, StackframeMemoryLayout};
use crate::gc_memory_layout_common::FrameHeader;
use crate::gc_memory_layout_common::RegionData;
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::ir_to_java_layer::vm_exit_abi::VMExitType;
use crate::java::lang::class::JClass;
use crate::java::lang::string::JString;
use crate::java_values::{JavaValue, NormalObject, Object, ObjectFieldsAndClass};
use crate::jit::{ByteCodeOffset, CompiledCodeID, IRInstructionIndex, LabelName, MethodResolver, NotSupported, ToIR, ToNative, transition_stack_frame, TransitionType};
use crate::jit::ir::{IRInstr, IRLabel, Register};
use crate::jit::state::birangemap::BiRangeMap;
use crate::jit_common::{JitCodeContext, RuntimeTypeInfo};
use crate::jit_common::java_stack::JavaStack;
use crate::jit_common::SavedRegisters;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::threading::JavaThreadId;

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
    exits: HashMap<*mut c_void, VMExitType>,
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
        dbg!(address_to_code);
        dbg!(address_to_code.get(&instruct_pointer).unwrap());
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
        let nop = CompressedInstruction { offset: 0, instruction_size: 0, info: CompressedInstructionInfo::nop };
        let ir = ToIR {
            labels,
            ir: vec![(ByteCodeOffset(0), IRInstr::Label { 0: IRLabel { name: start_label } }, nop.clone()), (ByteCodeOffset(0), IRInstr::VMExit { before_exit_label: exit_label, after_exit_label: None, exit_type: VMExitType::TopLevelReturn {} }, nop)],
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

    pub fn add_function(&mut self, code: &CompressedCode, methodid: usize, resolver: MethodResolver<'l>) -> *mut c_void {
        let current_code_id = self.next_code_id(methodid);
        let CompressedCode { instructions, max_locals, max_stack, exception_table, stack_map_table } = code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();
        let layout = resolver.lookup_method_layout(methodid);
        let ir = self.to_ir(cinstructions, &layout, resolver).unwrap();
        let (current_rc, method_i) = resolver.jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = current_rc.unwrap_class_class().class_view.clone();
        let method_view = view.method_view_i(method_i);
        let method_name = method_view.name().0.to_str(&resolver.jvm.string_pool);
        let class_name = view.name().unwrap_name().0.to_str(&resolver.jvm.string_pool);
        let res_ptr = self.add_from_ir(format!("{} {} {:?}", class_name, method_name, current_code_id), current_code_id, ir);
        res_ptr
    }

    fn to_ir<'l>(&mut self, byte_code: Vec<&CInstruction>, layout: &dyn StackframeMemoryLayout, resolver: MethodResolver<'l>) -> Result<ToIR, NotSupported> {
        let mut labels = vec![];
        let mut initial_ir = vec![];
        let function_start_label: LabelName = self.labeler.new_label(&mut labels);
        let function_end_label: LabelName = self.labeler.new_label(&mut labels);
        let mut pending_labels = vec![(ByteCodeOffset(0), function_start_label), (ByteCodeOffset(byte_code.last().unwrap().offset), function_end_label)];
        for (i, byte_code_instr) in byte_code.iter().enumerate() {
            let current_offset = ByteCodeOffset(byte_code_instr.offset);
            let current_byte_code_instr_count: u16 = i as u16;
            let next_byte_code_instr_count: u16 = (i + 1) as u16;
            match &byte_code_instr.info {
                CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => self.gen_code_invokestatic(resolver, &mut labels, &mut initial_ir, current_offset, method_name, descriptor, classname_ref_type),
                CompressedInstructionInfo::goto_(offset) => {
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    initial_ir.push((current_offset, IRInstr::BranchToLabel { label: branch_to_label }));
                }
                CompressedInstructionInfo::ifnull(offset) => {
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let possibly_null_register = Register(0);
                    initial_ir.push((
                        current_offset,
                        IRInstr::LoadFPRelative {
                            from: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                            to: possibly_null_register,
                        },
                    ));
                    let register_with_null = Register(1);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: register_with_null, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::BranchEqual { a: register_with_null, b: possibly_null_register, label: branch_to_label }))
                }
                CompressedInstructionInfo::ifnonnull(offset) => {
                    //todo dup
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let possibly_null_register = Register(0);
                    initial_ir.push((
                        current_offset,
                        IRInstr::LoadFPRelative {
                            from: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                            to: possibly_null_register,
                        },
                    ));
                    let register_with_null = Register(1);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: register_with_null, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::BranchNotEqual { a: register_with_null, b: possibly_null_register, label: branch_to_label }))
                }
                CompressedInstructionInfo::ifne(offset) => {
                    //todo dup
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let possibly_null_register = Register(0);
                    initial_ir.push((
                        current_offset,
                        IRInstr::LoadFPRelative {
                            from: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                            to: possibly_null_register,
                        },
                    ));
                    let register_with_null = Register(1);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: register_with_null, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::BranchNotEqual { a: register_with_null, b: possibly_null_register, label: branch_to_label }))
                }
                CompressedInstructionInfo::ifeq(offset) => {
                    //todo dup
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let possibly_null_register = Register(0);
                    initial_ir.push((
                        current_offset,
                        IRInstr::LoadFPRelative {
                            from: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                            to: possibly_null_register,
                        },
                    ));
                    let register_with_null = Register(1);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: register_with_null, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::BranchEqual { a: register_with_null, b: possibly_null_register, label: branch_to_label }))
                }
                CompressedInstructionInfo::if_icmpne(offset) => {
                    //todo dup
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let value1 = Register(0);
                    let value2 = Register(1);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: value2 }));
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 1), to: value1 }));
                    initial_ir.push((current_offset, IRInstr::BranchNotEqual { a: value1, b: value2, label: branch_to_label }))
                }
                CompressedInstructionInfo::if_icmpeq(offset) => {
                    //todo dup
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let value1 = Register(0);
                    let value2 = Register(1);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: value2 }));
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 1), to: value1 }));
                    initial_ir.push((current_offset, IRInstr::BranchEqual { a: value1, b: value2, label: branch_to_label }))
                }
                CompressedInstructionInfo::return_ => {
                    let frame_size = layout.full_frame_size() - 2 * size_of::<*mut c_void>();
                    initial_ir.push((
                        current_offset,
                        IRInstr::Return {
                            return_val: None,
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            temp_register_3: Register(3),
                            temp_register_4: Register(4),
                            frame_size,
                        },
                    ));
                }
                CompressedInstructionInfo::areturn => {
                    let temp = Register(0);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: temp }));
                    let frame_size = layout.full_frame_size() - 2 * size_of::<*mut c_void>();
                    initial_ir.push((
                        current_offset,
                        IRInstr::Return {
                            return_val: Some(temp),
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            temp_register_3: Register(3),
                            temp_register_4: Register(4),
                            frame_size,
                        },
                    ));
                }
                CompressedInstructionInfo::ireturn => {
                    //todo dup
                    let temp = Register(0);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: temp }));
                    let frame_size = layout.full_frame_size() - 2 * size_of::<*mut c_void>();
                    initial_ir.push((
                        current_offset,
                        IRInstr::Return {
                            return_val: Some(temp),
                            temp_register_1: Register(1),
                            temp_register_2: Register(2),
                            temp_register_3: Register(3),
                            temp_register_4: Register(4),
                            frame_size,
                        },
                    ));
                }
                CompressedInstructionInfo::iload_0 => {
                    JITedCodeState::gen_iload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 0);
                }
                CompressedInstructionInfo::iload_1 => {
                    JITedCodeState::gen_iload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 1);
                }
                CompressedInstructionInfo::iload_2 => {
                    JITedCodeState::gen_iload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 2);
                }
                CompressedInstructionInfo::iload_3 => {
                    JITedCodeState::gen_iload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 3);
                }
                CompressedInstructionInfo::aload_0 => {
                    JITedCodeState::gen_aload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 0);
                }
                CompressedInstructionInfo::aload_1 => {
                    JITedCodeState::gen_aload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 1);
                }
                CompressedInstructionInfo::aload_2 => {
                    JITedCodeState::gen_aload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 2);
                }
                CompressedInstructionInfo::aload_3 => {
                    JITedCodeState::gen_aload_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, 3);
                }
                CompressedInstructionInfo::astore_1 => {
                    JITedCodeState::gen_astore_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, 1);
                }
                CompressedInstructionInfo::astore_2 => {
                    JITedCodeState::gen_astore_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, 2);
                }
                CompressedInstructionInfo::astore_3 => {
                    JITedCodeState::gen_astore_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, 3);
                }
                CompressedInstructionInfo::istore_2 => {
                    JITedCodeState::gen_istore_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, 2);
                }
                CompressedInstructionInfo::istore_3 => {
                    JITedCodeState::gen_istore_n(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, 3);
                }
                CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                    self.gen_code_invokespecial(layout, resolver, &mut labels, &mut initial_ir, current_offset, current_byte_code_instr_count, method_name, descriptor, classname_ref_type)
                    //todo need to not constantly call same.
                }
                CompressedInstructionInfo::iconst_0 => JITedCodeState::gen_iconst_n(layout, &mut initial_ir, current_offset, next_byte_code_instr_count, 0),
                CompressedInstructionInfo::iconst_1 => JITedCodeState::gen_iconst_n(layout, &mut initial_ir, current_offset, next_byte_code_instr_count, 1),
                CompressedInstructionInfo::iconst_2 => JITedCodeState::gen_iconst_n(layout, &mut initial_ir, current_offset, next_byte_code_instr_count, 2),
                CompressedInstructionInfo::putfield { name, desc, target_class } => self.gen_code_putfield(layout, resolver, &mut labels, &mut initial_ir, current_offset, current_byte_code_instr_count, name, target_class),
                CompressedInstructionInfo::getfield { name, desc, target_class } => {
                    let cpd_type = (*target_class).into();
                    match resolver.lookup_type_loaded(&cpd_type) {
                        None => {
                            let exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::VMExit { before_exit_label: exit_label, after_exit_label: todo!(), exit_type: todo!()/*VMExitType::InitClass { target_class: cpd_type }*/ }));
                        }
                        Some((rc, _)) => {
                            let (field_number, field_type) = rc.unwrap_class_class().field_numbers.get(name).unwrap();
                            let class_ref_register = Register(0);
                            let to_get_field = Register(1);
                            let offset = Register(2);
                            let null = Register(3);
                            initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: class_ref_register }));
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: null, const_: 0 }));
                            let npe_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::BranchEqual { a: class_ref_register, b: null, label: npe_label }));
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: offset, const_: (field_number * size_of::<jlong>()) as u64 }));
                            initial_ir.push((current_offset, IRInstr::Add { res: class_ref_register, a: offset }));
                            initial_ir.push((current_offset, IRInstr::Load { from_address: class_ref_register, to: to_get_field }));
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: to_get_field, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
                            let after_npe_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::BranchToLabel { label: after_npe_label }));
                            initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: npe_label })));
                            let npe_exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::VMExit { before_exit_label: npe_exit_label, after_exit_label: todo!(), exit_type: todo!()/*VMExitType::Todo {}*/ }));
                            initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: after_npe_label })))
                        }
                    }
                }
                CompressedInstructionInfo::aconst_null => JITedCodeState::gen_code_aconst_null(layout, &mut initial_ir, current_offset, next_byte_code_instr_count),
                CompressedInstructionInfo::putstatic { name, desc, target_class } => self.gen_code_putstatic(layout, &mut labels, &mut initial_ir, current_offset, current_byte_code_instr_count, name, desc, target_class),
                CompressedInstructionInfo::anewarray(cpdtype) => self.gen_code_anewarray(layout, &mut labels, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count, cpdtype),
                CompressedInstructionInfo::new(class_name) => self.gen_code_new(layout, resolver, &mut labels, &mut initial_ir, current_offset, next_byte_code_instr_count, class_name),
                CompressedInstructionInfo::dup => {
                    JITedCodeState::gen_code_dup(layout, &mut initial_ir, current_offset, current_byte_code_instr_count, next_byte_code_instr_count);
                }
                CompressedInstructionInfo::ldc(Either::Left(left_ldc)) => self.gen_code_left_ldc(&mut labels, &mut initial_ir, layout, current_offset, next_byte_code_instr_count, left_ldc),
                CompressedInstructionInfo::athrow => {
                    self.gen_code_athrow(layout, &mut labels, &mut initial_ir, current_offset, current_byte_code_instr_count);
                }
                CompressedInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
                    match resolver.lookup_special(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
                        None => {
                            let exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((
                                current_offset,
                                IRInstr::VMExit {
                                    before_exit_label: exit_label,
                                    after_exit_label: None,
                                    exit_type: todo!()/*VMExitType::ResolveInvokeSpecial {
                                        method_name: *method_name,
                                        desc: descriptor.clone(),
                                        target_class: CPDType::Ref(classname_ref_type.clone()),
                                    }*/,
                                },
                            ));
                        }
                        Some((method_id, is_native)) => {
                            let after_call_label = self.labeler.new_label(&mut labels);
                            let code = resolver.get_compressed_code(method_id);
                            let target_at = self.add_function(&code, method_id, resolver);
                            let code_id = self.function_addresses.get(&target_at).unwrap();
                            let function_start_label = self.function_starts.get(&code_id).unwrap();

                            let callee_this_pointer_offset = layout.operand_stack_entry(current_byte_code_instr_count, descriptor.arg_types.len() as u16);
                            let this_pointer = Register(6);
                            let region_elemant_size_size = Register(5);
                            initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: callee_this_pointer_offset, to: this_pointer }));
                            let this_pointer = this_pointer.to_native_64();
                            let next_rbp = Register(6);
                            initial_ir.push((current_offset, IRInstr::LoadSP { to: next_rbp }));
                            let next_function_layout = resolver.lookup_method_layout(method_id);
                            initial_ir.push((current_offset, IRInstr::GrowStack { amount: next_function_layout.full_frame_size() }));
                            let rip_register = Register(0);
                            initial_ir.push((current_offset, IRInstr::LoadLabel { label: after_call_label, to: rip_register }));
                            let prev_rip_position = layout.full_frame_size();
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: rip_register, to: FramePointerOffset(prev_rip_position) }));
                            let prev_rbp_position = layout.full_frame_size() + size_of::<*mut c_void>();
                            let prev_rbp_register = Register(1);
                            initial_ir.push((current_offset, IRInstr::LoadRBP { to: prev_rbp_register }));
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: prev_rbp_register, to: FramePointerOffset(prev_rbp_position) }));
                            let frame_info_ptr = layout.full_frame_size() + 2 * size_of::<*mut c_void>();
                            let beafbeaf_register = Register(2);
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: beafbeaf_register, const_: 0xbeafbeafbeafbeaf }));
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: beafbeaf_register, to: FramePointerOffset(frame_info_ptr) }));
                            let method_id_register = Register(2);
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: beafbeaf_register, const_: method_id as u64 }));
                            let debug_ptr = layout.full_frame_size() + 3 * size_of::<*mut c_void>(); //todo should use offset of
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: method_id_register, to: FramePointerOffset(debug_ptr) }));
                            let magic_1 = layout.full_frame_size() + 4 * size_of::<*mut c_void>();
                            let magic_1_register = Register(3);
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: magic_1_register, const_: MAGIC_1_EXPECTED }));
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: magic_1_register, to: FramePointerOffset(magic_1) }));
                            let magic_2 = layout.full_frame_size() + 5 * size_of::<*mut c_void>();
                            let magic_2_register = Register(4);
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: magic_2_register, const_: MAGIC_2_EXPECTED }));
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: magic_2_register, to: FramePointerOffset(magic_2) }));
                            let local_vars_start = layout.full_frame_size() + 6 * size_of::<*mut c_void>();
                            let temp_arg_register = Register(5);
                            for i in local_vars_start..descriptor.arg_types.len() {
                                let load_from_location = layout.operand_stack_entry(current_byte_code_instr_count, i as u16);
                                let load_to_location = FramePointerOffset(local_vars_start + i * size_of::<jlong>());
                                initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: load_from_location, to: temp_arg_register }));
                                initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp_arg_register, to: load_to_location }));
                            }
                            //pop this last
                            let load_from_location = layout.operand_stack_entry(current_byte_code_instr_count, descriptor.arg_types.len() as u16);
                            let load_to_location = FramePointerOffset(local_vars_start + descriptor.arg_types.len() * size_of::<jlong>());
                            initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: load_from_location, to: temp_arg_register }));
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp_arg_register, to: load_to_location }));
                            let exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((
                                current_offset,
                                IRInstr::VMExit {
                                    before_exit_label: exit_label,
                                    after_exit_label: None,
                                    exit_type: todo!()/*VMExitType::Trace { values: vec![("obj pointer".to_string(), load_to_location)] }*/,
                                },
                            ));
                            initial_ir.push((current_offset, IRInstr::WriteRBP { from: next_rbp }));
                            initial_ir.push((current_offset, IRInstr::BranchToLabel { label: *function_start_label }));
                            initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: after_call_label })));
                            match descriptor.return_type {
                                CompressedParsedDescriptorType::ByteType => todo!(),
                                CompressedParsedDescriptorType::ShortType => todo!(),
                                CompressedParsedDescriptorType::CharType => todo!(),
                                CompressedParsedDescriptorType::IntType => todo!(),
                                CompressedParsedDescriptorType::LongType => todo!(),
                                CompressedParsedDescriptorType::FloatType => todo!(),
                                CompressedParsedDescriptorType::DoubleType => todo!(),
                                CompressedParsedDescriptorType::VoidType => {}
                                CompressedParsedDescriptorType::BooleanType | CompressedParsedDescriptorType::Ref(_) => {
                                    let return_address = Register(0);
                                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: return_address, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }))
                                }
                            }
                        }
                    }
                }
                CompressedInstructionInfo::isub => {
                    let value_2 = Register(0);
                    let value_1 = Register(1);
                    let res_value = value_1;
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: value_2 }));
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 1), to: value_1 }));
                    initial_ir.push((current_offset, IRInstr::Sub { res: res_value, to_subtract: value_2 }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: res_value, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
                }
                CompressedInstructionInfo::bipush(byte) => {
                    let temp_register = Register(0);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: temp_register, const_: *byte as u64 }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp_register, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
                }
                CompressedInstructionInfo::monitorenter => {
                    let exit_label = self.labeler.new_label(&mut labels);
                    initial_ir.push((
                        current_offset,
                        IRInstr::VMExit {
                            before_exit_label: exit_label,
                            after_exit_label: todo!(),
                            exit_type: todo!()/*VMExitType::MonitorEnter { ref_offset: layout.operand_stack_entry(current_byte_code_instr_count, 0) }*/,
                        },
                    ));
                }
                CompressedInstructionInfo::monitorexit => {
                    let exit_label = self.labeler.new_label(&mut labels);
                    initial_ir.push((
                        current_offset,
                        IRInstr::VMExit {
                            before_exit_label: exit_label,
                            after_exit_label: todo!(),
                            exit_type: todo!()/*VMExitType::MonitorExit { ref_offset: layout.operand_stack_entry(current_byte_code_instr_count, 0) }*/,
                        },
                    ));
                }
                todo => todo!("{:?}", todo),
            }
            initial_ir.push((current_offset, IRInstr::FNOP));
        }
        let mut ir = vec![];

        let mut pending_labels = pending_labels.into_iter().peekable();

        for (offset, ir_instr) in initial_ir {
            loop {
                match pending_labels.peek() {
                    None => break,
                    Some((label_offset, label)) => {
                        if label_offset == &offset {
                            let nop = CompressedInstruction { offset: 0, instruction_size: 0, info: CompressedInstructionInfo::nop };
                            ir.push((*label_offset, IRInstr::Label(IRLabel { name: *label }), nop));
                            let _ = pending_labels.next();
                            continue;
                        }
                    }
                }
                break;
            }
            ir.push((offset, ir_instr, byte_code.iter().find(|instr| instr.offset == offset.0).unwrap().clone().clone()));
        }

        Ok(ToIR { labels, ir, function_start_label })
    }

    fn gen_iconst_n(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, next_byte_code_instr_count: u16, n: u16) {
        let const_register = Register(0);
        initial_ir.push((current_offset, IRInstr::Const64bit { to: const_register, const_: n as i64 as u64 }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: const_register, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }))
    }

    fn gen_istore_n(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, n: u16) {
        let temp = Register(0);
        initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: temp }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp, to: layout.local_var_entry(current_byte_code_instr_count, n) }));
    }

    fn gen_astore_n(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, n: u16) {
        let temp = Register(0);
        initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: temp }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp, to: layout.local_var_entry(current_byte_code_instr_count, n) }));
    }

    fn gen_aload_n(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, next_byte_code_instr_count: u16, n: u16) {
        let temp = Register(0);
        initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.local_var_entry(current_byte_code_instr_count, n), to: temp }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
    }

    fn gen_iload_n(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, next_byte_code_instr_count: u16, n: u16) {
        let temp = Register(0);
        initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.local_var_entry(current_byte_code_instr_count, n), to: temp }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
    }

    fn gen_code_putfield(&mut self, layout: &dyn StackframeMemoryLayout, resolver: MethodResolver, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, name: &FieldName, target_class: &CClassName) {
        let cpd_type = (*target_class).into();
        match resolver.lookup_type_loaded(&cpd_type) {
            None => {
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((current_offset, IRInstr::VMExit { before_exit_label: exit_label, after_exit_label: todo!(), exit_type: todo!()/*VMExitType::InitClass { target_class: cpd_type }*/ }));
            }
            Some((rc, _)) => {
                let (field_number, field_type) = rc.unwrap_class_class().field_numbers.get(name).unwrap();
                let class_ref_register = Register(0);
                let to_put_value = Register(1);
                let offset = Register(2);
                let null = Register(3);
                initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 1), to: class_ref_register }));
                initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: to_put_value }));
                initial_ir.push((current_offset, IRInstr::Const64bit { to: null, const_: 0 }));
                let npe_label = self.labeler.new_label(&mut labels);
                initial_ir.push((current_offset, IRInstr::BranchEqual { a: class_ref_register, b: null, label: npe_label }));
                initial_ir.push((current_offset, IRInstr::Const64bit { to: offset, const_: (field_number * size_of::<jlong>()) as u64 }));
                initial_ir.push((current_offset, IRInstr::Add { res: class_ref_register, a: offset }));
                initial_ir.push((current_offset, IRInstr::Store { to_address: class_ref_register, from: to_put_value }));
                let after_npe_label = self.labeler.new_label(&mut labels);
                initial_ir.push((current_offset, IRInstr::BranchToLabel { label: after_npe_label }));
                initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: npe_label })));
                let npe_exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((current_offset, IRInstr::VMExit { before_exit_label: npe_exit_label, after_exit_label: todo!(), exit_type: todo!()/*VMExitType::NPE {}*/ }));
                initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: after_npe_label })))
            }
        }
    }

    fn gen_code_athrow(&mut self, layout: &dyn StackframeMemoryLayout, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16) {
        let exit_label = self.labeler.new_label(&mut labels);
        initial_ir.push((
            current_offset,
            IRInstr::VMExit {
                before_exit_label: exit_label,
                after_exit_label: todo!(),
                exit_type: todo!()/*VMExitType::Throw { res: layout.operand_stack_entry(current_byte_code_instr_count, 0) }*/,
            },
        ));
    }

    fn gen_code_left_ldc(&mut self, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, layout: &dyn StackframeMemoryLayout, current_offset: ByteCodeOffset, next_byte_code_instr_count: u16, left_ldc: &CompressedLdcW) {
        match left_ldc {
            CompressedLdcW::String { str } => {
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((
                    current_offset,
                    IRInstr::VMExit {
                        before_exit_label: exit_label,
                        after_exit_label: todo!(),
                        exit_type: todo!()/*VMExitType::LoadString { string: str.clone(), res: layout.operand_stack_entry(next_byte_code_instr_count, 0) }*/,
                    },
                ))
            }
            CompressedLdcW::Class { type_ } => {
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((
                    current_offset,
                    IRInstr::VMExit {
                        before_exit_label: exit_label,
                        after_exit_label: todo!(),
                        exit_type: todo!()/*VMExitType::LoadClass {
                            class_type: type_.clone(),
                            res: layout.operand_stack_entry(next_byte_code_instr_count, 0),
                            bytecode_size: 2,
                        }*/,
                    },
                ))
            }
            CompressedLdcW::Float { .. } => todo!(),
            CompressedLdcW::Integer { .. } => todo!(),
            CompressedLdcW::MethodType { .. } => todo!(),
            CompressedLdcW::MethodHandle { .. } => todo!(),
            CompressedLdcW::LiveObject(_) => todo!(),
        }
    }

    fn gen_code_putstatic(&mut self, layout: &dyn StackframeMemoryLayout, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, name: &FieldName, desc: &CFieldDescriptor, target_class: &CClassName) {
        let exit_label = self.labeler.new_label(&mut labels);
        initial_ir.push((
            current_offset,
            IRInstr::VMExit {
                before_exit_label: exit_label,
                after_exit_label: todo!(),
                exit_type: todo!()/*VMExitType::PutStatic {
                    target_class: CPDType::Ref(CPRefType::Class(*target_class)),
                    target_type: desc.0.clone(),
                    name: *name,
                    frame_pointer_offset_of_to_put: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                }*/,
            },
        ))
    }

    fn gen_code_aconst_null(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, next_byte_code_instr_count: u16) {
        let const_register = Register(0);
        initial_ir.push((current_offset, IRInstr::Const64bit { to: const_register, const_: 0 }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: const_register, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }))
    }

    fn gen_code_invokestatic(&mut self, resolver: MethodResolver, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, method_name: &MethodName, descriptor: &CMethodDescriptor, classname_ref_type: &CPRefType) {
        match resolver.lookup_static(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
            None => {
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((
                    current_offset,
                    IRInstr::VMExit {
                        before_exit_label: exit_label,
                        after_exit_label: todo!(),
                        exit_type: todo!()/*VMExitType::ResolveInvokeStatic {
                            method_name: *method_name,
                            desc: descriptor.clone(),
                            target_class: CPDType::Ref(classname_ref_type.clone()),
                        }*/,
                    },
                ));
            }
            Some((method_id, is_native)) => {
                if is_native {
                    let exit_label = self.labeler.new_label(&mut labels);
                    initial_ir.push((
                        current_offset,
                        IRInstr::VMExit {
                            before_exit_label: exit_label,
                            after_exit_label: todo!(),
                            exit_type: todo!()/*VMExitType::RunNativeStatic {
                                method_name: *method_name,
                                desc: descriptor.clone(),
                                target_class: CPDType::Ref(classname_ref_type.clone()),
                            }*/,
                        },
                    ));
                } else {
                    let code = resolver.get_compressed_code(method_id);
                    let target_at = self.add_function(&code, method_id, resolver);
                    todo!()
                }
            }
        }
    }

    fn gen_code_invokespecial(&mut self, layout: &dyn StackframeMemoryLayout, resolver: MethodResolver, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, method_name: &MethodName, descriptor: &CMethodDescriptor, classname_ref_type: &CPRefType) {
        match resolver.lookup_special(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
            None => {
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((
                    current_offset,
                    IRInstr::VMExit {
                        before_exit_label: exit_label,
                        after_exit_label: todo!(),
                        exit_type: todo!()/*VMExitType::ResolveInvokeSpecial {
                            method_name: *method_name,
                            desc: descriptor.clone(),
                            target_class: CPDType::Ref(classname_ref_type.clone()),
                        }*/,
                    },
                ));
            }
            Some((method_id, is_native)) => {
                if is_native {
                    let exit_label = self.labeler.new_label(&mut labels);
                    initial_ir.push((
                        current_offset,
                        IRInstr::VMExit {
                            before_exit_label: exit_label,
                            after_exit_label: todo!(),
                            exit_type: todo!()/*VMExitType::InvokeSpecialNative {
                                method_name: *method_name,
                                desc: descriptor.clone(),
                                target_class: CPDType::Ref(classname_ref_type.clone()),
                            }*/,
                        },
                    ));
                } else {
                    let jvm_method_table = resolver.jvm.method_table.read().unwrap();
                    let (rc, method_i) = jvm_method_table.try_lookup(method_id).unwrap();
                    let view = rc.view();
                    let method_view = view.method_view_i(method_i);
                    // assert!(!is_native);
                    let after_call_label = self.labeler.new_label(&mut labels);
                    let code = resolver.get_compressed_code(method_id);
                    drop(jvm_method_table);
                    let target_at = self.add_function(&code, method_id, resolver);
                    let code_id = self.function_addresses.get(&target_at).unwrap();
                    let function_start_label = self.function_starts.get(&code_id).unwrap();
                    let rip_register = Register(0);
                    let next_rbp = Register(6);
                    initial_ir.push((current_offset, IRInstr::LoadSP { to: next_rbp }));
                    let next_function_layout = resolver.lookup_method_layout(method_id);
                    initial_ir.push((current_offset, IRInstr::GrowStack { amount: next_function_layout.full_frame_size() }));
                    initial_ir.push((current_offset, IRInstr::LoadLabel { label: after_call_label, to: rip_register }));
                    let prev_rip_position = layout.full_frame_size();
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: rip_register, to: FramePointerOffset(prev_rip_position) }));
                    let prev_rbp_position = layout.full_frame_size() + size_of::<*mut c_void>();
                    let prev_rbp_register = Register(1);
                    initial_ir.push((current_offset, IRInstr::LoadRBP { to: prev_rbp_register }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: prev_rbp_register, to: FramePointerOffset(prev_rbp_position) }));
                    let frame_info_ptr = layout.full_frame_size() + 2 * size_of::<*mut c_void>();
                    let beafbeaf_register = Register(2);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: beafbeaf_register, const_: 0xbeafbeafbeafbeaf }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: beafbeaf_register, to: FramePointerOffset(frame_info_ptr) }));
                    let method_id_register = Register(2);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: method_id_register, const_: method_id as u64 }));
                    let method_id_ptr = layout.full_frame_size() + 3 * size_of::<*mut c_void>();
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: method_id_register, to: FramePointerOffset(method_id_ptr) }));
                    let magic_1 = layout.full_frame_size() + 4 * size_of::<*mut c_void>();
                    let magic_1_register = Register(3);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: magic_1_register, const_: MAGIC_1_EXPECTED }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: magic_1_register, to: FramePointerOffset(magic_1) }));
                    let magic_2 = layout.full_frame_size() + 5 * size_of::<*mut c_void>();
                    let magic_2_register = Register(4);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: magic_2_register, const_: MAGIC_2_EXPECTED }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: magic_2_register, to: FramePointerOffset(magic_2) }));
                    let local_vars_start = layout.full_frame_size() + 6 * size_of::<*mut c_void>();
                    let temp_arg_register = Register(5);
                    for i in local_vars_start..descriptor.arg_types.len() {
                        let load_from_location = layout.operand_stack_entry(current_byte_code_instr_count, i as u16);
                        let load_to_location = FramePointerOffset(local_vars_start + i * size_of::<jlong>());
                        initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: load_from_location, to: temp_arg_register }));
                        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp_arg_register, to: load_to_location }));
                    }
                    //pop this last
                    let load_from_location = layout.operand_stack_entry(current_byte_code_instr_count, descriptor.arg_types.len() as u16);
                    let load_to_location = FramePointerOffset(local_vars_start + descriptor.arg_types.len() * size_of::<jlong>());
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: load_from_location, to: temp_arg_register }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp_arg_register, to: load_to_location }));
                    // let one_element_skip = Register(7);
                    // initial_ir.push((current_offset, IRInstr::Const64bit { to: one_element_skip, const_: size_of::<*mut c_void>() as u64 }));
                    // initial_ir.push((current_offset, IRInstr::Add { res: next_rbp, a: one_element_skip }));
                    initial_ir.push((current_offset, IRInstr::WriteRBP { from: next_rbp }));
                    initial_ir.push((current_offset, IRInstr::BranchToLabel { label: *function_start_label }));
                    initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: after_call_label })));
                    match descriptor.return_type {
                        CompressedParsedDescriptorType::BooleanType => todo!(),
                        CompressedParsedDescriptorType::ByteType => todo!(),
                        CompressedParsedDescriptorType::ShortType => todo!(),
                        CompressedParsedDescriptorType::CharType => todo!(),
                        CompressedParsedDescriptorType::IntType => todo!(),
                        CompressedParsedDescriptorType::LongType => todo!(),
                        CompressedParsedDescriptorType::FloatType => todo!(),
                        CompressedParsedDescriptorType::DoubleType => todo!(),
                        CompressedParsedDescriptorType::VoidType => {}
                        CompressedParsedDescriptorType::Ref(_) => todo!(),
                    }
                }
            }
        }
    }

    fn gen_code_dup(layout: &dyn StackframeMemoryLayout, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, next_byte_code_instr_count: u16) {
        let temp_register = Register(0);
        initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.operand_stack_entry(current_byte_code_instr_count, 0), to: temp_register }));
        initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp_register, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
    }

    fn gen_code_anewarray(&mut self, layout: &dyn StackframeMemoryLayout, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, current_byte_code_instr_count: u16, next_byte_code_instr_count: u16, cpdtype: &CPDType) {
        let exit_label = self.labeler.new_label(&mut labels);
        initial_ir.push((
            current_offset,
            IRInstr::VMExit {
                before_exit_label: exit_label,
                after_exit_label: todo!(),
                exit_type: todo!()/*VMExitType::AllocateVariableSizeArrayANewArray {
                    target_type_sub_type: cpdtype.clone(),
                    len_offset: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                    res_write_offset: layout.operand_stack_entry(next_byte_code_instr_count, 0),
                }*/,
            },
        ))
    }

    fn gen_code_new(&mut self, layout: &dyn StackframeMemoryLayout, resolver: MethodResolver, mut labels: &mut Vec<IRLabel>, initial_ir: &mut Vec<(ByteCodeOffset, IRInstr)>, current_offset: ByteCodeOffset, next_byte_code_instr_count: u16, class_name: &CClassName) {
        match resolver.lookup_type_loaded(&(*class_name).into()) {
            None => {
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((
                    current_offset,
                    IRInstr::VMExit {
                        before_exit_label: exit_label,
                        after_exit_label: todo!(),
                        exit_type: todo!()/*VMExitType::InitClass { target_class: CPDType::Ref(CPRefType::Class(*class_name)) }*/,
                    },
                ))
            }
            Some((rc, loader)) => {
                // let allocated_type = runtime_class_to_allocated_object_type(rc.deref(), loader, None,todo!());
                let _todo_manual_allocation_closure = |assembler: &mut CodeAssembler| {
                    let mut start_label = assembler.create_label();
                    assembler.set_label(&mut start_label).unwrap();
                    let _reserved = Register(0).to_native_64();
                    let base = Register(1).to_native_64();
                    let start_index = Register(2).to_native_64();
                    let start_index_div_size = Register(4).to_native_64();
                    let ptr = Register(5).to_native_64();
                    let bitmap_ptr = Register(6).to_native_64();
                    let current_bitmap_ptr = Register(7).to_native_64();
                    let bitscan_res = Register(8).to_native_64();
                    // assembler.mov(start_index, base + offset_of!(RegionData,free_search_index)).unwrap();
                    // assembler.mov(ptr, base + offset_of!(RegionData,ptr)).unwrap();
                    // assembler.mov(start_index, base + offset_of!(RegionData,used_bitmap)).unwrap();
                    assembler.mov(rax, start_index).unwrap();
                    // assembler.mul(allocated_type.size() as i32).unwrap();//todo
                    assembler.mov(start_index, rax).unwrap();
                    assembler.mov(rax, start_index).unwrap();
                    // assembler.div(allocated_type.size()*8).unwrap();//div by size todo
                    assembler.mov(start_index_div_size, rax).unwrap();
                    assembler.mov(current_bitmap_ptr, bitmap_ptr).unwrap();
                    assembler.add(current_bitmap_ptr, start_index_div_size).unwrap();
                    assembler.cmp(current_bitmap_ptr + 0, 0).unwrap(); //all taken, continue
                    let mut increment_start_index = assembler.create_label();
                    assembler.je(increment_start_index).unwrap();
                    assembler.bsf(bitscan_res, current_bitmap_ptr + 0).unwrap(); //todo is this forwards or backwards

                    assembler.set_label(&mut increment_start_index).unwrap();
                    assembler.mov(rax, start_index).unwrap();
                    // assembler.div(allocated_type.size()).unwrap(); //todo
                    assembler.mov(start_index, rax).unwrap();
                    assembler.add(start_index, 1).unwrap();
                    // assembler.mov(base + offset_of!(RegionData,free_search_index), start_index).unwrap();
                    //need to check overflow
                    assembler.jmp(start_label).unwrap();
                    todo!()
                };
                //todo allocate exit
                // let region_data_ptr = resolver.jvm.gc.this_thread_memory_region.with(|memory_region| {
                //     let mut guard = memory_region.borrow_mut();
                //     let region = guard.find_or_new_region_for(allocated_type.clone(), None);
                //     let region_data_ptr = region.get_mut().get_mut() as *mut RegionData;
                //     region_data_ptr
                // });
                // let ptr_offset = offset_of!(RegionData,ptr);
                // let used_bitmap_offset = offset_of!(RegionData,used_bitmap);
                // let region_max_elements_offset = offset_of!(RegionData,region_max_elements);
                // let current_elements_count_offset = offset_of!(RegionData,current_elements_count);
                // let free_search_index_offset = offset_of!(RegionData,free_search_index);
                // let region_data_ptr = Register(0);
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: region_data_ptr, const_: region_data_ptr as usize as u64 }));
                // let current_elements_count = Register(1);
                // let current_elements_count_offset_ = Register(2);
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: current_elements_count, to: region_data_ptr }));
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: current_elements_count_offset_, const_: current_elements_count_offset as u64 }));
                // initial_ir.push((current_offset, IRInstr::Add { res: current_elements_count, a: current_elements_count_offset_ }));
                // initial_ir.push((current_offset, IRInstr::Load { to: current_elements_count, from_address: current_elements_count }));
                // let max_elements_count = Register(3);
                // let max_elements_count_offset_ = Register(4);
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: max_elements_count, to: region_data_ptr }));
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: max_elements_count_offset_, const_: region_max_elements_offset as u64 }));
                // initial_ir.push((current_offset, IRInstr::Add { res: max_elements_count, a: max_elements_count_offset_ }));
                // initial_ir.push((current_offset, IRInstr::Load { to: max_elements_count, from_address: max_elements_count }));
                // let need_new_region_exit = self.labeler.new_label(&mut labels);
                // initial_ir.push((current_offset, IRInstr::BranchEqual {
                //     a: max_elements_count,
                //     b: current_elements_count,
                //     label: need_new_region_exit,
                // }));
                // //todo for now have no gc, so no need to search
                // let free_search_index = Register(3);
                // let free_search_index_offset_ = Register(4);
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: free_search_index, to: region_data_ptr }));
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: free_search_index_offset_, const_: free_search_index_offset as u64 }));
                // initial_ir.push((current_offset, IRInstr::Add { res: free_search_index, a: free_search_index_offset_ }));
                // initial_ir.push((current_offset, IRInstr::Load { to: free_search_index, from_address: free_search_index }));
                //
                //
                // //registers in use beyond here: r1, r2, r3, r4, r5, r6, r7, r8 ,r9
                // let eight = Register(5);
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: eight, const_: 8 }));
                //
                // let current_index = Register(6);
                // let current_index_div_8 = Register(7);
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: current_index, to: current_index_div_8 }));
                // initial_ir.push((current_offset, IRInstr::Div { res: current_index_div_8, divisor: eight }));
                // let current_ptr = Register(8);
                //
                // let current_ptr_bitfield = Register(9);
                //
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: free_search_index, to: current_index }));
                //
                //
                // let ptr_base = Register(1);
                // let ptr_base_offset_ = Register(2);
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: ptr_base, to: region_data_ptr }));
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: ptr_base_offset_, const_: ptr_offset as u64 }));
                // initial_ir.push((current_offset, IRInstr::Add { res: ptr_base, a: ptr_base_offset_ }));
                // initial_ir.push((current_offset, IRInstr::Load { to: ptr_base, from_address: ptr_base }));
                //
                //
                // let bitmap_base = Register(3);
                // let bitmap_base_offset_ = Register(4);
                // initial_ir.push((current_offset, IRInstr::CopyRegister { from: bitmap_base, to: region_data_ptr }));
                // initial_ir.push((current_offset, IRInstr::Const64bit { to: bitmap_base_offset_, const_: used_bitmap_offset as u64 }));
                // initial_ir.push((current_offset, IRInstr::Add { res: bitmap_base, a: bitmap_base_offset_ }));
                // initial_ir.push((current_offset, IRInstr::Load { to: bitmap_base, from_address: bitmap_base }));
                //
                // initial_ir.push((current_offset, IRInstr::VMExit {
                //     exit_label,
                //     exit_type: VMExitType::NeedNewRegion {
                //         target_class: allocated_type,
                //     },
                // }));
                let exit_label = self.labeler.new_label(&mut labels);
                initial_ir.push((
                    current_offset,
                    IRInstr::VMExit {
                        before_exit_label: exit_label,
                        after_exit_label: todo!(),
                        exit_type: todo!()/*VMExitType::Allocate {
                            ptypeview: rc.cpdtype(),
                            loader,
                            res: layout.operand_stack_entry(next_byte_code_instr_count, 0),
                            bytecode_size: 3,
                        }*/,
                    },
                ));
            }
        }
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
                IRInstr::VMExit { before_exit_label: exit_label, exit_type, .. } => {
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
                    assembler.nop().unwrap()
                    // match exit_type.clone(){
                    //     VMExitType::ResolveInvokeStatic { method_name, desc, target_class } => {
                    //
                    //     }
                    //     VMExitType::TopLevelReturn { .. } => {
                    //         todo!()
                    //     }
                    // }
                }
                IRInstr::Label(label) => {
                    let iced_label = iced_labels.get_mut(&label.name).unwrap();
                    label_instruction_offsets.push((label.name, assembler.instructions().len() as u32));
                    assembler.set_label(iced_label).unwrap();
                    assembler.nop().unwrap();
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
                IRInstr::FNOP => {
                    // assembler.fnop().unwrap();
                }
                IRInstr::CopyRegister { .. } => todo!(),
                IRInstr::BinaryBitAnd { .. } => todo!(),
                IRInstr::ForwardBitScan { .. } => todo!(),
                IRInstr::WithAssembler { .. } => todo!(),
                IRInstr::IRNewFrame { .. } => todo!(),
                IRInstr::VMExit2 { .. } => {}
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
        let result = BlockEncoder::encode(assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
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

    pub fn recompile_method_and_restart(jit_state: &RefCell<JITedCodeState>, methodid: usize, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, code: &CompressedCode, old_java_ip: *mut c_void, transition_type: TransitionType) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        transition_stack_frame(transition_type, todo!()/*int_state.java_stack()*/);
        let instruct_pointer = todo!()/*int_state.java_stack().saved_registers().instruction_pointer*/;
        assert_eq!(instruct_pointer, old_java_ip);
        let compiled_code_id = *jit_state.borrow().function_addresses.get(&instruct_pointer).unwrap();
        let temp = jit_state.borrow();
        let compiled_code = temp.address_to_byte_code_offset.get(&compiled_code_id).unwrap();
        // problem here is that a function call overwrites the old old ip
        let return_to_byte_code_offset = *compiled_code.get(&instruct_pointer).unwrap();
        drop(temp);
        let new_base_address = jit_state.borrow_mut().add_function(code, methodid, MethodResolver { jvm, loader: int_state.current_loader(jvm) });
        let new_code_id = *jit_state.borrow().function_addresses.get(&new_base_address).unwrap();
        let start_byte_code_addresses = jit_state.borrow().address_to_byte_code_offset.get(&new_code_id).unwrap().get_reverse(&return_to_byte_code_offset).unwrap().clone();
        let restart_execution_at = start_byte_code_addresses.start;
        unsafe { Self::resume_method(jit_state, restart_execution_at, jvm, int_state, methodid, new_code_id) }
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
    unsafe fn resume_method(jit_state: &RefCell<JITedCodeState>, mut target_ip: *mut c_void, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        loop {
            //todo reacrchited pushing/popping of frames storing sp.
            let java_stack: &mut JavaStack = todo!();//int_state.java_stack();
            let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: as_ptr, status_register } = java_stack.handle_vm_entry();
            let rust_stack: u64 = stack_pointer as u64;
            let rust_frame: u64 = frame_pointer as u64;
            let memory_region: MutexGuard<MemoryRegions> = jvm.gc.memory_region.lock().unwrap();
            let mut jit_code_context = JitCodeContext {
                native_saved: SavedRegisters {
                    stack_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    frame_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    instruction_pointer: 0xdeaddeaddeaddead as *mut c_void,
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
            let old_java_ip: *mut c_void;
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
    pub unsafe fn run_method(jitstate: &RefCell<JITedCodeState>, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        let target_ip = jitstate.borrow().function_addresses.get_reverse(&compiled_id).unwrap().start;
        drop(jitstate.borrow_mut());
        JITedCodeState::resume_method(jitstate, target_ip, jvm, int_state, methodid, compiled_id)
    }

    #[allow(unaligned_references)]
    fn handle_exit(jitstate: &RefCell<JITedCodeState>, exit_type: VMExitType, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: usize, old_java_ip: *mut c_void) -> Option<*mut c_void> {
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
                        len: arr_len.unwrap(),
                        sub_type_loader: loader,
                    };
                }
                RuntimeClass::Top => panic!(),
            };
            AllocatedObjectType::PrimitiveArray { thread: thread_id, primitive_type, len: arr_len.unwrap() }
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

pub fn setup_args_from_current_frame(jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, desc: &CMethodDescriptor, is_virtual: bool) -> Vec<JavaValue<'gc_life>> {
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