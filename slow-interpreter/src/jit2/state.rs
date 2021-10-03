use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::cmp::max;
use std::collections::HashMap;
use std::env::current_exe;
use std::error::Error;
use std::ffi::c_void;
use std::fs::read_to_string;
use std::intrinsics::copy_nonoverlapping;
use std::mem::size_of;
use std::ops::Deref;
use std::ptr::null_mut;
use std::sync::Arc;
use std::thread;
use std::thread::LocalKey;

use bimap::BiHashMap;
use iced_x86::{BlockEncoder, Formatter, InstructionBlock};
use iced_x86::BlockEncoderOptions;
use iced_x86::code_asm::{CodeAssembler, dword_ptr, qword_ptr, r15, rax, rbp, rsp};
use iced_x86::ConditionCode::l;
use iced_x86::IntelFormatter;
use iced_x86::OpCodeOperandKind::cl;
use itertools::Itertools;
use memoffset::offset_of;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::{AllocatedObjectType, ArrayMemoryLayout, FrameHeader, FramePointerOffset, MAGIC_1_EXPECTED, MAGIC_2_EXPECTED, MemoryRegions, ObjectMemoryLayout, StackframeMemoryLayout};
use jit_common::{JitCodeContext, SavedRegisters};
use jit_common::java_stack::JavaStack;
use jvmti_jni_bindings::{jdouble, jlong, jobject, jvalue};
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedCode, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::names::CompressedClassName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::class_loading::{assert_loaded_class, check_initing_or_inited_class};
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jit2::{ByteCodeOffset, CompiledCodeID, IRInstructionIndex, LabelName, MethodResolver, NotSupported, ToIR, ToNative, transition_stack_frame, TransitionType, VMExitType};
use crate::jit2::ir::{IRInstr, IRLabel, Register};
use crate::jit2::state::birangemap::BiRangeMap;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};

thread_local! {
pub static JITSTATE : RefCell<JITState> = RefCell::new(JITState::new());
}

//could be own crate
pub mod birangemap;

pub struct JITState {
    code: *mut c_void,
    current_max_compiled_code_id: CompiledCodeID,
    method_id_to_code: HashMap<usize, CompiledCodeID>,
    current_end: *mut c_void,
    // indexed by compiled id:
    function_addresses: BiRangeMap<*mut c_void, CompiledCodeID>,
    function_starts: HashMap<CompiledCodeID, LabelName>,
    current_jit_instr: IRInstructionIndex,
    exits: HashMap<*mut c_void, VMExitType>,
    labels: HashMap<LabelName, *mut c_void>,
    labeler: Labeler,
    pub top_level_exit_code: *mut c_void,
    address_to_byte_code_offset: HashMap<CompiledCodeID, BiRangeMap<*mut c_void, ByteCodeOffset>>,
}

