use std::collections::HashMap;
use std::mem::size_of;
use std::sync::Arc;

use itertools::Itertools;
use another_jit_vm::Register;

use rust_jvm_common::compressed_classfile::code::{CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::loading::LoaderName;

use crate::gc_memory_layout_common::{FramePointerOffset, StackframeMemoryLayout};
use crate::ir_to_java_layer::compiler::branching::{goto_, if_acmp, ReferenceEqualityType};
use crate::ir_to_java_layer::compiler::consts::const_64;
use crate::ir_to_java_layer::compiler::dup::dup;
use crate::ir_to_java_layer::compiler::invoke::invokestatic;
use crate::ir_to_java_layer::compiler::returns::{ireturn, return_void};
use crate::ir_to_java_layer::vm_exit_abi::{IRVMExitType, VMExitTypeWithArgs};
use crate::jit::{ByteCodeOffset, LabelName, MethodResolver};
use crate::jit::ir::{IRInstr, IRLabel};
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::JVMState;
use crate::method_table::MethodId;
use crate::native_to_ir_layer::{FRAME_HEADER_END_OFFSET, FRAME_HEADER_PREV_RBP_OFFSET};
use crate::runtime_class::RuntimeClass;
use crate::stack_entry::FrameView;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ByteCodeIndex(u16);

// all metadata needed to compile to ir, excluding resolver stuff
pub struct JavaCompilerMethodAndFrameData {
    max_locals: u16,
    max_stack: u16,
    stack_depth_by_index: Vec<u16>,
    code_by_index: Vec<CompressedInstruction>,
    index_by_bytecode_offset: HashMap<ByteCodeOffset, ByteCodeIndex>,

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
            index_by_bytecode_offset: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).enumerate().map(|(index, (bytecode_offset, _))| (ByteCodeOffset(*bytecode_offset), ByteCodeIndex(index as u16))).collect(),
        }
    }

    pub fn operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
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

pub struct CurrentInstructionCompilerData<'l, 'k> {
    current_index: ByteCodeIndex,
    next_index: ByteCodeIndex,
    current_offset: ByteCodeOffset,
    compiler_labeler: &'k mut CompilerLabeler<'l>,
}

pub struct CompilerLabeler<'l> {
    labeler: &'l Labeler,
    labels_vec: Vec<IRLabel>,
    label_to_offset: HashMap<ByteCodeIndex, LabelName>,
    index_by_bytecode_offset: &'l HashMap<ByteCodeOffset, ByteCodeIndex>,
}

impl<'l> CompilerLabeler<'l> {
    pub fn label_at(&mut self, byte_code_offset: ByteCodeOffset) -> LabelName {
        let byte_code_index = self.index_by_bytecode_offset[&byte_code_offset];
        let labels_vec = &mut self.labels_vec;
        let label_to_offset = &mut self.label_to_offset;
        let labeler = self.labeler;
        *label_to_offset.entry(byte_code_index).or_insert_with(|| {
            labeler.new_label(labels_vec)
        })
    }
}

