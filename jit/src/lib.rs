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
use std::sync::RwLock;

use nix::sys::mman::{MapFlags, mmap, ProtFlags};

use gc_memory_layout_common::{ArrayMemoryLayout, StackframeMemoryLayout};
use jit_common::VMExitType;
use jit_ir::{ArithmeticType, Constant, InstructionSink, IRIndexToNative, IRInstruction, IRLabel, Size, VMExits};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CInstructionInfo};

use crate::arrays::{array_load, array_store};
use crate::integer_arithmetic::{binary_and, binary_or, binary_xor, integer_add, integer_div, integer_mul, integer_sub, shift, ShiftDirection};

pub const MAX_CODE_SIZE: usize = 2_000_000_000usize;
pub const CODE_LOCATION: usize = 0x1_000_000_000_000usize;

pub struct VMExit {
    type_: VMExitType,
    start: NonNull<c_void>,
    return_to: Option<NonNull<c_void>>,
}

pub struct CompiledMethodTable {
    table_start: *mut c_void,
    method_id_to_location: HashMap<usize, *mut c_void>,
    location_to_java_pc: HashMap<*mut c_void, u16>,
    exits: HashMap<*mut c_void, VMExit>,
    table_end: *mut c_void,
}

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

    pub fn add_method(&mut self, method_id: usize, code: Vec<CInstruction>, memory_layout: &dyn StackframeMemoryLayout) {
        let JitIROutput { main_block: JitBlock { ir_to_java_pc, instructions } } = code_to_ir(code, memory_layout).unwrap();

        let mut instruction_sink = InstructionSink::new();
        for instruction in instructions {
            instruction.to_x86(&mut instruction_sink)
        }
        let table_used_size = self.table_end as usize - self.table_start as usize;
        let (VMExits { memory_offset_to_vm_exit, memory_offset_to_vm_return }, IRIndexToNative { inner: ir_index_to_native }, added_len) = instruction_sink.fully_compiled(self.table_end, MAX_CODE_SIZE - (table_used_size));
        for (abs_offset, vm_exit_type) in memory_offset_to_vm_exit {
            let vm_exit = VMExit {
                type_: vm_exit_type,
                start: NonNull::new(abs_offset.0).unwrap(),
                return_to: memory_offset_to_vm_return[&abs_offset].map(|vm_return| NonNull::new(vm_return.0).unwrap()),
            };
            self.exits.insert(abs_offset.0, vm_exit);
        }
        for (ir_index, native) in ir_index_to_native {
            self.location_to_java_pc.insert(native, ir_to_java_pc[&ir_index]);
        }
        unsafe { self.table_end = self.table_end.offset(added_len as isize); }
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
    pub fn add_instruction(&mut self, instruction: IRInstruction) {
        self.instructions.push(instruction);//todo need to handle java_pc somehow
    }
}

pub struct JitIROutput {
    main_block: JitBlock
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

pub fn code_to_ir(code: Vec<CInstruction>, memory_layout: &dyn StackframeMemoryLayout) -> Result<JitIROutput, JITError> {
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
            jit_state.next_pc = Some(NonZeroUsize::new(future_instr.offset as usize).unwrap());
            jit_state.java_pc = current_instr.offset;
            byte_code_to_ir(current_instr, &mut jit_state)?;
        }
        jit_state.next_pc = None;
        current_instr = Some(future_instr);
    }
    byte_code_to_ir(current_instr.unwrap(), &mut jit_state)?;
    Ok(jit_state.output)
}

pub fn byte_code_to_ir(bytecode: &CInstruction, current_jit_state: &mut JitState) -> Result<(), JITError> {
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
            });
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
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::Throw));
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
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::CheckCast));
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
            current_jit_state.output.main_block.add_instruction(instruct);
            Ok(())
        }
        CInstructionInfo::ineg => {
            todo!()
        }
        CInstructionInfo::instanceof(_) => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::InstanceOf));
            Ok(())
        }
        CInstructionInfo::invokedynamic(_) => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeDynamic));
            Ok(())
        }
        CInstructionInfo::invokeinterface { method_name, descriptor, classname_ref_type, count } => {
            let resolved_function_location = current_jit_state.memory_layout.safe_temp_location(java_pc as u16, 0);
            let local_var_and_operand_stack_size_location = current_jit_state.memory_layout.safe_temp_location(java_pc, 1);
            let exit_to_get_target = IRInstruction::VMExit(VMExitType::InvokeInterfaceResolveTarget { resolved: resolved_function_location });
            current_jit_state.output.main_block.add_instruction(exit_to_get_target);
            let call = IRInstruction::Call { resolved_destination: resolved_function_location, local_var_and_operand_stack_size: local_var_and_operand_stack_size_location, return_location: todo!() };
            current_jit_state.output.main_block.add_instruction(call);
            Ok(())
        }
        CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeSpecialResolveTarget { resolved: todo!() }));
            Ok(())
        }
        CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeStaticResolveTarget { resolved: todo!() }));
            Ok(())
        }
        CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::InvokeVirtualResolveTarget { resolved: todo!() }));
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
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::MonitorEnter));
            Ok(())
        }
        CInstructionInfo::monitorexit => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::MonitorExit));
            Ok(())
        }
        CInstructionInfo::multianewarray { type_, dimensions } => {
            current_jit_state.output.main_block.add_instruction(IRInstruction::VMExit(VMExitType::MultiNewArray));
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
            current_jit_state.output.main_block.add_instruction(IRInstruction::ReturnNone);
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
    current_jit_state.output.main_block.add_instruction(copy_to_temp);
    let b_to_a = IRInstruction::CopyRelative {
        input_offset: b,
        output_offset: a,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    current_jit_state.output.main_block.add_instruction(b_to_a);
    let temp_to_b = IRInstruction::CopyRelative {
        input_offset: temp,
        output_offset: b,
        input_size: Size::Int,
        output_size: Size::Int,
        signed: false,
    };
    current_jit_state.output.main_block.add_instruction(temp_to_b);
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
    });
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
    });
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
    });
    Ok(())
}

pub mod native;

pub mod compiled_methods;