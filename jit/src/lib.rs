#![feature(asm)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_raw_ptr_deref)]
#![feature(box_syntax)]

extern crate compiler_builtins;

use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::transmute;
use std::num::NonZeroUsize;
use std::ptr::NonNull;

use iced_x86::{Code, Instruction, MemoryOperand, Register};
use iced_x86::ConditionCode::s;
use itertools::Either;
use memoffset::offset_of;
use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::{ArrayMemoryLayout, FramePointerOffset, StackframeMemoryLayout};
use jit_common::{JitCodeContext, SavedRegisters, VMExitData};
use jit_common::java_stack::JavaStack;
use jit_common::VMExitData::InvokeStaticResolveTarget;
use jit_ir::{ArithmeticType, Constant, InstructionSink, IRIndexToNative, IRInstruction, IRLabel, Size, VMExits};
use jvmti_jni_bindings::jvalue;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CInstructionInfo};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::arrays::{array_load, array_store};
use crate::integer_arithmetic::{binary_and, binary_or, binary_xor, integer_add, integer_div, integer_mul, integer_sub, shift, ShiftDirection};

pub const MAX_CODE_SIZE: usize = 2_000_000_000usize;
pub const CODE_LOCATION: usize = 0x100_000_000_000usize;

#[derive(Debug)]
pub struct VMExit {
    type_: VMExitData,
    start: NonNull<c_void>,
    return_to: Option<NonNull<c_void>>,
}

pub struct CompiledMethodTable {
    table_start: *mut c_void,
    method_id_to_location: HashMap<MethodId, NonNull<c_void>>,
    location_to_java_pc: HashMap<*mut c_void, u16>,
    exits: HashMap<*mut c_void, VMExit>,
    table_end: *mut c_void,
}

pub struct NotCompiled {}

impl CompiledMethodTable {
    pub fn new() -> Self {
        let prot_flags = ProtFlags::PROT_EXEC | ProtFlags::PROT_WRITE | ProtFlags::PROT_READ;
        let flags = MapFlags::MAP_ANONYMOUS | MapFlags::MAP_NORESERVE | MapFlags::MAP_PRIVATE;
        let mmap_addr = unsafe { mmap(transmute(CODE_LOCATION), MAX_CODE_SIZE, prot_flags, flags, -1, 0) }.unwrap();
        Self {
            table_start: mmap_addr,
            method_id_to_location: HashMap::new(),
            location_to_java_pc: HashMap::new(),
            exits: HashMap::new(),
            table_end: mmap_addr,
        }
    }

