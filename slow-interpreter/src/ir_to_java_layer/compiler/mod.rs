use std::collections::HashMap;
use std::ffi::c_void;
use std::iter;
use std::mem::size_of;
use std::sync::Arc;

use iced_x86::CC_be::na;
use itertools::{Either, Itertools};
use libc::input_absinfo;

use another_jit_vm::{FloatRegister, MMRegister, Register};
use another_jit_vm_ir::compiler::{FloatCompareMode, IRCallTarget, IRInstr, IRLabel, LabelName, RestartPointGenerator, RestartPointID};
use another_jit_vm_ir::compiler::IRInstr::{IRCall, VMExit2};
use another_jit_vm_ir::ir_stack::FRAME_HEADER_END_OFFSET;
use another_jit_vm_ir::vm_exit_abi::{InvokeInterfaceResolve, IRVMExitType};
use classfile_view::view::HasAccessFlags;
use gc_memory_layout_common::FramePointerOffset;
use jvmti_jni_bindings::{jlong, jvalue};
use rust_jvm_common::{ByteCodeOffset, MethodId};
use rust_jvm_common::classfile::{Atype, Code, IInc};
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::code::{CompressedCode, CompressedInstruction, CompressedInstructionInfo, CompressedLdc2W, CompressedLdcW};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::vtype::VType;
use verification::verifier::codecorrectness::method_is_type_safe;
use verification::verifier::Frame;

use crate::instructions::invoke::native::mhn_temp::init;
use crate::ir_to_java_layer::compiler::allocate::{anewarray, new, newarray};
use crate::ir_to_java_layer::compiler::arithmetic::{iadd, iinc, imul, irem, isub, ladd, lcmp};
use crate::ir_to_java_layer::compiler::array_load::{aaload, caload};
use crate::ir_to_java_layer::compiler::array_store::{aastore, castore, iastore};
use crate::ir_to_java_layer::compiler::arrays::arraylength;
use crate::ir_to_java_layer::compiler::bitmanip::{iand, ior, ishl, ishr, iushr, ixor, land, lshl};
use crate::ir_to_java_layer::compiler::branching::{goto_, if_, if_acmp, if_icmp, if_nonnull, if_null, IntEqualityType, ReferenceComparisonType};
use crate::ir_to_java_layer::compiler::consts::{bipush, const_64, dconst, fconst, sipush};
use crate::ir_to_java_layer::compiler::dup::{dup, dup2, dup_x1};
use crate::ir_to_java_layer::compiler::fields::{getfield, putfield};
use crate::ir_to_java_layer::compiler::float_arithmetic::{fadd, fcmpg, fcmpl, fdiv, fmul};
use crate::ir_to_java_layer::compiler::float_convert::{f2i, i2f};
use crate::ir_to_java_layer::compiler::instance_of_and_casting::{checkcast, instanceof};
use crate::ir_to_java_layer::compiler::invoke::{invoke_interface, invokespecial, invokestatic, invokevirtual};
use crate::ir_to_java_layer::compiler::ldc::{ldc_class, ldc_double, ldc_float, ldc_integer, ldc_long, ldc_string};
use crate::ir_to_java_layer::compiler::local_var_loads::{aload_n, fload_n, iload_n, lload_n};
use crate::ir_to_java_layer::compiler::local_var_stores::{astore_n, fstore_n, istore_n, lstore_n};
use crate::ir_to_java_layer::compiler::monitors::{monitor_enter, monitor_exit};
use crate::ir_to_java_layer::compiler::returns::{areturn, dreturn, freturn, ireturn, lreturn, return_void};
use crate::ir_to_java_layer::compiler::static_fields::{getstatic, putstatic};
use crate::ir_to_java_layer::compiler::throw::athrow;
use crate::java_values::NativeJavaValue;
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
        let function_frame_type_guard = jvm.function_frame_type_data_no_tops.read().unwrap();
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

    pub fn is_category_2(&self, index: ByteCodeIndex, from_end: u16) -> bool {
        self.layout.is_category_2(index, from_end)
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
    is_type_2_computational_type: Vec<Vec<bool>>,
    code_by_index: Vec<CompressedInstruction>,
}

impl YetAnotherLayoutImpl {
    pub fn new(frames_no_top: &HashMap<ByteCodeOffset, Frame>, code: &CompressedCode) -> Self {
        let stack_depth = frames_no_top.iter().sorted_by_key(|(offset, _)| *offset).enumerate().map(|(i, (_offset, frame))| {
            assert!(frame.stack_map.iter().all(|types| !matches!(types, VType::TopType)));
            frame.stack_map.len() as u16
        }).collect();
        let computational_type = frames_no_top.iter().sorted_by_key(|(offset, _)| *offset).enumerate().map(|(i, (_offset, frame))| {
            assert!(frame.stack_map.iter().all(|types| !matches!(types, VType::TopType)));
            frame.stack_map.iter().map(|vtype| Self::is_type_2_computational_type(vtype)).collect()
        }).collect();
        Self {
            max_locals: code.max_locals,
            max_stack: code.max_stack,
            stack_depth_by_index: stack_depth,
            is_type_2_computational_type: computational_type,
            code_by_index: code.instructions.iter().sorted_by_key(|(byte_code_offset, _)| *byte_code_offset).map(|(_, instr)| instr.clone()).collect(),
        }
    }

