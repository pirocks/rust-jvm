use std::collections::HashMap;
use std::mem::size_of;

use itertools::Itertools;

use rust_jvm_common::compressed_classfile::code::{CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use crate::jit::{ByteCodeOffset, MethodResolver};
use crate::jit::ir::{IRInstr, Register};
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::JVMState;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{FRAME_HEADER_END_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET};
use crate::stack_entry::FrameView;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ByteCodeIndex(u16);

// all metadata needed to compile to ir, excluding resolver stuff
pub struct JavaCompilerMethodAndFrameData {
    max_locals: u16,
    max_stack: u16,
    stack_depth_by_index: Vec<u16>,
    code_by_index: Vec<CompressedInstruction>,
}

impl JavaCompilerMethodAndFrameData {
    pub fn new(jvm: &'vm_life JVMState<'vm_life>, method_id: MethodId) -> Self {
        let function_frame_type_guard = jvm.function_frame_type_data.read().unwrap();
        let frames = function_frame_type_guard.get(&method_id).unwrap();
        let stack_depth = frames.iter().sorted_by_key(|(offset, _)| *offset).enumerate().map(|(i, (_offset, frame))| frame.stack_map.len() as u16).collect();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        Self {
            max_locals: code.max_locals,
            max_stack: code.max_stack,
            stack_depth_by_index: stack_depth,
            code_by_index: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).map(|(_, instr)| instr.clone()).collect(),
        }
    }

    pub fn lookup_operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
        FramePointerOffset(FRAME_HEADER_END_OFFSET + (self.max_locals + self.stack_depth_by_index[index.0 as usize] - from_end) as usize * size_of::<u64>())
    }

    pub fn local_var_entry(&self, index: ByteCodeIndex, local_var_index: u16) -> FramePointerOffset {
        assert!(local_var_index <= self.max_locals);
        FramePointerOffset(FRAME_HEADER_END_OFFSET + local_var_index as usize * size_of::<u64>())
    }

    pub fn full_frame_size(&self) -> usize {
        FRAME_HEADER_END_OFFSET + (self.max_locals + self.max_stack) as usize * size_of::<u64>()
    }
}

pub struct CurrentInstructionCompilerData {
    current_index: ByteCodeIndex,
    next_index: ByteCodeIndex,
}

pub fn compile_to_ir(resolver: &MethodResolver<'vm_life>, labeler: &Labeler, method_frame_data: &JavaCompilerMethodAndFrameData) -> Vec<IRInstr> {
    let cinstructions: &[CompressedInstruction] = method_frame_data.code_by_index.as_slice();
    let mut initial_ir: Vec<IRInstr> = vec![];
    let mut labels = vec![];
    for (i, compressed_instruction) in cinstructions.iter().enumerate() {
        let current_offset = ByteCodeOffset(compressed_instruction.offset);
        let current_index = ByteCodeIndex(i as u16);
        let next_index = ByteCodeIndex((i + 1) as u16);
        let current_instr_data = CurrentInstructionCompilerData {
            current_index,
            next_index,
        };
        match &compressed_instruction.info {
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                match resolver.lookup_static(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
                    None => {
                        let exit_label = labeler.new_label(&mut labels);
                        initial_ir.push(
                            IRInstr::VMExit {
                                exit_label,
                                exit_type: todo!()/*VMExitType::ResolveInvokeStatic {
                                method_name: *method_name,
                                desc: descriptor.clone(),
                                target_class: CPDType::Ref(classname_ref_type.clone()),
                            }*/,
                            },
                        );
                    }
                    Some((method_id, is_native)) => {
                        if is_native {
                            let exit_label = labeler.new_label(&mut labels);
                            initial_ir.push(
                                IRInstr::VMExit {
                                    exit_label,
                                    exit_type: todo!()/*VMExitType::RunNativeStatic {
                                    method_name: *method_name,
                                    desc: descriptor.clone(),
                                    target_class: CPDType::Ref(classname_ref_type.clone()),
                                }*/,
                                },
                            );
                        } else {
                            todo!()
                        }
                    }
                }
            }
            CompressedInstructionInfo::return_ => {
                initial_ir.push(IRInstr::Return {
                    return_val: None,
                    temp_register_1: Register(1),
                    temp_register_2: Register(2),
                    temp_register_3: Register(3),
                    temp_register_4: Register(4),
                    frame_size: method_frame_data.full_frame_size(),
                });
            }
            CompressedInstructionInfo::aload_0 => {
                initial_ir.extend(aload_n(method_frame_data, &current_instr_data, 0));
            }
            CompressedInstructionInfo::aload_1 => {
                initial_ir.extend(aload_n(method_frame_data, &current_instr_data, 1));
            }
            CompressedInstructionInfo::aload_2 => {
                initial_ir.extend(aload_n(method_frame_data, &current_instr_data, 2));
            }
            CompressedInstructionInfo::aload_3 => {
                initial_ir.extend(aload_n(method_frame_data, &current_instr_data, 3));
            }
            CompressedInstructionInfo::aload(n) => {
                initial_ir.extend(aload_n(method_frame_data, &current_instr_data, *n as u16));
            }
            CompressedInstructionInfo::if_acmpne(offset) => {}
            other => {
                dbg!(other);
                todo!()
            }
        }
    }
    initial_ir.into_iter().collect_vec()
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ReferenceEqualityType {
    NE,
    EQ,
}

pub fn if_acmp(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, ref_equality: ReferenceEqualityType, bytecode_offset: ByteCodeOffset) -> impl Iterator<Item=IRInstr> {}


pub fn aload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(0);
    <[IRInstr; 2]>::into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.local_var_entry(current_instr_data.next_index, 0) }
    ])
}