    pub fn run_method(&self, methodid: usize, stack: &mut JavaStack) -> Result<Either<Option<jvalue>, VMExitData>, NotCompiled> {
        match self.method_id_to_location.get(&methodid) {
            None => {
                Err(NotCompiled {})
            }
            Some(location) => {
                unsafe {
                    let as_ptr = *location;
                    let SavedRegisters { stack_pointer, frame_pointer, instruction_pointer: _, status_register } = stack.handle_vm_entry();
                    let rust_stack: u64 = stack_pointer as u64;
                    let rust_frame: u64 = frame_pointer as u64;
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
                            instruction_pointer: as_ptr.as_ptr(),
                            status_register,
                        },
                    };
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
                    // load context pointer into r15
                    // store old stack pointer into context
                    "mov [{0} + {old_stack_pointer_offset}],rsp",
                    // store old frame pointer into context
                    "mov [{0} + {old_frame_pointer_offset}],rbp",
                    // store exit instruction pointer into context
                    "lea r15, [rip+after_call]",
                    "mov [{0} + {old_rip_offset}],r15",
                    "mov r15,{0}",
                    // load java frame pointer
                    "mov rbp, [{0} + {new_frame_pointer_offset}]",
                    // load java stack pointer
                    "mov rsp, [{0} + {new_stack_pointer_offset}]",
                    // jump to jitted code
                    "jmp [{0} + {new_rip_offset}]",
                    //
                    "after_call:",
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
                    dbg!(jit_code_context.java_saved.instruction_pointer);
                    dbg!(jit_code_context.java_saved.frame_pointer);
                    dbg!(jit_code_context.java_saved.stack_pointer);
                    dbg!(jit_code_context.native_saved.instruction_pointer);
                    dbg!(jit_code_context.native_saved.stack_pointer);
                    dbg!(jit_code_context.native_saved.frame_pointer);
                    dbg!(&self.exits);
                    stack.saved_registers = Some(jit_code_context.java_saved.clone());
                    let vm_exit_type = self.exits.get(&jit_code_context.java_saved.instruction_pointer).unwrap().type_.clone();
                    match vm_exit_type {
                        VMExitData::CheckCast => todo!(),
                        VMExitData::InstanceOf => todo!(),
                        VMExitData::Throw => todo!(),
                        VMExitData::InvokeDynamic => todo!(),
                        VMExitData::InvokeStaticResolveTarget { method_name, descriptor, classname_ref_type, native_start, native_end } => return Ok(Either::Right(VMExitData::InvokeStaticResolveTarget { method_name, descriptor, classname_ref_type, native_start, native_end })),
                        VMExitData::InvokeVirtualResolveTarget { .. } => todo!(),
                        VMExitData::InvokeSpecialResolveTarget { .. } => todo!(),
                        VMExitData::InvokeInterfaceResolveTarget { .. } => todo!(),
                        VMExitData::MonitorEnter => todo!(),
                        VMExitData::MonitorExit => todo!(),
                        VMExitData::MultiNewArray => todo!(),
                        VMExitData::ArrayOutOfBounds => todo!(),
                        VMExitData::DebugTestExit => {
                            todo!()//we are in test and expected this
                        }
                        VMExitData::ExitDueToCompletion => {
                            todo!()
                        }
                        VMExitData::DebugTestExitValue { .. } => {
                            todo!()
                        }
                    }
                }
            }
        }
    }

    pub fn add_method(&mut self, method_id: usize, code: Vec<CInstruction>, memory_layout: &dyn StackframeMemoryLayout) {
        let JitIROutput { main_block: JitBlock { ir_to_java_pc, instructions } } = code_to_ir(code, memory_layout, self).unwrap();

        let mut instruction_sink = InstructionSink::new();
        for (ir_index, instruction) in instructions.into_iter().enumerate() {
            dbg!(&instruction);
            instruction.to_x86(ir_index, &mut instruction_sink)
        }
        self.add_from_instruction_sink(Some(method_id), &ir_to_java_pc, instruction_sink)
    }


    pub fn replace_exit(&mut self, start: *mut c_void, end: *mut c_void, replace_with: Vec<IRInstruction>) {
        let exit = self.exits.remove(&start).unwrap();
        let mut sink = InstructionSink::new();
        for instruct in replace_with {
            instruct.to_x86(todo!(), &mut sink);
        }
        let (exits, ir_to_native, size) = sink.fully_compiled(start, unsafe { start.offset_from(end) } as usize);
        let replaced_end = unsafe { start.offset(size as isize) };
        let remaining_branch = todo!();
    }

    fn add_from_instruction_sink(&mut self, method_id: Option<usize>, ir_to_java_pc: &HashMap<usize, u16>, instruction_sink: InstructionSink) {
        let table_used_size = self.table_end as usize - self.table_start as usize;
        let (
            VMExits {
                memory_offset_to_vm_exit,
                memory_offset_to_vm_return
            },
            IRIndexToNative {
                inner: ir_index_to_native
            },
            added_len
        ) = instruction_sink.fully_compiled(self.table_end, MAX_CODE_SIZE - (table_used_size));
        for (abs_offset, vm_exit_type) in memory_offset_to_vm_exit {
            let vm_exit = VMExit {
                type_: vm_exit_type,
                start: NonNull::new(abs_offset.0).unwrap(),
                return_to: memory_offset_to_vm_return[&abs_offset].map(|vm_return| NonNull::new(vm_return.0).unwrap()),
            };
            self.exits.insert(abs_offset.0, vm_exit);
        }
        for (ir_index, native) in ir_index_to_native {
            dbg!(ir_index);
            dbg!(&ir_to_java_pc);
            self.location_to_java_pc.insert(native, ir_to_java_pc[&ir_index]);
        }
        if let Some(method_id) = method_id {
            self.method_id_to_location.insert(method_id, NonNull::new(self.table_end).unwrap());
        }
        unsafe { self.table_end = self.table_end.offset(added_len as isize); }
    }
}

impl MethodLocationLookup for CompiledMethodTable {
    fn lookup(&self, method_id: MethodId) -> Option<NonNull<c_void>> {
        self.method_id_to_location.get(&method_id).cloned()
    }
}

#[derive(Debug)]
pub enum JITError {
    NotSupported
}


pub struct JitBlock {
    ir_to_java_pc: HashMap<usize, u16>,
    instructions: Vec<IRInstruction>,
}

