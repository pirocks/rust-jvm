use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::Arc;

use iced_x86::CC_be::na;
use itertools::{Either, Itertools};

use another_jit_vm::Register;
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, LabelName, RestartPointGenerator, RestartPointID};
use another_jit_vm_ir::ir_stack::FRAME_HEADER_END_OFFSET;
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::FramePointerOffset;
use jvmti_jni_bindings::jvalue;
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::classfile::{Atype, Code};
use rust_jvm_common::classfile::InstructionInfo::getfield;
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo, CompressedLdc2W, CompressedLdcW};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::loading::LoaderName;
use verification::verifier::codecorrectness::method_is_type_safe;
use verification::verifier::Frame;

use crate::instructions::invoke::native::mhn_temp::init;
use crate::ir_to_java_layer::compiler::allocate::{anewarray, new, newarray};
use crate::ir_to_java_layer::compiler::arrays::arraylength;
use crate::ir_to_java_layer::compiler::branching::{goto_, if_, if_acmp, if_nonnull, if_null, IntEqualityType, ReferenceComparisonType};
use crate::ir_to_java_layer::compiler::consts::const_64;
use crate::ir_to_java_layer::compiler::dup::dup;
use crate::ir_to_java_layer::compiler::fields::{gettfield, putfield};
use crate::ir_to_java_layer::compiler::invoke::{invokespecial, invokestatic, invokevirtual};
use crate::ir_to_java_layer::compiler::ldc::{ldc_class, ldc_double, ldc_float, ldc_string};
use crate::ir_to_java_layer::compiler::local_var_loads::{aload_n, iload_n};
use crate::ir_to_java_layer::compiler::local_var_stores::astore_n;
use crate::ir_to_java_layer::compiler::monitors::{monitor_enter, monitor_exit};
use crate::ir_to_java_layer::compiler::returns::{areturn, ireturn, return_void};
use crate::ir_to_java_layer::compiler::static_fields::putstatic;
use crate::ir_to_java_layer::compiler::throw::athrow;
use crate::jit::MethodResolver;
use crate::jit::state::{Labeler, NaiveStackframeLayout};
use crate::JVMState;
use crate::runtime_class::RuntimeClass;
use crate::stack_entry::FrameView;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ByteCodeIndex(pub u16);

// all metadata needed to compile to ir, excluding resolver stuff
pub struct JavaCompilerMethodAndFrameData {
    layout: YetAnotherLayoutImpl,
    index_by_bytecode_offset: HashMap<ByteCodeOffset, ByteCodeIndex>,
    current_method_id: MethodId,
}

impl JavaCompilerMethodAndFrameData {
    pub fn new(jvm: &'vm_life JVMState<'vm_life>, method_id: MethodId) -> Self {
        let function_frame_type_guard = jvm.function_frame_type_data.read().unwrap();
        let frames = function_frame_type_guard.get(&method_id).unwrap();
        let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
        let view = rc.view();
        let method_view = view.method_view_i(method_i);
        let code = method_view.code_attribute().unwrap();
        Self {
            layout: YetAnotherLayoutImpl::new(frames, code),
            index_by_bytecode_offset: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).enumerate().map(|(index, (bytecode_offset, _))| (*bytecode_offset, ByteCodeIndex(index as u16))).collect(),
            current_method_id: method_id,
        }
    }

    pub fn operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
        self.layout.operand_stack_entry(index, from_end)
    }

    pub fn local_var_entry(&self, index: ByteCodeIndex, local_var_index: u16) -> FramePointerOffset {
        self.layout.local_var_entry(index, local_var_index)
    }

    pub fn full_frame_size(&self) -> usize {
        self.layout.full_frame_size()
    }
}

pub struct YetAnotherLayoutImpl {
    max_locals: u16,
    max_stack: u16,
    stack_depth_by_index: Vec<u16>,
    code_by_index: Vec<CompressedInstruction>,
}

impl YetAnotherLayoutImpl {
    pub fn new(frames: &HashMap<ByteCodeOffset, Frame>, code: &CompressedCode) -> Self {
        let stack_depth = frames.iter().sorted_by_key(|(offset, _)| *offset).enumerate().map(|(i, (_offset, frame))| frame.stack_map.len() as u16).collect();
        Self {
            max_locals: code.max_locals,
            max_stack: code.max_stack,
            stack_depth_by_index: stack_depth,
            code_by_index: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).map(|(_, instr)| instr.clone()).collect(),
        }
    }

    pub fn operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
        FramePointerOffset(FRAME_HEADER_END_OFFSET + (self.max_locals + self.stack_depth_by_index[index.0 as usize] - from_end - 1) as usize * size_of::<u64>())//-1 b/c stack depth is a len
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
    label_to_index: HashMap<ByteCodeIndex, LabelName>,
    index_by_bytecode_offset: &'l HashMap<ByteCodeOffset, ByteCodeIndex>,
}