impl JITState {
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
            labeler: Labeler { current_label: 0 },
            top_level_exit_code: null_mut(),
            address_to_byte_code_offset: HashMap::new(),
            function_starts: HashMap::new(),
        };
        res.top_level_exit_code = res.add_top_level_exit_code();
        res
    }


    fn add_top_level_exit_code(&mut self) -> *mut c_void {
        let mut labels = vec![];
        let start_label = self.labeler.new_label(&mut labels);
        let exit_label = self.labeler.new_label(&mut labels);
        let ir = ToIR {
            labels,
            ir: vec![(ByteCodeOffset(0), IRInstr::Label { 0: IRLabel { name: start_label } }), (ByteCodeOffset(0), IRInstr::VMExit { exit_label, exit_type: VMExitType::TopLevelReturn {} })],
            function_start_label: start_label,
        };

        let current_code_id = self.next_code_id((-1isize) as usize);
        self.add_from_ir("top level exit wrapper function".to_string(), current_code_id, ir)
    }

    fn next_code_id(&mut self, method_id: MethodId) -> CompiledCodeID {
        let next_code_id = CompiledCodeID(self.current_max_compiled_code_id.0 + 1);
        self.current_max_compiled_code_id = next_code_id;
        assert!(!self.method_id_to_code.values().contains(&next_code_id));
        self.method_id_to_code.insert(method_id, next_code_id);
        next_code_id
    }

    pub fn add_function(&mut self, code: &CompressedCode, methodid: usize, resolver: MethodResolver<'l>) -> *mut c_void {
        let current_code_id = self.next_code_id(methodid);
        let CompressedCode {
            instructions,
            max_locals,
            max_stack,
            exception_table,
            stack_map_table
        } = code;
        let cinstructions = instructions.iter().sorted_by_key(|(offset, _)| **offset).map(|(_, ci)| ci).collect_vec();
        let layout = NaiveStackframeLayout::new(&cinstructions, *max_locals, *max_stack);
        let ir = self.to_ir(cinstructions, &layout, resolver).unwrap();
        let (current_rc, method_i) = resolver.jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
        let view = current_rc.unwrap_class_class().class_view.clone();
        let method_view = view.method_view_i(method_i);
        let method_name = method_view.name().0.to_str(&resolver.jvm.string_pool);
        let class_name = view.name().unwrap_name().0.to_str(&resolver.jvm.string_pool);
        let res_ptr = self.add_from_ir(format!("{} {}", class_name, method_name), current_code_id, ir);
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
            dbg!(current_byte_code_instr_count);
            dbg!(next_byte_code_instr_count);
            dbg!(byte_code_instr);
            match &byte_code_instr.info {
                CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                    match resolver.lookup_static(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
                        None => {
                            let exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::VMExit { exit_label, exit_type: VMExitType::ResolveInvokeStatic { method_name: *method_name, desc: descriptor.clone(), target_class: CPDType::Ref(classname_ref_type.clone()) } }));
                        }
                        Some((method_id, is_native)) => {
                            if is_native {
                                let exit_label = self.labeler.new_label(&mut labels);
                                initial_ir.push((current_offset, IRInstr::VMExit { exit_label, exit_type: VMExitType::RunNativeStatic { method_name: *method_name, desc: descriptor.clone(), target_class: CPDType::Ref(classname_ref_type.clone()) } }));
                            } else {
                                let code = resolver.get_compressed_code(method_id);
                                let target_at = self.add_function(&code, method_id, resolver);
                                todo!()
                            }
                        }
                    }
                }
                CompressedInstructionInfo::ifnull(offset) => {
                    let branch_to_label = self.labeler.new_label(&mut labels);
                    pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                    let possibly_null_register = Register(0);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative {
                        from: layout.operand_stack_entry(current_byte_code_instr_count, 0),
                        to: possibly_null_register,
                    }));
                    let register_with_null = Register(1);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: register_with_null, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::BranchEqual { a: register_with_null, b: possibly_null_register, label: branch_to_label }))
                }
                CompressedInstructionInfo::return_ => {
                    initial_ir.push((current_offset, IRInstr::Return { return_val: None }));
                }
                CompressedInstructionInfo::aload_0 => {
                    let temp = Register(0);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.local_var_entry(current_byte_code_instr_count, 0), to: temp }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp, to: dbg!(layout.operand_stack_entry(next_byte_code_instr_count, 0)) }));
                }
                CompressedInstructionInfo::aload_1 => {
                    let temp = Register(0);
                    initial_ir.push((current_offset, IRInstr::LoadFPRelative { from: layout.local_var_entry(current_byte_code_instr_count, 0), to: temp }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: temp, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }));
                    //todo dup
                }
                CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                    match resolver.lookup_special(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
                        None => {
                            let exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::VMExit { exit_label, exit_type: VMExitType::ResolveInvokeSpecial { method_name: *method_name, desc: descriptor.clone(), target_class: CPDType::Ref(classname_ref_type.clone()) } }));
                        }
                        Some((method_id, is_native)) => {
                            assert!(!is_native);
                            let after_call_label = self.labeler.new_label(&mut labels);
                            let code = resolver.get_compressed_code(method_id);
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
                            let debug_ptr = layout.full_frame_size() + 3 * size_of::<*mut c_void>();
                            initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: beafbeaf_register, to: FramePointerOffset(debug_ptr) }));
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
                            let one_element_skip = Register(7);
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: one_element_skip, const_: size_of::<*mut c_void>() as u64 }));
                            initial_ir.push((current_offset, IRInstr::Add { res: next_rbp, a: one_element_skip }));
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
                                CompressedParsedDescriptorType::Ref(_) => todo!()
                            }
                        }
                    }
                    //todo need to not constantly call same.
                }
                CompressedInstructionInfo::iconst_0 => {
                    let const_register = Register(0);
                    initial_ir.push((current_offset, IRInstr::Const32bit { to: const_register, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: const_register, to: dbg!(layout.operand_stack_entry(next_byte_code_instr_count, 0)) }))
                }
                CompressedInstructionInfo::iconst_1 => {
                    //todo dup
                    let const_register = Register(0);
                    initial_ir.push((current_offset, IRInstr::Const32bit { to: const_register, const_: 1 }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: const_register, to: layout.operand_stack_entry(next_byte_code_instr_count, 0) }))
                }
                CompressedInstructionInfo::putfield { name, desc, target_class } => {
                    let cpd_type = (*target_class).into();
                    match resolver.lookup_type_loaded(&cpd_type) {
                        None => {
                            let exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::VMExit { exit_label, exit_type: VMExitType::InitClass { target_class: cpd_type } }));
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
                            initial_ir.push((current_offset, IRInstr::BranchEqual {
                                a: class_ref_register,
                                b: null,
                                label: npe_label,
                            }));
                            initial_ir.push((current_offset, IRInstr::Const64bit { to: offset, const_: (field_number * size_of::<jlong>()) as u64 }));
                            initial_ir.push((current_offset, IRInstr::Add {
                                res: class_ref_register,
                                a: offset,
                            }));
                            initial_ir.push((current_offset, IRInstr::Store { to_address: class_ref_register, from: to_put_value }));
                            let after_npe_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::BranchToLabel { label: after_npe_label }));
                            initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: npe_label })));
                            let npe_exit_label = self.labeler.new_label(&mut labels);
                            initial_ir.push((current_offset, IRInstr::VMExit { exit_label: npe_exit_label, exit_type: VMExitType::Todo {} }));
                            initial_ir.push((current_offset, IRInstr::Label(IRLabel { name: after_npe_label })))
                        }
                    }
                }
                CompressedInstructionInfo::aconst_null => {
                    let const_register = Register(0);
                    initial_ir.push((current_offset, IRInstr::Const64bit { to: const_register, const_: 0 }));
                    initial_ir.push((current_offset, IRInstr::StoreFPRelative { from: const_register, to: dbg!(layout.operand_stack_entry(next_byte_code_instr_count, 0)) }))
                }
                CompressedInstructionInfo::putstatic { name, desc, target_class } => {
                    let exit_label = self.labeler.new_label(&mut labels);
                    initial_ir.push((current_offset, IRInstr::VMExit { exit_label, exit_type: VMExitType::PutStatic { target_class: CPDType::Ref(CPRefType::Class(*target_class)), target_type: desc.0.clone(), name: *name, frame_pointer_offset_of_to_put: layout.operand_stack_entry(current_byte_code_instr_count, 0) } }))
                }
                CompressedInstructionInfo::anewarray(cpdtype) => {
                    let exit_label = self.labeler.new_label(&mut labels);
                    initial_ir.push((current_offset, IRInstr::VMExit {
                        exit_label,
                        exit_type: VMExitType::AllocateVariableSizeArrayANewArray {
                            target_type_sub_type: cpdtype.clone(),
                            len_offset: layout.operand_stack_entry(
                                current_byte_code_instr_count,
                                0,
                            ),
                            res_write_offset: layout.operand_stack_entry(next_byte_code_instr_count, 0),
                        },
                    }))
                }
                todo => todo!("{:?}", todo)
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
                            ir.push((*label_offset, IRInstr::Label(IRLabel { name: *label })));
                            let _ = pending_labels.next();
                            continue;
                        }
                    }
                }
                break;
            }
            ir.push((offset, ir_instr));
        }

        Ok(ToIR {
            labels,
            ir,
            function_start_label,
        })
    }


    pub fn ir_to_native(&self, ir: ToIR, base_address: *mut c_void, method_log_info: String) -> ToNative {
        let ToIR { labels: ir_labels, ir, function_start_label } = ir;
        let mut exits = HashMap::new();
        let mut assembler = CodeAssembler::new(64).unwrap();
        let mut iced_labels = ir_labels.into_iter().map(|label| (label.name, assembler.create_label())).collect::<HashMap<_, _>>();
        let mut label_instruction_offsets: Vec<(LabelName, u32)> = vec![];
        let mut instruction_index_to_bytecode_offset_start: HashMap<u32, ByteCodeOffset> = HashMap::new();
        for (bytecode_offset, ir_instr) in ir {
            instruction_index_to_bytecode_offset_start.insert(assembler.instructions().len() as u32, bytecode_offset);
            match ir_instr {
                IRInstr::LoadFPRelative { from, to } => {
                    assembler.mov(to.to_native_64(), rbp + from.0).unwrap();
                }
                IRInstr::StoreFPRelative { from, to } => {
                    assembler.mov(rbp + to.0, from.to_native_64()).unwrap();
                }
                IRInstr::Load { .. } => todo!(),
                IRInstr::Store { from, to_address } => {
                    assembler.mov(qword_ptr(to_address.to_native_64()), from.to_native_64()).unwrap();
                }
                IRInstr::Add { res, a } => {
                    assembler.add(res.to_native_64(), a.to_native_64()).unwrap();
                }
                IRInstr::Sub { .. } => todo!(),
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
                IRInstr::VMExit { exit_label, exit_type } => {
                    let native_stack_pointer = (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,stack_pointer)) as i64;
                    let native_frame_pointer = (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,frame_pointer)) as i64;
                    let native_instruction_pointer = (offset_of!(JitCodeContext,native_saved) + offset_of!(SavedRegisters,instruction_pointer)) as i64;
                    let java_stack_pointer = (offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,stack_pointer)) as i64;
                    let java_frame_pointer = (offset_of!(JitCodeContext,java_saved) + offset_of!(SavedRegisters,frame_pointer)) as i64;
                    let exit_handler_ip = offset_of!(JitCodeContext,exit_handler_ip) as i64;
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
                IRInstr::Return { return_val } => {
                    if let Some(return_register) = return_val {
                        assembler.mov(rax, return_register.to_native_64()).unwrap();
                    }
                    //rsp is now equal is to prev rbp + 1, so that we can pop the previous rip in ret
                    assembler.mov(rsp, rbp).unwrap();
                    assert_eq!(offset_of!(FrameHeader,prev_rip), 0);
                    //load prev fram pointer
                    assembler.mov(rbp, rbp + offset_of!(FrameHeader,prev_rpb)).unwrap();
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
                    assembler.fnop().unwrap();
                }
            }
        }
        let block = InstructionBlock::new(assembler.instructions(), base_address as u64);
        let mut formatted_instructions = String::new();
        let mut formatter = IntelFormatter::default();
        for (i, instruction) in assembler.instructions().iter().enumerate() {
            formatted_instructions.push_str(format!("#{}:", i).as_str());
            formatter.format(instruction, &mut formatted_instructions);
            formatted_instructions.push('\n');
        }
        eprintln!("{}", format!("{} :\n{}", method_log_info, formatted_instructions));
        let result = BlockEncoder::encode(assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
        let mut bytecode_offset_to_address = BiRangeMap::new();
        let mut new_labels: HashMap<LabelName, *mut c_void> = Default::default();
        let mut label_instruction_indexes = label_instruction_offsets.into_iter().peekable();
        let mut current_byte_code_offset = Some(ByteCodeOffset(0));
        let mut current_byte_code_start_address = Some(base_address);
        for (i, native_offset) in result.new_instruction_offsets.iter().enumerate() {
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
            if let Some(new_byte_code_offset) = instruction_index_to_bytecode_offset_start.get(&(i as u32)) {
                assert!((current_byte_code_start_address.unwrap() as u64) < (current_instruction_address as u64));
                bytecode_offset_to_address.insert_range(current_byte_code_start_address.unwrap()..current_instruction_address, current_byte_code_offset.unwrap());
                current_byte_code_offset = Some(*new_byte_code_offset);
                current_byte_code_start_address = Some(current_instruction_address);
            }
        }
        assert!(label_instruction_indexes.peek().is_none());
        ToNative { code: result.code_buffer, new_labels, bytecode_offset_to_address, exits, function_start_label }
    }


    fn add_from_ir(&mut self, method_log_info: String, current_code_id: CompiledCodeID, ir: ToIR) -> *mut c_void {
        let ToNative {
            code,
            new_labels,
            bytecode_offset_to_address,
            exits,
            function_start_label
        } = self.ir_to_native(ir, self.current_end, method_log_info);
        self.function_starts.insert(current_code_id, function_start_label);
        let install_at = self.current_end;
        unsafe { self.current_end = install_at.offset(code.len() as isize); }
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
        for (address_range, offset) in bytecode_offset_to_address {
            self.address_to_byte_code_offset.entry(current_code_id).or_insert(BiRangeMap::new()).insert_range(address_range, offset);
        }
        unsafe {
            copy_nonoverlapping(
                code.as_ptr(),
                install_at as *mut u8,
                code.len(),
            )
        }
        unsafe { self.function_addresses.insert_range(install_at..(install_at.offset(code.len() as isize)), current_code_id); }
        install_at
    }

    pub fn recompile_method_and_restart(jit_state: &RefCell<JITState>,
                                        methodid: usize,
                                        jvm: &'gc_life JVMState<'gc_life>,
                                        int_state: &mut InterpreterStateGuard<'gc_life, 'l>,
                                        code: &CompressedCode,
                                        transition_type: TransitionType,
    ) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        transition_stack_frame(transition_type, int_state.get_java_stack());
        let instruct_pointer = int_state.get_java_stack().saved_registers().instruction_pointer;
        let compiled_code_id = *jit_state.borrow().function_addresses.get(&instruct_pointer).unwrap();
        let return_to_byte_code_offset = *jit_state.borrow().address_to_byte_code_offset.get(&compiled_code_id).unwrap().get(&instruct_pointer).unwrap();
        let new_base_address = jit_state.borrow_mut().add_function(code, methodid, MethodResolver { jvm, loader: int_state.current_loader() });
        let new_code_id = *jit_state.borrow().function_addresses.get(&new_base_address).unwrap();
        let start_byte_code_addresses = jit_state.borrow().address_to_byte_code_offset.get(&new_code_id).unwrap().get_reverse(&return_to_byte_code_offset).unwrap().clone();
        let restart_execution_at = start_byte_code_addresses.start;
        unsafe { Self::resume_method(jit_state, restart_execution_at, jvm, int_state, methodid, new_code_id) }
    }

    pub fn run_method_safe(jit_state: &RefCell<JITState>, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, 'l>, methodid: MethodId) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        let res = unsafe {
            let code_id = jit_state.borrow().method_id_to_code[&methodid];
            JITState::run_method(jit_state, jvm, int_state, methodid, code_id)
        };
        res
    }

    #[allow(named_asm_labels)]
    unsafe fn resume_method(jit_state: &RefCell<JITState>, mut target_ip: *mut c_void, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        loop {
            let java_stack: &mut JavaStack = int_state.get_java_stack();
            let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: as_ptr, status_register } = java_stack.handle_vm_entry();
            let rust_stack: u64 = stack_pointer as u64;
            let rust_frame: u64 = frame_pointer as u64;
            dbg!(frame_pointer);

            let mut jit_code_context = JitCodeContext {
                native_saved: SavedRegisters {
                    stack_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    frame_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    instruction_pointer: 0xdeaddeaddeaddead as *mut c_void,
                    status_register,
                },
                java_saved: SavedRegisters {
                    stack_pointer,
                    frame_pointer,
                    instruction_pointer: target_ip,
                    status_register,
                },
                exit_handler_ip: null_mut(),
            };
            eprintln!("going in");
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
            asm!(
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
            );
            jit_code_context.java_saved.instruction_pointer = old_java_ip;
            java_stack.saved_registers = Some(jit_code_context.java_saved.clone());
            //todo exception handling
            eprintln!("going out ");
            let exit_type = jit_state.borrow().exits.get(&old_java_ip).unwrap().clone();
            drop(jit_state.borrow_mut());
            target_ip = match JITState::handle_exit(jit_state, exit_type, jvm, int_state, methodid, old_java_ip) {
                None => {
                    return Ok(None);
                }
                Some(target_ip) => target_ip
            };
        }
    }

    #[allow(named_asm_labels)]
    pub unsafe fn run_method(jitstate: &RefCell<JITState>, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: MethodId, compiled_id: CompiledCodeID) -> Result<Option<JavaValue<'gc_life>>, WasException> {
        let target_ip = jitstate.borrow().function_addresses.get_reverse(&compiled_id).unwrap().start;
        drop(jitstate.borrow_mut());
        JITState::resume_method(jitstate, target_ip, jvm, int_state, methodid, compiled_id)
    }
    fn handle_exit(jitstate: &RefCell<JITState>, exit_type: VMExitType, jvm: &'gc_life JVMState<'gc_life>, int_state: &mut InterpreterStateGuard<'gc_life, '_>, methodid: usize, old_java_ip: *mut c_void) -> Option<*mut c_void> {
        match exit_type {
            VMExitType::ResolveInvokeStatic { method_name, desc, target_class } => {
                let inited_class = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                let method_view = inited_class.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap();
                let to_call_function_method_id = jvm.method_table.write().unwrap().get_method_id(inited_class.clone(), method_view.method_i());
                if method_view.is_native() {
                    match run_native_method(jvm, int_state, inited_class.clone(), method_view.method_i()) {
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
                    Self::recompile_method_and_restart(jitstate, methodid, jvm, int_state, code, TransitionType::ResolveCalls).unwrap();
                    todo!()
                }
            }
            VMExitType::TopLevelReturn { .. } => {
                dbg!(int_state.current_loader());
                int_state.set_function_return(true);
                None
            }
            VMExitType::ResolveInvokeSpecial { method_name, desc, target_class } => {
                let inited_class = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                let method_view = inited_class.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap();
                let to_call_function_method_id = jvm.method_table.write().unwrap().get_method_id(inited_class.clone(), method_view.method_i());
                jitstate.borrow_mut().add_function(method_view.code_attribute().unwrap(), to_call_function_method_id, MethodResolver { jvm, loader: int_state.current_loader() });
                let (current_function_rc, current_function_method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
                let method_view = current_function_rc.unwrap_class_class().class_view.method_view_i(current_function_method_i);
                let code = method_view.code_attribute().unwrap();
                Self::recompile_method_and_restart(jitstate, methodid, jvm, int_state, code, TransitionType::ResolveCalls).unwrap();
                todo!()
            }
            VMExitType::Todo { .. } => {
                todo!()
            }
            VMExitType::RunNativeStatic { method_name, desc, target_class } => {
                let inited = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                let method_i = inited.unwrap_class_class().class_view.lookup_method(method_name, &desc).unwrap().method_i();
                let res = run_native_method(jvm, int_state, inited, method_i).unwrap();
                dbg!(&res);
                match res {
                    None => {}
                    Some(_) => {
                        todo!()
                    }
                }
                let jit_state = jitstate.borrow();
                let code_id = jit_state.method_id_to_code[&methodid];
                let address_to_bytecode_for_this_method = jit_state.address_to_byte_code_offset.get(&code_id).unwrap();
                let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + 3);// call static is 3 bytes
                let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                Some(restart_address)
            }
            VMExitType::PutStatic { target_type, target_class, name, frame_pointer_offset_of_to_put } => {
                let target_class_rc = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                // dbg!(target_type.clone());
                // let _target_type_rc = assert_loaded_class(jvm, target_type.clone());
                target_class_rc.static_vars().insert(name, int_state.raw_read_at_frame_pointer_offset(frame_pointer_offset_of_to_put, target_type.to_runtime_type().unwrap()));
                //todo dup
                let jitstate_borrow = jitstate.borrow();
                let code_id = jitstate_borrow.method_id_to_code[&methodid];
                let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + 3);// put static is 3 bytes
                let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                Some(restart_address)
            }
            VMExitType::AllocateVariableSizeArrayANewArray { target_type_sub_type, len_offset, res_write_offset } => {
                drop(jitstate.borrow_mut());
                let inited_target_type_rc = check_initing_or_inited_class(jvm, int_state, target_type_sub_type).unwrap();
                let array_len = int_state.raw_read_at_frame_pointer_offset(len_offset, RuntimeType::IntType).unwrap_int() as usize;
                use gc_memory_layout_common::THIS_THREAD_MEMORY_REGIONS;
                let allocated_object_type = runtime_class_to_allocated_object_type(&inited_target_type_rc, int_state.current_loader(), array_len);
                let mut guard = THIS_THREAD_MEMORY_REGIONS.lock().unwrap();
                let region_data = guard.find_or_new_region_for(allocated_object_type, None);
                let allocation = region_data.get_allocation();
                let to_write = jvalue { l: allocation as jobject };
                int_state.raw_write_at_frame_pointer_offset(res_write_offset, to_write);
                let jitstate_borrow = jitstate.borrow();
                let code_id = jitstate_borrow.method_id_to_code[&methodid];
                let address_to_bytecode_for_this_method = jitstate_borrow.address_to_byte_code_offset.get(&code_id).unwrap();
                let bytecode_offset = address_to_bytecode_for_this_method.get(&old_java_ip).unwrap();
                let restart_bytecode_offset = ByteCodeOffset(bytecode_offset.0 + 3);// anewarray is 3 bytes
                let restart_address = address_to_bytecode_for_this_method.get_reverse(&restart_bytecode_offset).unwrap().start;
                Some(restart_address)
            }
            VMExitType::InitClass { target_class } => {
                let inited_target_type_rc = check_initing_or_inited_class(jvm, int_state, target_class).unwrap();
                let (current_function_rc, current_function_method_i) = jvm.method_table.read().unwrap().try_lookup(methodid).unwrap();
                let method_view = current_function_rc.unwrap_class_class().class_view.method_view_i(current_function_method_i);
                let code = method_view.code_attribute().unwrap();
                Self::recompile_method_and_restart(jitstate, methodid, jvm, int_state, code, TransitionType::ResolveCalls).unwrap();
                todo!()
            }
        }
    }
}