impl JitBlock {
    pub fn add_instruction(&mut self, instruction: IRInstruction, java_pc: u16) {
        self.ir_to_java_pc.insert(self.instructions.len(), java_pc);
        self.instructions.push(instruction);//todo need to handle java_pc somehow
    }
}

pub struct JitIROutput {
    main_block: JitBlock,
}

impl JitIROutput {
    pub fn add_block(&mut self, block: JitBlock) {
        todo!()
    }
}

pub struct JitState<'l> {
    memory_layout: &'l dyn StackframeMemoryLayout,
    java_pc: u16,
    next_pc: Option<NonZeroUsize>,
    output: JitIROutput,
}

impl JitState<'_> {
    pub fn new_ir_label(&self) -> IRLabel {
        todo!()
    }

    pub fn next_pc(&self) -> u16 {
        self.next_pc.unwrap().get() as u16
    }
}

const MAX_INTERMEDIATE_VALUE_PADDING: usize = 3;

pub type MethodId = usize; //todo I should do something about this hack with multiple defs of this

pub trait MethodLocationLookup {
    fn lookup(&self, method_id: MethodId) -> Option<NonNull<c_void>>;
}

pub trait MethodLayoutLookup {
    fn lookup_layout(&self, method_id: MethodId) -> &dyn StackframeMemoryLayout;
}

pub fn code_to_ir(code: Vec<CInstruction>, memory_layout: &dyn StackframeMemoryLayout, lookup_method_location: &dyn MethodLocationLookup) -> Result<JitIROutput, JITError> {
    // let  = StackframeMemoryLayout::new((code.max_stack as usize + MAX_INTERMEDIATE_VALUE_PADDING) as usize, code.max_locals as usize, frame_vtypes);
    let mut jit_state = JitState {
        memory_layout,
        java_pc: 0,
        next_pc: None,
        output: JitIROutput { main_block: JitBlock { ir_to_java_pc: Default::default(), instructions: vec![] } },
    };
    let mut current_instr: Option<&CInstruction> = None;
    for future_instr in &code {
        if let Some(current_instr) = current_instr.take() {
            dbg!(&code);
            dbg!(future_instr);
            dbg!(current_instr);
            jit_state.next_pc = Some(NonZeroUsize::new(future_instr.offset as usize).unwrap());
            jit_state.java_pc = current_instr.offset;
            byte_code_to_ir(current_instr, &mut jit_state, lookup_method_location)?;
        }
        jit_state.next_pc = None;
        current_instr = Some(future_instr);
    }
    byte_code_to_ir(current_instr.unwrap(), &mut jit_state, lookup_method_location)?;
    Ok(jit_state.output)
}

