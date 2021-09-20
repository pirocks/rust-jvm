#![feature(asm)]
#![feature(thread_id_value)]

use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use std::mem::size_of;
use std::panic;
use std::panic::catch_unwind;
use std::process::exit;

use iced_x86::{BlockEncoder, InstructionBlock};
use iced_x86::BlockEncoderOptions;
use iced_x86::code_asm::{CodeAssembler, qword_ptr, r15, rax, rbp, rsp};
use memoffset::offset_of;

use gc_memory_layout_common::{FrameHeader, StackframeMemoryLayout};
use jit_common::java_stack::JavaStack;
use jit_common::JitCodeContext;
use jit_common::SavedRegisters;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::names::MethodName;

use crate::jit2::ir::{IRInstr, IRLabel, Register};
use crate::jit2::state::Labeler;

pub mod ir;
pub mod state;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct LabelName(u32);

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum VMExitType {
    ResolveInvokeStatic { method_name: MethodName, desc: CMethodDescriptor, target_class: CPDType },
    TopLevelReturn {},
}


#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct CompiledCodeID(pub u32);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct IRInstructionIndex(u32);

pub struct NativeInstructionLocation(*mut c_void);

#[derive(Clone, Debug)]
pub struct NotSupported;


#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ByteCodeOffset(u16);

pub struct ToIR {
    labels: Vec<IRLabel>,
    ir: Vec<(ByteCodeOffset, IRInstr)>,
}

pub fn to_ir(byte_code: Vec<&CInstruction>, labeler: &mut Labeler, layout: &dyn StackframeMemoryLayout) -> Result<ToIR, NotSupported> {
    let mut labels = vec![];
    let mut initial_ir = vec![];
    let function_start_label: LabelName = labeler.new_label(&mut labels);
    let function_end_label: LabelName = labeler.new_label(&mut labels);
    let mut pending_labels = vec![(ByteCodeOffset(0), function_start_label), (ByteCodeOffset(byte_code.last().unwrap().offset), function_end_label)];
    for byte_code_instr in byte_code {
        let current_offset = ByteCodeOffset(byte_code_instr.offset);
        match &byte_code_instr.info {
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                let exit_label = labeler.new_label(&mut labels);
                initial_ir.push((current_offset, IRInstr::VMExit { exit_label, exit_type: VMExitType::ResolveInvokeStatic { method_name: *method_name, desc: descriptor.clone(), target_class: CPDType::Ref(classname_ref_type.clone()) } }));
            }
            CompressedInstructionInfo::ifnull(offset) => {
                let branch_to_label = labeler.new_label(&mut labels);
                pending_labels.push((ByteCodeOffset((current_offset.0 as i32 + *offset as i32) as u16), branch_to_label));
                let possibly_null_register = Register(0);
                initial_ir.push((current_offset, IRInstr::LoadFPRelative {
                    from: layout.operand_stack_entry(current_offset.0, 0),
                    to: possibly_null_register,
                }));
                let register_with_null = Register(1);
                initial_ir.push((current_offset, IRInstr::Const64bit { to: register_with_null, const_: 0 }));
                initial_ir.push((current_offset, IRInstr::BranchEqual { a: register_with_null, b: possibly_null_register, label: branch_to_label }))
            }
            CompressedInstructionInfo::return_ => {
                initial_ir.push((current_offset, IRInstr::Return { return_val: None }));
            }
            _ => todo!()
        }
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
    })
}


pub struct ToNative {
    code: Vec<u8>,
    new_labels: HashMap<LabelName, *mut c_void>,
    exits: HashMap<LabelName, VMExitType>,
}

pub fn ir_to_native(ir: ToIR, base_address: *mut c_void) -> ToNative {
    let ToIR { labels: ir_labels, ir } = ir;
    let mut exits = HashMap::new();
    let mut assembler = CodeAssembler::new(64).unwrap();
    let iced_labels = ir_labels.into_iter().map(|label| (label, assembler.create_label())).collect::<HashMap<_, _>>();
    let mut label_instruction_offsets: Vec<(LabelName, u32)> = vec![];
    for (bytecode_offset, ir_instr) in ir {
        match ir_instr {
            IRInstr::LoadFPRelative { .. } => todo!(),
            IRInstr::StoreFPRelative { .. } => todo!(),
            IRInstr::Load { .. } => todo!(),
            IRInstr::Store { .. } => todo!(),
            IRInstr::Add { .. } => todo!(),
            IRInstr::Sub { .. } => todo!(),
            IRInstr::Div { .. } => todo!(),
            IRInstr::Mod { .. } => todo!(),
            IRInstr::Mul { .. } => todo!(),
            IRInstr::Const32bit { .. } => todo!(),
            IRInstr::Const64bit { .. } => todo!(),
            IRInstr::BranchToLabel { .. } => todo!(),
            IRInstr::BranchEqual { label, a, b } => {
                assembler.cmp(a.to_native_64(), b.to_native_64()).unwrap();
                assembler.je(iced_labels[&IRLabel { name: label }]).unwrap();
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
                let mut iced_label = iced_labels[&label];
                label_instruction_offsets.push((label.name, assembler.instructions().len() as u32));
                assembler.set_label(&mut iced_label).unwrap()// todo this could fail if two labels on same instruction which is likely to happen
            }
            IRInstr::Return { return_val } => {
                if let Some(return_register) = return_val {
                    assembler.mov(rax, return_register.to_native_64()).unwrap();
                }
                assembler.mov(rsp, rbp + size_of::<*mut c_void>()).unwrap();
                assert_eq!(offset_of!(FrameHeader,prev_rip), 0);
                assembler.mov(rbp, rbp + offset_of!(FrameHeader,prev_rpb)).unwrap();
                assembler.ret().unwrap();
            }
        }
    }
    let block = InstructionBlock::new(assembler.instructions(), base_address as u64);
    let result = BlockEncoder::encode(assembler.bitness(), block, BlockEncoderOptions::RETURN_NEW_INSTRUCTION_OFFSETS).unwrap();
    let mut new_labels: HashMap<LabelName, *mut c_void> = Default::default();
    let mut label_instruction_indexes = label_instruction_offsets.into_iter().peekable();
    for (i, native_offset) in result.new_instruction_offsets.iter().enumerate() {
        loop {
            match label_instruction_indexes.peek() {
                None => break,
                Some((label, instruction_index)) => {
                    assert!(i <= *instruction_index as usize);
                    if *instruction_index as usize == i {
                        new_labels.insert(*label, unsafe { base_address.offset(*native_offset as isize) });
                        let _ = label_instruction_indexes.next();
                        continue;
                    }
                }
            }
            break;
        }
    }
    assert!(label_instruction_indexes.peek().is_none());
    ToNative { code: result.code_buffer, new_labels, exits }
}


pub enum TransitionType {
    ResolveCalls
}

pub fn transition_stack_frame(transition_type: TransitionType, frame_to_fix: &mut JavaStack) {
    match transition_type {
        TransitionType::ResolveCalls => {}
    }
}


// recompile calling fn completely, but only switch over for new fn calls

unsafe extern "C" fn exit_handler() {
    panic!("not using exit handler for most stuff and just restoring to run_method")
}