pub fn compile_to_ir(resolver: &MethodResolver<'vm_life>, labeler: &Labeler, method_frame_data: &JavaCompilerMethodAndFrameData) -> Vec<IRInstr> {
    let cinstructions: &[CompressedInstruction] = method_frame_data.code_by_index.as_slice();
    let mut initial_ir: Vec<IRInstr> = vec![];
    let mut labels = vec![];
    let mut compiler_labeler = CompilerLabeler {
        labeler,
        labels_vec: vec![],
        label_to_offset: Default::default(),
        index_by_bytecode_offset: &method_frame_data.index_by_bytecode_offset,
    };
    for (i, compressed_instruction) in cinstructions.iter().enumerate() {
        let current_offset = ByteCodeOffset(compressed_instruction.offset);
        let current_index = ByteCodeIndex(i as u16);
        let next_index = ByteCodeIndex((i + 1) as u16);
        let current_instr_data = CurrentInstructionCompilerData {
            current_index,
            next_index,
            current_offset,
            compiler_labeler: &mut compiler_labeler,
        };
        match &compressed_instruction.info {
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                initial_ir.extend(invokestatic(resolver, method_frame_data, current_instr_data, *method_name, descriptor, classname_ref_type));
            }
            CompressedInstructionInfo::return_ => {
                initial_ir.extend(return_void(method_frame_data));
            }
            CompressedInstructionInfo::ireturn => {
                initial_ir.extend(ireturn(method_frame_data, current_instr_data));
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
            CompressedInstructionInfo::if_acmpne(offset) => {
                initial_ir.extend(if_acmp(method_frame_data, current_instr_data, ReferenceEqualityType::NE, *offset as i32));
            }
            CompressedInstructionInfo::if_acmpeq(offset) => {
                initial_ir.extend(if_acmp(method_frame_data, current_instr_data, ReferenceEqualityType::EQ, *offset as i32));
            }
            CompressedInstructionInfo::iconst_0 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, 0))
            }
            CompressedInstructionInfo::iconst_1 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, 1))
            }
            CompressedInstructionInfo::iconst_2 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, 2))
            }
            CompressedInstructionInfo::iconst_3 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, 3))
            }
            CompressedInstructionInfo::iconst_4 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, 4))
            }
            CompressedInstructionInfo::iconst_5 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, 5))
            }
            CompressedInstructionInfo::iconst_m1 => {
                initial_ir.extend(const_64(method_frame_data, current_instr_data, -1i64 as u64))
            }
            CompressedInstructionInfo::goto_(offset) => {
                initial_ir.extend(goto_(method_frame_data, current_instr_data, *offset as i32))
            }
            CompressedInstructionInfo::new(ccn) => {
                match resolver.lookup_type_loaded(&(*ccn).into()) {
                    None => {
                        let exit_label = todo!();
                        initial_ir.push(
                            IRInstr::VMExit2 {
                                exit_type: IRVMExitType::LoadClassAndRecompile{ class: todo!() },
                            },
                        );
                    }
                    Some((loaded_class, loader)) => {
                        todo!()
                    }
                }
            }
            CompressedInstructionInfo::dup => {
                initial_ir.extend(dup(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                match resolver.lookup_type_loaded(&CPDType::Ref(classname_ref_type.clone())) {
                    None => {
                        let exit_label = labeler.new_label(&mut labels);
                        initial_ir.push(
                            IRInstr::VMExit2 {
                                exit_type: IRVMExitType::LoadClassAndRecompile{ class: todo!() },
                            },
                        );
                    }
                    Some(_) => {
                        todo!()
                    }
                }
            }
            CompressedInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
                match resolver.lookup_virtual(CPDType::Ref(classname_ref_type.clone()), *method_name, descriptor.clone()) {
                    None => {
                        let exit_label = labeler.new_label(&mut labels);
                        initial_ir.push(
                            IRInstr::VMExit2 {
                                exit_type: IRVMExitType::LoadClassAndRecompile{ class: todo!() },
                            },
                        );
                    }
                    Some((method_id, is_native)) => {
                        if is_native {
                            todo!()
                        } else {
                            todo!()
                        }
                    }
                }
            }
            other => {
                dbg!(other);
                todo!()
            }
        }
    }
    initial_ir.into_iter().collect_vec()
}

pub mod invoke {
    use itertools::Either;

    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
    use rust_jvm_common::compressed_classfile::names::MethodName;

    use crate::ir_to_java_layer::compiler::{array_into_iter, CompilerLabeler, CurrentInstructionCompilerData, JavaCompilerMethodAndFrameData};
    use crate::ir_to_java_layer::vm_exit_abi::{IRVMExitType, VMExitTypeWithArgs};
    use crate::jit::ir::{IRInstr};
    use crate::jit::MethodResolver;

    pub fn invokestatic(resolver: &MethodResolver<'vm_life>, method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData, method_name: MethodName, descriptor: &CMethodDescriptor, classname_ref_type: &CPRefType) -> impl Iterator<Item=IRInstr> {
        let class_as_cpdtype = CPDType::Ref(classname_ref_type.clone());
        match resolver.lookup_static(class_as_cpdtype.clone(), method_name, descriptor.clone()) {
            None => {
                let before_exit_label = current_instr_data.compiler_labeler.label_at(current_instr_data.current_offset);
                Either::Left(array_into_iter([IRInstr::VMExit2 {
                    exit_type: IRVMExitType::LoadClassAndRecompile{
                        class: class_as_cpdtype
                    },
                }]))
            }
            Some((method_id, is_native)) => {
                Either::Right(if is_native {
                    let exit_label = current_instr_data.compiler_labeler.label_at(current_instr_data.current_offset);
                    let num_args = resolver.num_args(method_id);
                    let arg_start_frame_offset = method_frame_data.operand_stack_entry(current_instr_data.current_index,num_args);
                    array_into_iter([IRInstr::VMExit2 {
                        exit_type: IRVMExitType::RunStaticNative {
                            method_id,
                            arg_start_frame_offset,
                            num_args
                        },
                    }])
                } else {
                    todo!()
                })
            }
        }
    }
}

pub mod dup;
pub mod returns;
pub mod consts;
pub mod branching;


pub fn aload_n(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: &CurrentInstructionCompilerData, n: u16) -> impl Iterator<Item=IRInstr> {
    //todo have register allocator
    let temp = Register(0);
    array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.local_var_entry(current_instr_data.current_index, n), to: temp },
        IRInstr::StoreFPRelative { from: temp, to: method_frame_data.local_var_entry(current_instr_data.next_index, 0) }
    ])
}

pub fn array_into_iter<T, const N: usize>(array: [T; N]) -> impl Iterator<Item=T> {
    <[T; N]>::into_iter(array)
}