pub fn runtime_class_to_allocated_object_type(ref_type: &RuntimeClass, loader: LoaderName, arr_len: usize) -> AllocatedObjectType {
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
                        sub_type: arr.sub_class.cpdtype().unwrap_ref_type().clone(),
                        len: arr_len,
                        sub_type_loader: loader,
                    };
                }
                RuntimeClass::Top => panic!()
            };
            AllocatedObjectType::PrimitiveArray { primitive_type, len: arr_len }
        }
        RuntimeClass::Object(class_class) => {
            AllocatedObjectType::Class {
                name: class_class.class_view.name().unwrap_name(),
                loader,
                size: class_class.recursive_num_fields * size_of::<jlong>(),
            }
        }
        RuntimeClass::Top => panic!(),
    }
}


pub struct Labeler {
    current_label: u32,
}

impl Labeler {
    pub fn new_label(&mut self, labels_vec: &mut Vec<IRLabel>) -> LabelName {
        let current_label = self.current_label.checked_add(1).unwrap();
        self.current_label = current_label;
        let res = LabelName(current_label);
        labels_vec.push(IRLabel { name: res });
        res
    }
}


pub struct NaiveStackframeLayout {
    pub(crate) max_locals: u16,
    pub(crate) max_stack: u16,
    pub(crate) stack_depth: HashMap<u16, u16>,
}