impl<'l> CompilerLabeler<'l> {
    pub fn label_at(&mut self, byte_code_offset: ByteCodeOffset) -> LabelName {
        let byte_code_index = self.index_by_bytecode_offset[&byte_code_offset];
        let labels_vec = &mut self.labels_vec;
        let label_to_offset = &mut self.label_to_index;
        let labeler = self.labeler;
        *label_to_offset.entry(byte_code_index).or_insert_with(|| {
            labeler.new_label(labels_vec)
        })
    }
}


pub fn compile_to_ir(resolver: &MethodResolver<'vm_life>, labeler: &Labeler, method_frame_data: &JavaCompilerMethodAndFrameData) -> Vec<(ByteCodeOffset, IRInstr)> {
    let cinstructions = method_frame_data.layout.code_by_index.as_slice();
    let mut final_ir_without_labels: Vec<(ByteCodeOffset, IRInstr)> = vec![];
    let mut compiler_labeler = CompilerLabeler {
        labeler,
        labels_vec: vec![],
        label_to_index: Default::default(),
        index_by_bytecode_offset: &method_frame_data.index_by_bytecode_offset,
    };
    let mut restart_point_generator = RestartPointGenerator::new();
    let mut prev_offset: Option<ByteCodeOffset> = None;
    for (i, compressed_instruction) in cinstructions.iter().enumerate() {
        let current_offset = compressed_instruction.offset;
        let current_index = ByteCodeIndex(i as u16);
        let next_index = ByteCodeIndex((i + 1) as u16);
        let current_instr_data = CurrentInstructionCompilerData {
            current_index,
            next_index,
            current_offset,
            compiler_labeler: &mut compiler_labeler,
        };
        let mut this_function_ir = vec![];
        if let Some(prev_offset) = prev_offset {
            this_function_ir.push(IRInstr::VMExit2 { exit_type: IRVMExitType::TraceInstructionAfter { method_id: method_frame_data.current_method_id, offset: prev_offset } });
        }
        this_function_ir.push(IRInstr::VMExit2 { exit_type: IRVMExitType::TraceInstructionBefore { method_id: method_frame_data.current_method_id, offset: current_offset } });
        match &compressed_instruction.info {
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                this_function_ir.extend(invokestatic(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, *method_name, descriptor, classname_ref_type));
            }
            CompressedInstructionInfo::return_ => {
                this_function_ir.extend(return_void(method_frame_data));
            }
            CompressedInstructionInfo::ireturn => {
                this_function_ir.extend(ireturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::aload_0 => {
                this_function_ir.extend(aload_n(method_frame_data, &current_instr_data, 0));
            }
            CompressedInstructionInfo::aload_1 => {
                this_function_ir.extend(aload_n(method_frame_data, &current_instr_data, 1));
            }
            CompressedInstructionInfo::aload_2 => {
                this_function_ir.extend(aload_n(method_frame_data, &current_instr_data, 2));
            }
            CompressedInstructionInfo::aload_3 => {
                this_function_ir.extend(aload_n(method_frame_data, &current_instr_data, 3));
            }
            CompressedInstructionInfo::aload(n) => {
                this_function_ir.extend(aload_n(method_frame_data, &current_instr_data, *n as u16));
            }
            CompressedInstructionInfo::aconst_null => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 0))
            }
            CompressedInstructionInfo::if_acmpne(offset) => {
                this_function_ir.extend(if_acmp(method_frame_data, current_instr_data, ReferenceComparisonType::NE, *offset as i32));
            }
            CompressedInstructionInfo::if_acmpeq(offset) => {
                this_function_ir.extend(if_acmp(method_frame_data, current_instr_data, ReferenceComparisonType::EQ, *offset as i32));
            }
            CompressedInstructionInfo::ifne(offset) => {
                this_function_ir.extend(if_(method_frame_data, current_instr_data, IntEqualityType::NE, *offset as i32))
            }
            CompressedInstructionInfo::iconst_0 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 0))
            }
            CompressedInstructionInfo::iconst_1 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 1))
            }
            CompressedInstructionInfo::iconst_2 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 2))
            }
            CompressedInstructionInfo::iconst_3 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 3))
            }
            CompressedInstructionInfo::iconst_4 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 4))
            }
            CompressedInstructionInfo::iconst_5 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 5))
            }
            CompressedInstructionInfo::iconst_m1 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, -1i64 as u64))
            }
            CompressedInstructionInfo::goto_(offset) => {
                this_function_ir.extend(goto_(method_frame_data, current_instr_data, *offset as i32))
            }
            CompressedInstructionInfo::new(ccn) => {
                this_function_ir.extend(new(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, *ccn))
            }
            CompressedInstructionInfo::dup => {
                this_function_ir.extend(dup(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::putfield { name, desc, target_class } => {
                this_function_ir.extend(putfield(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, *target_class, *name))
            }
            CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                this_function_ir.extend(invokespecial(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, *method_name, descriptor, classname_ref_type))
            }
            CompressedInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
                this_function_ir.extend(invokevirtual(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, *method_name, descriptor, classname_ref_type))
            }
            CompressedInstructionInfo::putstatic { name, desc, target_class } => {
                this_function_ir.extend(putstatic(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, *target_class, *name))
            }
            CompressedInstructionInfo::anewarray(elem_type) => {
                this_function_ir.extend(anewarray(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, elem_type))
            }
            CompressedInstructionInfo::ldc(either) => {
                match either {
                    Either::Left(left) => {
                        match left {
                            CompressedLdcW::String { str } => {
                                let compressed = resolver.get_commpressed_version_of_wtf8(str);
                                this_function_ir.extend(ldc_string(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, compressed))
                            }
                            CompressedLdcW::Class { type_ } => {
                                this_function_ir.extend(ldc_class(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, type_))
                            }
                            CompressedLdcW::Float { float } => {
                                this_function_ir.extend(ldc_float(method_frame_data, &current_instr_data, *float))
                            }
                            CompressedLdcW::Integer { .. } => todo!(),
                            CompressedLdcW::MethodType { .. } => todo!(),
                            CompressedLdcW::MethodHandle { .. } => todo!(),
                            CompressedLdcW::LiveObject(_) => todo!(),
                        }
                    }
                    Either::Right(right) => {
                        match right {
                            CompressedLdc2W::Long(_) => todo!(),
                            CompressedLdc2W::Double(_) => todo!(),
                        }
                    }
                }
            }
            CompressedInstructionInfo::arraylength => {
                this_function_ir.extend(arraylength(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::astore_1 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::astore_2 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::ifnonnull(offset) => {
                this_function_ir.extend(if_nonnull(method_frame_data, current_instr_data, *offset as i32))
            }
            CompressedInstructionInfo::getfield { name, desc: _, target_class } => {
                this_function_ir.extend(gettfield(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, *target_class, *name))
            }
            //todo handle implicit monitor enters on synchronized  functions
            CompressedInstructionInfo::monitorenter => {
                this_function_ir.extend(monitor_enter(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::monitorexit => {
                this_function_ir.extend(monitor_exit(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ifnull(offset) => {
                this_function_ir.extend(if_null(method_frame_data, current_instr_data, *offset as i32))
            }
            CompressedInstructionInfo::astore_3 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::athrow => {
                this_function_ir.extend(athrow());
            }
            CompressedInstructionInfo::areturn => {
                this_function_ir.extend(areturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::iload_1 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::newarray(atype) => {
                this_function_ir.extend(newarray(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, atype))
            }
            CompressedInstructionInfo::i2l => {
                //for now does nothing but should really store ints as  actual 32 bit ints so in future todo
            }
            CompressedInstructionInfo::ldc2_w(ldc2) => {
                match ldc2 {
                    CompressedLdc2W::Long(_) => { todo!() }
                    CompressedLdc2W::Double(double) => {
                        this_function_ir.extend(ldc_double(method_frame_data, &current_instr_data, *double))
                    }
                }
            }
            CompressedInstructionInfo::sipush(val) => {
                this_function_ir.extend(array_into_iter([IRInstr::Const16bit { to: Register(1), const_: *val },
                    IRInstr::StoreFPRelative { from: Register(1), to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0) }]))
            }
            CompressedInstructionInfo::iload_0 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::if_icmpgt(offset) => {
                this_function_ir.extend(if_acmp(method_frame_data, current_instr_data, ReferenceComparisonType::GT, *offset as i32));
            }
            other => {
                dbg!(other);
                todo!()
            }
        }
        final_ir_without_labels.extend(std::iter::repeat(compressed_instruction.offset).zip(this_function_ir.into_iter()));
        prev_offset = Some(current_offset);
    }
    let mut final_ir = vec![];
    for (offset, ir_instr) in final_ir_without_labels {
        let index = *compiler_labeler.index_by_bytecode_offset.get(&offset).unwrap();
        if let Some(label_name) = compiler_labeler.label_to_index.remove(&index) {
            final_ir.push((offset, IRInstr::Label(IRLabel { name: label_name })));
        }
        final_ir.push((offset, ir_instr));
    }
    final_ir
}

pub mod throw;
pub mod monitors;
pub mod arrays;
pub mod static_fields;
pub mod fields;
pub mod allocate;
pub mod invoke;
pub mod dup;
pub mod returns;
pub mod consts;
pub mod branching;
pub mod local_var_loads;
pub mod local_var_stores;
pub mod ldc;

pub fn array_into_iter<T, const N: usize>(array: [T; N]) -> impl Iterator<Item=T> {
    <[T; N]>::into_iter(array)
}