pub fn byte_code_to_ir(bytecode: &CInstruction, current_jit_state: &mut JitState, lookup_method_location: &dyn MethodLocationLookup) -> Result<(), JITError> {
    let CInstruction { offset, instruction_size, info } = bytecode;
    current_jit_state.java_pc = *offset;
    let java_pc = current_jit_state.java_pc as u16;
    match info {
        CInstructionInfo::aaload => {
            array_load(current_jit_state, Size::Long)
        }
        CInstructionInfo::aastore => {
            array_store(current_jit_state, Size::Long)
        }
        CInstructionInfo::aconst_null => {
            constant(current_jit_state, Constant::Pointer(0))
        }
        CInstructionInfo::aload(variable_index) => {
            aload_n(current_jit_state, *variable_index as usize)
        }
        CInstructionInfo::aload_0 => {
            aload_n(current_jit_state, 0)
        }
        CInstructionInfo::aload_1 => {
            aload_n(current_jit_state, 1)
        }
        CInstructionInfo::aload_2 => {
            aload_n(current_jit_state, 2)
        }
        CInstructionInfo::aload_3 => {
            aload_n(current_jit_state, 3)
        }
        CInstructionInfo::anewarray(_) => Err(JITError::NotSupported),
        CInstructionInfo::areturn => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::Return {
                return_value: current_jit_state.memory_layout.operand_stack_entry(java_pc as u16, 0),
                return_value_size: Size::Long,
            }, current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::arraylength => {
            let layout: ArrayMemoryLayout = todo!();
            layout.len_entry();
            todo!();
            Ok(())
        }
        CInstructionInfo::astore(variable_index) => {
            astore_n(current_jit_state, *variable_index as u16)
        }
        CInstructionInfo::astore_0 => {
            astore_n(current_jit_state, 0)
        }
        CInstructionInfo::astore_1 => {
            astore_n(current_jit_state, 1)
        }
        CInstructionInfo::astore_2 => {
            astore_n(current_jit_state, 2)
        }
        CInstructionInfo::astore_3 => {
            astore_n(current_jit_state, 3)
        }
        CInstructionInfo::athrow => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::Throw), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::baload => {
            array_load(current_jit_state, Size::Byte)
        }
        CInstructionInfo::bastore => {
            array_store(current_jit_state, Size::Byte)
        }
        CInstructionInfo::bipush(_) => Err(JITError::NotSupported),
        CInstructionInfo::caload => {
            array_load(current_jit_state, Size::Short)
        }
        CInstructionInfo::castore => { Err(JITError::NotSupported) }
        CInstructionInfo::checkcast(_) => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::CheckCast), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::d2f => Err(JITError::NotSupported),
        CInstructionInfo::d2i => Err(JITError::NotSupported),
        CInstructionInfo::d2l => Err(JITError::NotSupported),
        CInstructionInfo::dadd => Err(JITError::NotSupported),
        CInstructionInfo::daload => {
            array_load(current_jit_state, Size::Long)
        }
        CInstructionInfo::dastore => {
            array_store(current_jit_state, Size::Long)
        }
        CInstructionInfo::dcmpg => Err(JITError::NotSupported),
        CInstructionInfo::dcmpl => Err(JITError::NotSupported),
        CInstructionInfo::dconst_0 => {
            constant(current_jit_state, Constant::Double(0f64))
        }
        CInstructionInfo::dconst_1 => {
            constant(current_jit_state, Constant::Double(1f64))
        }
        CInstructionInfo::ddiv => Err(JITError::NotSupported),
        CInstructionInfo::dload(n) => {
            store_n(current_jit_state, *n as u16, Size::Long)
        }
        CInstructionInfo::dload_0 => {
            store_n(current_jit_state, 0, Size::Long)
        }
        CInstructionInfo::dload_1 => {
            store_n(current_jit_state, 1, Size::Long)
        }
        CInstructionInfo::dload_2 => {
            store_n(current_jit_state, 2, Size::Long)
        }
        CInstructionInfo::dload_3 => {
            store_n(current_jit_state, 3, Size::Long)
        }
        CInstructionInfo::dmul => Err(JITError::NotSupported),
        CInstructionInfo::dneg => Err(JITError::NotSupported),
        CInstructionInfo::drem => Err(JITError::NotSupported),
        CInstructionInfo::dreturn => Err(JITError::NotSupported),
        CInstructionInfo::dstore(n) => {
            store_n(current_jit_state, *n as u16, Size::Long)
        }
        CInstructionInfo::dstore_0 => {
            store_n(current_jit_state, 0, Size::Long)
        }
        CInstructionInfo::dstore_1 => {
            store_n(current_jit_state, 1, Size::Long)
        }
        CInstructionInfo::dstore_2 => {
            store_n(current_jit_state, 2, Size::Long)
        }
        CInstructionInfo::dstore_3 => {
            store_n(current_jit_state, 3, Size::Long)
        }
        CInstructionInfo::dsub => Err(JITError::NotSupported),
        CInstructionInfo::dup => Err(JITError::NotSupported),
        CInstructionInfo::dup_x1 => Err(JITError::NotSupported),
        CInstructionInfo::dup_x2 => Err(JITError::NotSupported),
        CInstructionInfo::dup2 => Err(JITError::NotSupported),
        CInstructionInfo::dup2_x1 => Err(JITError::NotSupported),
        CInstructionInfo::dup2_x2 => Err(JITError::NotSupported),
        CInstructionInfo::f2d => Err(JITError::NotSupported),
        CInstructionInfo::f2i => Err(JITError::NotSupported),
        CInstructionInfo::f2l => Err(JITError::NotSupported),
        CInstructionInfo::fadd => Err(JITError::NotSupported),
        CInstructionInfo::faload => {
            array_load(current_jit_state, Size::Int)
        }
        CInstructionInfo::fastore => {
            array_store(current_jit_state, Size::Int)
        }
        CInstructionInfo::fcmpg => Err(JITError::NotSupported),
        CInstructionInfo::fcmpl => Err(JITError::NotSupported),
        CInstructionInfo::fconst_0 => {
            constant(current_jit_state, Constant::Float(0.0f32))
        }
        CInstructionInfo::fconst_1 => {
            constant(current_jit_state, Constant::Float(1.0f32))
        }
        CInstructionInfo::fconst_2 => {
            constant(current_jit_state, Constant::Float(2.0f32))
        }
        CInstructionInfo::fdiv => Err(JITError::NotSupported),
        CInstructionInfo::fload(n) => {
            load_n(current_jit_state, *n as usize, Size::Int)
        }
        CInstructionInfo::fload_0 => {
            load_n(current_jit_state, 0, Size::Int)
        }
        CInstructionInfo::fload_1 => {
            load_n(current_jit_state, 1, Size::Int)
        }
        CInstructionInfo::fload_2 => {
            load_n(current_jit_state, 2, Size::Int)
        }
        CInstructionInfo::fload_3 => {
            load_n(current_jit_state, 3, Size::Int)
        }
        CInstructionInfo::fmul => Err(JITError::NotSupported),
        CInstructionInfo::fneg => Err(JITError::NotSupported),
        CInstructionInfo::frem => Err(JITError::NotSupported),
        CInstructionInfo::freturn => Err(JITError::NotSupported),
        CInstructionInfo::fstore(n) => {
            store_n(current_jit_state, *n as u16, Size::Int)
        }
        CInstructionInfo::fstore_0 => {
            store_n(current_jit_state, 0, Size::Int)
        }
        CInstructionInfo::fstore_1 => {
            store_n(current_jit_state, 1, Size::Int)
        }
        CInstructionInfo::fstore_2 => {
            store_n(current_jit_state, 2, Size::Int)
        }
        CInstructionInfo::fstore_3 => {
            store_n(current_jit_state, 3, Size::Int)
        }
        CInstructionInfo::fsub => Err(JITError::NotSupported),
        CInstructionInfo::getfield { name, desc, target_class } => Err(JITError::NotSupported),
        CInstructionInfo::getstatic { name, desc, target_class } => Err(JITError::NotSupported),
        CInstructionInfo::goto_(_) => Err(JITError::NotSupported),
        CInstructionInfo::goto_w(_) => Err(JITError::NotSupported),
        CInstructionInfo::i2b => Err(JITError::NotSupported),
        CInstructionInfo::i2c => Err(JITError::NotSupported),
        CInstructionInfo::i2d => Err(JITError::NotSupported),
        CInstructionInfo::i2f => Err(JITError::NotSupported),
        CInstructionInfo::i2l => Err(JITError::NotSupported),
        CInstructionInfo::i2s => Err(JITError::NotSupported),
        CInstructionInfo::iadd => {
            integer_add(current_jit_state, Size::Int)
        }
        CInstructionInfo::iaload => {
            array_load(current_jit_state, Size::Int)
        }
        CInstructionInfo::iand => {
            binary_and(current_jit_state, Size::Int)
        }
        CInstructionInfo::iastore => {
            array_store(current_jit_state, Size::Int)
        }
        CInstructionInfo::iconst_m1 => {
            constant(current_jit_state, Constant::Int(-1))
        }
        CInstructionInfo::iconst_0 => {
            constant(current_jit_state, Constant::Int(0))
        }
        CInstructionInfo::iconst_1 => {
            constant(current_jit_state, Constant::Int(1))
        }
        CInstructionInfo::iconst_2 => {
            constant(current_jit_state, Constant::Int(2))
        }
        CInstructionInfo::iconst_3 => {
            constant(current_jit_state, Constant::Int(3))
        }
        CInstructionInfo::iconst_4 => {
            constant(current_jit_state, Constant::Int(4))
        }
        CInstructionInfo::iconst_5 => {
            constant(current_jit_state, Constant::Int(5))
        }
        CInstructionInfo::idiv => {
            integer_div(current_jit_state, Size::Int)
        }
        CInstructionInfo::if_acmpeq(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_acmpne(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_icmpeq(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_icmpne(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_icmplt(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_icmpge(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_icmpgt(_) => Err(JITError::NotSupported),
        CInstructionInfo::if_icmple(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifeq(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifne(_) => Err(JITError::NotSupported),
        CInstructionInfo::iflt(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifge(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifgt(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifle(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifnonnull(_) => Err(JITError::NotSupported),
        CInstructionInfo::ifnull(_) => Err(JITError::NotSupported),
        CInstructionInfo::iinc(_) => Err(JITError::NotSupported),
        CInstructionInfo::iload(n) => {
            load_n(current_jit_state, *n as usize, Size::Int)
        }
        CInstructionInfo::iload_0 => {
            load_n(current_jit_state, 0, Size::Int)
        }
        CInstructionInfo::iload_1 => {
            load_n(current_jit_state, 1, Size::Int)
        }
        CInstructionInfo::iload_2 => {
            load_n(current_jit_state, 2, Size::Int)
        }
        CInstructionInfo::iload_3 => {
            load_n(current_jit_state, 3, Size::Int)
        }
        CInstructionInfo::imul => {
            let instruct = IRInstruction::IntegerArithmetic {
                input_offset_a: current_jit_state.memory_layout.operand_stack_entry(java_pc as u16, 1),
                input_offset_b: current_jit_state.memory_layout.operand_stack_entry(java_pc as u16, 0),
                output_offset: current_jit_state.memory_layout.operand_stack_entry(java_pc as u16, 1),
                size: Size::Int,
                signed: true,
                arithmetic_type: ArithmeticType::Mul,
            };
            current_jit_state.output.main_block.add_instruction(instruct, current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::ineg => {
            todo!()
        }
        CInstructionInfo::instanceof(_) => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::InstanceOf), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::invokedynamic(_) => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::InvokeDynamic), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::invokeinterface { method_name, descriptor, classname_ref_type, count } => {
            let resolved_function_location = current_jit_state.memory_layout.safe_temp_location(java_pc as u16, 0);
            let local_var_and_operand_stack_size_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 1);
            let exit_to_get_target = IRInstruction::VMExit(VMExitData::InvokeInterfaceResolveTarget {});
            current_jit_state.output.main_block.add_instruction(exit_to_get_target, current_jit_state.java_pc);
            let call = IRInstruction::Call { resolved_destination_rel: todo!(), local_var_and_operand_stack_size: local_var_and_operand_stack_size_location, return_location: todo!() };
            current_jit_state.output.main_block.add_instruction(call, current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::InvokeSpecialResolveTarget {}), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
            let CMethodDescriptor { arg_types, return_type } = descriptor;
            //todo need to get memory layout for next
            for (i, arg) in arg_types.iter().rev().enumerate() {
                let size = match arg {
                    CompressedParsedDescriptorType::BooleanType => Size::Byte,
                    CompressedParsedDescriptorType::ByteType => Size::Byte,
                    CompressedParsedDescriptorType::ShortType => Size::Short,
                    CompressedParsedDescriptorType::CharType => Size::Short,
                    CompressedParsedDescriptorType::IntType => Size::Int,
                    CompressedParsedDescriptorType::LongType => Size::Long,
                    CompressedParsedDescriptorType::FloatType => Size::Int,
                    CompressedParsedDescriptorType::DoubleType => Size::Long,
                    CompressedParsedDescriptorType::VoidType => panic!(),
                    CompressedParsedDescriptorType::Ref(_) => Size::Long
                };
                current_jit_state.output.main_block.add_instruction(IRInstruction::CopyRelative {
                    input_offset: current_jit_state.memory_layout.operand_stack_entry(java_pc, 0),
                    output_offset: FramePointerOffset(current_jit_state.memory_layout.full_frame_size() + todo!() as usize),
                    input_size: size,
                    output_size: size,
                    signed: false,
                }, current_jit_state.java_pc);
            }
            assert_eq!(return_type, &CPDType::VoidType);
            // let call_location = lookup_method_location.lookup(method_name, descriptor);
            // match call_location{
            //     None => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::InvokeStaticResolveTarget {
                method_name: *method_name,
                descriptor: descriptor.clone(),
                classname_ref_type: classname_ref_type.clone(),
                native_start: todo!(),
                native_end: todo!(),
            }), current_jit_state.java_pc);
            // }
            // Some(call_location) => {
            //     let actual_call = IRInstruction::Call {
            //         resolved_destination_rel: call_location,
            //         local_var_and_operand_stack_size: FramePointerOffset(todo!()),
            //         return_location: None,
            //     };
            //     current_jit_state.output.main_block.add_instruction(actual_call, current_jit_state.java_pc);
            // }
            // }


            Ok(())
        }
        CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
            //todo handle if resolved
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::InvokeVirtualResolveTarget {}), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::ior => {
            binary_or(current_jit_state, Size::Int)
        }
        CInstructionInfo::irem => Err(JITError::NotSupported),
        CInstructionInfo::ireturn => Err(JITError::NotSupported),
        CInstructionInfo::ishl => {
            shift(current_jit_state, java_pc, Size::Int, ShiftDirection::ArithmeticLeft)
        }
        CInstructionInfo::ishr => {
            shift(current_jit_state, java_pc, Size::Int, ShiftDirection::ArithmeticRight)
        }
        CInstructionInfo::istore(n) => {
            store_n(current_jit_state, *n as u16, Size::Int)
        }
        CInstructionInfo::istore_0 => {
            store_n(current_jit_state, 0, Size::Int)
        }
        CInstructionInfo::istore_1 => {
            store_n(current_jit_state, 1, Size::Int)
        }
        CInstructionInfo::istore_2 => {
            store_n(current_jit_state, 2, Size::Int)
        }
        CInstructionInfo::istore_3 => {
            store_n(current_jit_state, 3, Size::Int)
        }
        CInstructionInfo::isub => {
            integer_sub(current_jit_state, Size::Int)
        }
        CInstructionInfo::iushr => {
            shift(current_jit_state, java_pc, Size::Int, ShiftDirection::LogicalRight)
        }
        CInstructionInfo::ixor => {
            binary_xor(current_jit_state, Size::Int)
        }
        CInstructionInfo::jsr(_) => Err(JITError::NotSupported),
        CInstructionInfo::jsr_w(_) => Err(JITError::NotSupported),
        CInstructionInfo::l2d => Err(JITError::NotSupported),
        CInstructionInfo::l2f => Err(JITError::NotSupported),
        CInstructionInfo::l2i => Err(JITError::NotSupported),
        CInstructionInfo::ladd => {
            integer_add(current_jit_state, Size::Long)
        }
        CInstructionInfo::laload => {
            array_load(current_jit_state, Size::Long)
        }
        CInstructionInfo::land => {
            binary_and(current_jit_state, Size::Long)
        }
        CInstructionInfo::lastore => {
            array_store(current_jit_state, Size::Long)
        }
        CInstructionInfo::lcmp => Err(JITError::NotSupported),
        CInstructionInfo::lconst_0 => {
            constant(current_jit_state, Constant::Long(0))
        }
        CInstructionInfo::lconst_1 => {
            constant(current_jit_state, Constant::Long(1))
        }
        CInstructionInfo::ldc(_) => Err(JITError::NotSupported),
        CInstructionInfo::ldc_w(_) => Err(JITError::NotSupported),
        CInstructionInfo::ldc2_w(_) => Err(JITError::NotSupported),
        CInstructionInfo::ldiv => {
            integer_div(current_jit_state, Size::Long)
        }
        CInstructionInfo::lload(n) => {
            load_n(current_jit_state, *n as usize, Size::Long)
        }
        CInstructionInfo::lload_0 => {
            load_n(current_jit_state, 0, Size::Long)
        }
        CInstructionInfo::lload_1 => {
            load_n(current_jit_state, 1, Size::Long)
        }
        CInstructionInfo::lload_2 => {
            load_n(current_jit_state, 2, Size::Long)
        }
        CInstructionInfo::lload_3 => {
            load_n(current_jit_state, 3, Size::Long)
        }
        CInstructionInfo::lmul => {
            integer_mul(current_jit_state, Size::Long)
        }
        CInstructionInfo::lneg => Err(JITError::NotSupported),
        CInstructionInfo::lookupswitch(_) => Err(JITError::NotSupported),
        CInstructionInfo::lor => {
            binary_or(current_jit_state, Size::Long)
        }
        CInstructionInfo::lrem => Err(JITError::NotSupported),
        CInstructionInfo::lreturn => Err(JITError::NotSupported),
        CInstructionInfo::lshl => {
            shift(current_jit_state, java_pc, Size::Long, ShiftDirection::ArithmeticLeft)
        }
        CInstructionInfo::lshr => {
            shift(current_jit_state, java_pc, Size::Long, ShiftDirection::ArithmeticRight)
        }
        CInstructionInfo::lstore(n) => {
            store_n(current_jit_state, *n as u16, Size::Long)
        }
        CInstructionInfo::lstore_0 => {
            store_n(current_jit_state, 0, Size::Long)
        }
        CInstructionInfo::lstore_1 => {
            store_n(current_jit_state, 1, Size::Long)
        }
        CInstructionInfo::lstore_2 => {
            store_n(current_jit_state, 2, Size::Long)
        }
        CInstructionInfo::lstore_3 => {
            store_n(current_jit_state, 3, Size::Long)
        }
        CInstructionInfo::lsub => {
            integer_sub(current_jit_state, Size::Long)
        }
        CInstructionInfo::lushr => {
            shift(current_jit_state, java_pc, Size::Long, ShiftDirection::LogicalRight)
        }
        CInstructionInfo::lxor => {
            binary_xor(current_jit_state, Size::Long)
        }
        CInstructionInfo::monitorenter => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::MonitorEnter), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::monitorexit => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::MonitorExit), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::multianewarray { type_, dimensions } => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitData::MultiNewArray), current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::new(_) => Err(JITError::NotSupported),
        CInstructionInfo::newarray(_) => Err(JITError::NotSupported),
        CInstructionInfo::nop => {
            Ok(())
        }
        CInstructionInfo::pop => {
            Ok(())
        }
        CInstructionInfo::pop2 => {
            Ok(())
        }
        CInstructionInfo::putfield { name, desc, target_class } => Err(JITError::NotSupported),
        CInstructionInfo::putstatic { name, desc, target_class } => Err(JITError::NotSupported),
        CInstructionInfo::ret(_) => Err(JITError::NotSupported),
        CInstructionInfo::return_ => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::ReturnNone, current_jit_state.java_pc);
            Ok(())
        }
        CInstructionInfo::saload => {
            array_load(current_jit_state, Size::Short)
        }
        CInstructionInfo::sastore => {
            array_store(current_jit_state, Size::Short)
        }
        CInstructionInfo::sipush(_) => Err(JITError::NotSupported),
        CInstructionInfo::swap => {
            swap(current_jit_state)
        }
        CInstructionInfo::tableswitch(_) => Err(JITError::NotSupported),
        CInstructionInfo::wide(_) => Err(JITError::NotSupported),
        CInstructionInfo::EndOfCode => Err(JITError::NotSupported),
    }
}