impl NaiveStackframeLayout {
    pub fn new(instructions: &Vec<&CInstruction>, max_locals: u16, max_stack: u16) -> Self {
        let mut stack_depth = HashMap::new();
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
                CompressedInstructionInfo::aload_0 => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::aload_1 => {
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
                CompressedInstructionInfo::aconst_null => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::iconst_1 {} => {
                    current_depth += 1;
                }
                CompressedInstructionInfo::putstatic { name, desc, target_class } => {
                    current_depth -= 1;
                }
                CompressedInstructionInfo::anewarray(_) => {}
                todo => todo!("{:?}", todo)
            }
        }
        dbg!(stack_depth.iter().map(|(offset, depth)| (*offset, *depth)).sorted_by_key(|(offset, _)| *offset).collect_vec());
        Self {
            max_locals,
            max_stack,
            stack_depth,
        }
    }
}

impl StackframeMemoryLayout for NaiveStackframeLayout {
    fn local_var_entry(&self, current_count: u16, i: u16) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + i as usize * size_of::<jlong>())
    }

    fn operand_stack_entry(&self, current_count: u16, from_end: u16) -> FramePointerOffset {
        FramePointerOffset(size_of::<FrameHeader>() + (self.max_locals + dbg!(self.stack_depth[&current_count]) - from_end) as usize * size_of::<jlong>())
    }

    fn operand_stack_entry_array_layout(&self, pc: u16, from_end: u16) -> ArrayMemoryLayout {
        todo!()
    }

    fn operand_stack_entry_object_layout(&self, pc: u16, from_end: u16) -> ObjectMemoryLayout {
        todo!()
    }

    fn full_frame_size(&self) -> usize {
        size_of::<FrameHeader>() + (self.max_locals as usize + self.max_stack as usize + 1) * size_of::<jlong>()// max stack is maximum depth which means we need 1 one more for size
    }

    fn safe_temp_location(&self, pc: u16, i: u16) -> FramePointerOffset {
        todo!()
    }
}

