use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::intrinsics::copy_nonoverlapping;
use std::mem::{size_of};
use std::ops::{Deref};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

use bimap::BiHashMap;
use iced_x86::code_asm::CodeAssembler;
use itertools::Itertools;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use another_jit_vm_ir::compiler::{IRInstr, IRLabel, LabelName};
use another_jit_vm_ir::vm_exit_abi::VMExitTypeWithArgs;
use gc_memory_layout_common::layout::{ArrayMemoryLayout, FrameHeader, ObjectMemoryLayout, StackframeMemoryLayout};
use gc_memory_layout_common::memory_regions::{AllocatedObjectType, FramePointerOffset};
use jvmti_jni_bindings::{jlong};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::loading::LoaderName;

use crate::interpreter_state::InterpreterStateGuard;
use crate::jit::{CompiledCodeID, ToIR, ToNative};
use crate::jit::state::birangemap::BiRangeMap;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::runtime_class::{RuntimeClass};

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
        let mut assembler: CodeAssembler = CodeAssembler::new(64).unwrap();
        let iced_labels = ir_labels.into_iter().map(|label| (label.name, assembler.create_label())).collect::<HashMap<_, _>>();
        let label_instruction_offsets: Vec<(LabelName, u32)> = vec![];
        let mut instruction_index_to_bytecode_offset_start: HashMap<u32, (ByteCodeOffset, CInstruction)> = HashMap::new();
        for (bytecode_offset, ir_instr, cinstruction) in ir {
            let cinstruction: CInstruction = cinstruction;
            instruction_index_to_bytecode_offset_start.insert(assembler.instructions().len() as u32, (bytecode_offset, cinstruction));
            todo!()
        }
        todo!()
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

}

pub fn runtime_class_to_allocated_object_type(ref_type: &RuntimeClass, loader: LoaderName, arr_len: Option<usize>) -> AllocatedObjectType {
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
                        len: arr_len.unwrap() as i32,
                        sub_type_loader: loader,
                    };
                }
                RuntimeClass::Top => panic!(),
            };
            AllocatedObjectType::PrimitiveArray { primitive_type, len: arr_len.unwrap() as i32 }
        }
        RuntimeClass::Object(class_class) => AllocatedObjectType::Class {
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

pub fn setup_args_from_current_frame<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut InterpreterStateGuard<'gc, 'l>, desc: &CMethodDescriptor, is_virtual: bool) -> Vec<NewJavaValueHandle<'gc>> {
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