fn swap(current_jit_state: &mut JitState) -> Result<(), JITError> {
    let a = current_jit_state.memory_layout.operand_stack_entry(current_jit_state.java_pc as u16, 0);
    let b = current_jit_state.memory_layout.operand_stack_entry(current_jit_state.java_pc as u16, 1);
    let temp = current_jit_state.memory_layout.safe_temp_location(current_jit_state.java_pc as u16, 0);
    let copy_to_temp = IRInstruction::CopyRelative {
        input_offset: a,
        output_offset: temp,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    current_jit_state.output.main_block.add_instruction(copy_to_temp, current_jit_state.java_pc);
    let b_to_a = IRInstruction::CopyRelative {
        input_offset: b,
        output_offset: a,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    current_jit_state.output.main_block.add_instruction(b_to_a, current_jit_state.java_pc);
    let temp_to_b = IRInstruction::CopyRelative {
        input_offset: temp,
        output_offset: b,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    current_jit_state.output.main_block.add_instruction(temp_to_b, current_jit_state.java_pc);
    Ok(())
}

pub mod arrays;
pub mod integer_arithmetic;

fn constant(current_jit_state: &mut JitState, constant: Constant) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc, next_pc } = current_jit_state;
    let null_offset = memory_layout.operand_stack_entry(next_pc.unwrap().get() as u16, 0);
    current_jit_state.output.main_block.add_instruction(IRInstruction::Constant {
        output_offset: null_offset,
        constant,
    }, current_jit_state.java_pc);
    Ok(())
}

fn aload_n(current_jit_state: &mut JitState, variable_index: usize) -> Result<(), JITError> {
    load_n(current_jit_state, variable_index, Size::Long)
}

fn load_n(current_jit_state: &mut JitState, variable_index: usize, size: Size) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc, next_pc } = current_jit_state;
    let local_var_offset = memory_layout.local_var_entry(*java_pc as u16, variable_index as u16);
    current_jit_state.output.main_block.add_instruction(IRInstruction::CopyRelative {
        input_offset: local_var_offset,
        output_offset: memory_layout.operand_stack_entry(next_pc.unwrap().get() as u16, 0),
        input_size: size,
        output_size: size,
        signed: false,
    }, current_jit_state.java_pc);
    Ok(())
}

fn astore_n(current_jit_state: &mut JitState, variable_index: u16) -> Result<(), JITError> {
    store_n(current_jit_state, variable_index, Size::Long)
}

//todo these should all return not mutate
fn store_n(current_jit_state: &mut JitState, variable_index: u16, size: Size) -> Result<(), JITError> {
    let JitState { memory_layout, output, java_pc, next_pc } = current_jit_state;
    let local_var_offset = memory_layout.local_var_entry(*java_pc as u16, variable_index);
    current_jit_state.output.main_block.add_instruction(IRInstruction::CopyRelative {
        input_offset: memory_layout.operand_stack_entry(*java_pc, 0),
        output_offset: local_var_offset,
        input_size: size,
        output_size: size,
        signed: false,
    }, current_jit_state.java_pc);
    Ok(())
}

pub mod native;

pub mod compiled_methods;