    fn is_type_2_computational_type(vtype: &VType) -> bool {
        match vtype {
            VType::DoubleType => true,
            VType::FloatType => false,
            VType::IntType => false,
            VType::LongType => true,
            VType::Class(_) => false,
            VType::ArrayReferenceType(_) => false,
            VType::VoidType => false,
            VType::TopType => false,
            VType::NullType => false,
            VType::Uninitialized(_) => false,
            VType::UninitializedThis => false,
            VType::UninitializedThisOrClass(_) => false,
            VType::TwoWord => true,
            VType::OneWord => false,
            VType::Reference => false,
            VType::UninitializedEmpty => false
        }
    }

    pub fn operand_stack_entry(&self, index: ByteCodeIndex, from_end: u16) -> FramePointerOffset {
        FramePointerOffset(FRAME_HEADER_END_OFFSET + (self.max_locals + self.stack_depth_by_index[index.0 as usize] - from_end - 1) as usize * size_of::<u64>())//-1 b/c stack depth is a len
    }

    pub fn is_category_2(&self, index: ByteCodeIndex, from_end: u16) -> bool {
        let category_2_array = &self.is_type_2_computational_type[index.0 as usize];
        *category_2_array.iter().rev().nth(from_end as usize).unwrap()
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
            CompressedInstructionInfo::freturn => {
                this_function_ir.extend(freturn(method_frame_data, current_instr_data));
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
            CompressedInstructionInfo::fload_0 => {
                this_function_ir.extend(fload_n(method_frame_data, &current_instr_data, 0));
            }
            CompressedInstructionInfo::fload_1 => {
                this_function_ir.extend(fload_n(method_frame_data, &current_instr_data, 1));
            }
            CompressedInstructionInfo::fload_2 => {
                this_function_ir.extend(fload_n(method_frame_data, &current_instr_data, 2));
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
            CompressedInstructionInfo::ifeq(offset) => {
                this_function_ir.extend(if_(method_frame_data, current_instr_data, IntEqualityType::EQ, *offset as i32))
            }
            CompressedInstructionInfo::iflt(offset) => {
                this_function_ir.extend(if_(method_frame_data, current_instr_data, IntEqualityType::LT, *offset as i32))
            }
            CompressedInstructionInfo::ifle(offset) => {
                this_function_ir.extend(if_(method_frame_data, current_instr_data, IntEqualityType::LE, *offset as i32))
            }
            CompressedInstructionInfo::ifge(offset) => {
                this_function_ir.extend(if_(method_frame_data, current_instr_data, IntEqualityType::GE, *offset as i32))
            }
            CompressedInstructionInfo::ifgt(offset) => {
                this_function_ir.extend(if_(method_frame_data, current_instr_data, IntEqualityType::GT, *offset as i32))
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
                            CompressedLdcW::Integer { integer } => {
                                this_function_ir.extend(ldc_integer(method_frame_data, &current_instr_data, *integer))
                            }
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
                this_function_ir.extend(getfield(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, *target_class, *name))
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
            CompressedInstructionInfo::astore_0 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::astore_1 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::astore_2 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::astore_3 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::astore(index) => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::athrow => {
                this_function_ir.extend(athrow());
            }
            CompressedInstructionInfo::areturn => {
                this_function_ir.extend(areturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::lreturn => {
                this_function_ir.extend(lreturn(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dreturn => {
                this_function_ir.extend(dreturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::iload_1 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::newarray(atype) => {
                this_function_ir.extend(newarray(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, atype))
            }
            CompressedInstructionInfo::i2l => {
                //TODO bug there are places where we don't sign extend properly
                //for now does nothing but should really store ints as  actual 32 bit ints so in future todo
            }
            CompressedInstructionInfo::ldc2_w(ldc2) => {
                match ldc2 {
                    CompressedLdc2W::Long(long) => {
                        this_function_ir.extend(ldc_long(method_frame_data, &current_instr_data, *long))
                    }
                    CompressedLdc2W::Double(double) => {
                        this_function_ir.extend(ldc_double(method_frame_data, &current_instr_data, *double))
                    }
                }
            }
            CompressedInstructionInfo::sipush(val) => {
                this_function_ir.extend(sipush(method_frame_data, &current_instr_data, val))
            }
            CompressedInstructionInfo::iload_0 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::if_icmpgt(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::GT, *offset as i32));
            }
            CompressedInstructionInfo::getstatic { name, desc, target_class } => {
                this_function_ir.extend(getstatic(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, *target_class, *name));
            }
            CompressedInstructionInfo::if_icmplt(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::LT, *offset as i32));
            }
            CompressedInstructionInfo::if_icmple(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::LE, *offset as i32));
            }
            CompressedInstructionInfo::if_icmpne(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::NE, *offset as i32));
            }
            CompressedInstructionInfo::if_icmpeq(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::EQ, *offset as i32));
            }
            CompressedInstructionInfo::if_icmpge(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::GE, *offset as i32));
            }
            CompressedInstructionInfo::ladd => {
                this_function_ir.extend(ladd(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::lcmp => {
                this_function_ir.extend(lcmp(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::bipush(val_) => {
                this_function_ir.extend(bipush(method_frame_data, current_instr_data, val_))
            }
            CompressedInstructionInfo::lshl => {
                this_function_ir.extend(lshl(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::land => {
                this_function_ir.extend(land(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::istore_1 => {
                this_function_ir.extend(istore_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::istore_2 => {
                this_function_ir.extend(istore_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::istore_3 => {
                this_function_ir.extend(istore_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::istore(index) => {
                this_function_ir.extend(istore_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::fstore(index) => {
                this_function_ir.extend(fstore_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::iload_2 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::iload_3 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::iload(index) => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::isub => {
                this_function_ir.extend(isub(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ixor => {
                this_function_ir.extend(ixor(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ior => {
                this_function_ir.extend(ior(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::iand => {
                this_function_ir.extend(iand(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::caload => {
                this_function_ir.extend(caload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::aaload => {
                this_function_ir.extend(aaload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::castore => {
                this_function_ir.extend(castore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::iastore => {
                this_function_ir.extend(iastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::aastore => {
                this_function_ir.extend(aastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::instanceof(cpdtype) => {
                this_function_ir.extend(instanceof(resolver, method_frame_data, &current_instr_data, cpdtype))
            }
            CompressedInstructionInfo::checkcast(cpdtype) => {
                this_function_ir.extend(checkcast(resolver, method_frame_data, &current_instr_data, cpdtype))
            }
            CompressedInstructionInfo::iinc(IInc { index, const_ }) => {
                this_function_ir.extend(iinc(method_frame_data, current_instr_data, index, const_))
            }
            CompressedInstructionInfo::fconst_0 => {
                this_function_ir.extend(fconst(method_frame_data, current_instr_data, 0.0))
            }
            CompressedInstructionInfo::fconst_1 => {
                this_function_ir.extend(fconst(method_frame_data, current_instr_data, 1.0))
            }
            CompressedInstructionInfo::fconst_2 => {
                this_function_ir.extend(fconst(method_frame_data, current_instr_data, 2.0))
            }
            CompressedInstructionInfo::dconst_0 => {
                this_function_ir.extend(dconst(method_frame_data, current_instr_data, 0.0))
            }
            CompressedInstructionInfo::lconst_0 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 0))
            }
            CompressedInstructionInfo::lconst_1 => {
                this_function_ir.extend(const_64(method_frame_data, current_instr_data, 1))
            }
            CompressedInstructionInfo::lload(index) => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::lload_1 => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::lload_2 => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::fload(index) => {
                this_function_ir.extend(fload_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::invokeinterface { method_name, descriptor, classname_ref_type, count } => {
                this_function_ir.extend(invoke_interface(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, method_name, descriptor, classname_ref_type))
            }
            CompressedInstructionInfo::pop => {
                this_function_ir.extend(array_into_iter([]))
            }
            CompressedInstructionInfo::iadd => {
                this_function_ir.extend(iadd(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::imul => {
                this_function_ir.extend(imul(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dup_x1 => {
                this_function_ir.extend(dup_x1(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dup2 => {
                this_function_ir.extend(dup2(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::fcmpg => {
                this_function_ir.extend(fcmpg(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fcmpl => {
                this_function_ir.extend(fcmpl(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::i2f => {
                this_function_ir.extend(i2f(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::fmul => {
                this_function_ir.extend(fmul(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fadd => {
                this_function_ir.extend(fadd(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fdiv => {
                this_function_ir.extend(fdiv(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::f2i => {
                this_function_ir.extend(f2i(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::iushr => {
                this_function_ir.extend(iushr(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ishr => {
                this_function_ir.extend(ishr(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ishl => {
                this_function_ir.extend(ishl(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::irem => {
                this_function_ir.extend(irem(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::lstore_2 => {
                this_function_ir.extend(lstore_n(method_frame_data, &current_instr_data, 2))
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


pub mod float_convert;
pub mod float_arithmetic;
pub mod instance_of_and_casting;
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
pub mod array_load;
pub mod array_store;
pub mod ldc;
pub mod arithmetic;
pub mod bitmanip;

pub fn array_into_iter<T, const N: usize>(array: [T; N]) -> impl Iterator<Item=T> {
    <[T; N]>::into_iter(array)
}