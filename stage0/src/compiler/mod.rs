use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::iter::repeat;
use std::sync::atomic::{AtomicU32, Ordering};

use itertools::Either;

use another_jit_vm::{IRMethodID, Register};
use another_jit_vm_ir::compiler::{IRInstr, IRLabel, LabelName, RestartPointGenerator, Size};
use another_jit_vm_ir::vm_exit_abi::IRVMExitType;
use classfile_view::view::ClassView;
use gc_memory_layout_common::frame_layout::NativeStackframeMemoryLayout;
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset, MethodId};
use rust_jvm_common::classfile::{IInc, LookupSwitch, TableSwitch, Wide};
use rust_jvm_common::compressed_classfile::code::{CompressedInstructionInfo, CompressedLdc2W, CompressedLdcW};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;


use crate::compiler::allocate::{anewarray, multianewarray, new, newarray};
use crate::compiler::arithmetic::{iadd, idiv, iinc, imul, ineg, irem, isub, ladd, lcmp, ldiv, lmul, lneg, lrem, lsub};
use crate::compiler::array_load::{aaload, baload, caload, daload, faload, iaload, laload, saload};
use crate::compiler::array_store::{aastore, bastore, castore, dastore, fastore, iastore, lastore, sastore};
use crate::compiler::arrays::arraylength;
use crate::compiler::bitmanip::{iand, ior, ishl, ishr, iushr, ixor, land, lor, lshl, lshr, lushr, lxor};
use crate::compiler::branching::{goto_, if_, if_acmp, if_icmp, if_nonnull, if_null, IntEqualityType, lookup_switch, ReferenceComparisonType, tableswitch};
use crate::compiler::consts::{bipush, const_64, dconst, fconst, sipush};
use crate::compiler::dup::{dup, dup2, dup2_x1, dup2_x2, dup_x1, dup_x2};
use crate::compiler::fields::{getfield, putfield};
use crate::compiler::float_arithmetic::{dadd, dcmpg, dcmpl, ddiv, dmul, dneg, drem, dsub, fadd, fcmpg, fcmpl, fdiv, fmul, fneg, frem, fsub};
use crate::compiler::float_convert::{d2f, d2i, d2l, f2d, f2i, f2l, i2d, i2f, l2d, l2f};
use crate::compiler::instance_of_and_casting::{checkcast, instanceof};
use crate::compiler::int_convert::{i2b, i2c, i2l, i2s, l2i};
use crate::compiler::intrinsics::gen_intrinsic_ir;
use crate::compiler::invoke::{invoke_interface, invokespecial, invokestatic, invokevirtual};
use crate::compiler::ldc::{ldc_class, ldc_double, ldc_float, ldc_integer, ldc_long, ldc_string};
use crate::compiler::local_var_loads::{aload_n, dload_n, fload_n, iload_n, lload_n};
use crate::compiler::local_var_stores::{astore_n, dstore_n, fstore_n, istore_n, lstore_n};
use crate::compiler::monitors::{monitor_enter, monitor_exit};
use crate::compiler::returns::{areturn, dreturn, freturn, ireturn, lreturn, return_void};
use crate::compiler::static_fields::{getstatic, putstatic};
use crate::compiler::throw::athrow;
use crate::compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};

pub struct CurrentInstructionCompilerData<'l, 'k> {
    current_index: ByteCodeIndex,
    next_index: ByteCodeIndex,
    current_offset: ByteCodeOffset,
    compiler_labeler: &'k mut CompilerLabeler<'l>,
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

    pub fn local_label(&mut self) -> LabelName {
        let labels_vec = &mut self.labels_vec;
        let labeler = self.labeler;
        labeler.new_label(labels_vec)
    }
}

pub struct RecompileConditions {
    conditions: HashMap<MethodId, HashSet<NeedsRecompileIf>>,
}

impl RecompileConditions {
    pub fn new() -> Self {
        Self {
            conditions: HashMap::new()
        }
    }

    pub fn should_recompile<'gc>(&self, method_id: MethodId, method_resolver: &impl MethodResolver<'gc>, interpreter_debug: bool) -> bool {
        match self.conditions.get(&method_id) {
            None => {
                return true;
            }
            Some(needs_recompiling) => {
                if interpreter_debug {
                    // assert!(needs_recompiling.iter().any(|elem| matches!(elem,NeedsRecompileIf::Interpreted {..})))
                }
                for condition in needs_recompiling {
                    if condition.should_recompile(method_resolver) {
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn recompile_conditions(&mut self, method_id: MethodId) -> MethodRecompileConditions<'_> {
        self.conditions.insert(method_id, HashSet::new());
        MethodRecompileConditions {
            conditions: self.conditions.get_mut(&method_id).unwrap()
        }
    }
}


pub struct MethodRecompileConditions<'l> {
    conditions: &'l mut HashSet<NeedsRecompileIf>,
}

impl<'l> MethodRecompileConditions<'l> {
    pub fn add_condition(&mut self, condition: NeedsRecompileIf) {
        self.conditions.insert(condition);
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq, Hash)]
pub enum NeedsRecompileIf {
    FunctionRecompiled {
        function_method_id: MethodId,
        current_ir_method_id: IRMethodID,
    },
    FunctionCompiled {
        method_id: MethodId
    },
    ClassLoaded {
        class: CPDType
    },
    Interpreted {
        method_id: MethodId
    },
}

impl NeedsRecompileIf {
    pub fn should_recompile<'gc>(&self, method_resolver: &impl MethodResolver<'gc>) -> bool {
        match self {
            NeedsRecompileIf::FunctionRecompiled { function_method_id, current_ir_method_id } => {
                let (ir_method_id, _address) = method_resolver.lookup_ir_method_id_and_address(*function_method_id).unwrap();
                ir_method_id != *current_ir_method_id
            }
            NeedsRecompileIf::FunctionCompiled { method_id } => {
                method_resolver.lookup_ir_method_id_and_address(*method_id).is_some()
            }
            NeedsRecompileIf::ClassLoaded { class } => {
                method_resolver.lookup_type_inited_initing(class).is_some()
            }
            NeedsRecompileIf::Interpreted { method_id } => {
                !method_resolver.compile_interpreted(*method_id)
            }
        }
    }
}

pub fn native_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, labeler: &Labeler, method_id: MethodId, ir_method_id: IRMethodID) -> Vec<IRInstr> {
    //todo handle synchronized
    let empty = HashMap::new();
    let mut compiler_labeler = CompilerLabeler {
        labeler: labeler,
        labels_vec: vec![],
        label_to_index: Default::default(),
        index_by_bytecode_offset: &empty,
    };
    let layout = NativeStackframeMemoryLayout { num_locals: resolver.num_locals(method_id) };
    if let Some(intrinsic_ir) = gen_intrinsic_ir(resolver, &layout, method_id, ir_method_id, &mut compiler_labeler) {
        return intrinsic_ir;
    }

    let mut res = vec![IRInstr::IRStart {
        temp_register: Register(2),
        ir_method_id,
        method_id,
        frame_size: layout.full_frame_size(),
        num_locals: resolver.num_locals(method_id) as usize,
    }];
    let desc = resolver.lookup_method_desc(method_id);
    if resolver.is_static(method_id) {
        res.push(IRInstr::VMExit2 {
            exit_type: IRVMExitType::RunStaticNativeNew {
                method_id,
            }
        });
    } else {
        res.push(IRInstr::VMExit2 { exit_type: IRVMExitType::RunSpecialNativeNew { method_id } });
    }
    res.push(IRInstr::Return {
        return_val: if desc.return_type.is_void() {
            None
        } else {
            Some(Register(0))//todo assert this always matches exit return register
        },
        temp_register_1: Register(1),
        temp_register_2: Register(2),
        temp_register_3: Register(3),
        temp_register_4: Register(4),
        frame_size: layout.full_frame_size(),
    });
    res
}

pub fn compile_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, labeler: &Labeler, method_frame_data: &JavaCompilerMethodAndFrameData, recompile_conditions: &mut MethodRecompileConditions, reserved_ir_method_id: IRMethodID) -> Vec<(ByteCodeOffset, IRInstr)> {
    let cinstructions = method_frame_data.layout.code_by_index.as_slice();
    let class_cpdtype = resolver.using_method_view_impl(method_frame_data.current_method_id, |method_view| {
        method_view.classview().type_()
    });
    let mut final_ir_without_labels: Vec<(ByteCodeOffset, IRInstr)> = vec![(ByteCodeOffset(0), IRInstr::IRStart {
        temp_register: Register(1),
        ir_method_id: reserved_ir_method_id,
        method_id: method_frame_data.current_method_id,
        frame_size: method_frame_data.full_frame_size(),
        num_locals: method_frame_data.layout.max_locals as usize,
    })];

    let mut compiler_labeler = CompilerLabeler {
        labeler,
        labels_vec: vec![],
        label_to_index: Default::default(),
        index_by_bytecode_offset: &method_frame_data.index_by_bytecode_offset,
    };
    let mut restart_point_generator = RestartPointGenerator::new();
    let mut prev_offset: Option<ByteCodeOffset> = None;

    if method_frame_data.should_synchronize {
        if method_frame_data.is_static {
            final_ir_without_labels.extend(repeat(ByteCodeOffset(0)).zip(monitor_enter_static(resolver, method_frame_data, &CurrentInstructionCompilerData {
                current_index: ByteCodeIndex(0),
                next_index: ByteCodeIndex(1),
                current_offset: ByteCodeOffset(0),
                compiler_labeler: &mut compiler_labeler,
            }, recompile_conditions, &mut restart_point_generator, class_cpdtype)))
        } else {
            final_ir_without_labels.push((ByteCodeOffset(0), IRInstr::VMExit2 {
                exit_type: IRVMExitType::MonitorEnter {
                    obj: method_frame_data.local_var_entry(ByteCodeIndex(0), 0),
                    java_pc: ByteCodeOffset(0),
                }
            }))
        }
    }

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
            if method_frame_data.should_trace_instructions {
                this_function_ir.push(IRInstr::VMExit2 { exit_type: IRVMExitType::TraceInstructionAfter { method_id: method_frame_data.current_method_id, offset: prev_offset, java_pc: current_instr_data.current_offset } });
            }
        }
        if method_frame_data.should_trace_instructions {
            this_function_ir.push(IRInstr::VMExit2 { exit_type: IRVMExitType::TraceInstructionBefore { method_id: method_frame_data.current_method_id, offset: current_offset, java_pc: current_instr_data.current_offset } });
        }
        match &compressed_instruction.info {
            CompressedInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => {
                this_function_ir.extend(invokestatic(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, recompile_conditions, *method_name, descriptor, classname_ref_type));
            }
            CompressedInstructionInfo::return_ => {
                synchronize_exit(resolver, method_frame_data, &current_instr_data, recompile_conditions, &mut restart_point_generator, class_cpdtype, &mut this_function_ir);
                this_function_ir.extend(return_void(method_frame_data));
            }
            CompressedInstructionInfo::ireturn => {
                synchronize_exit(resolver, method_frame_data, &current_instr_data, recompile_conditions, &mut restart_point_generator, class_cpdtype, &mut this_function_ir);
                this_function_ir.extend(ireturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::freturn => {
                synchronize_exit(resolver, method_frame_data, &current_instr_data, recompile_conditions, &mut restart_point_generator, class_cpdtype, &mut this_function_ir);
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
            CompressedInstructionInfo::fload_3 => {
                this_function_ir.extend(fload_n(method_frame_data, &current_instr_data, 3));
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
                this_function_ir.extend(goto_(current_instr_data, *offset as i32))
            }
            CompressedInstructionInfo::new(ccn) => {
                this_function_ir.extend(new(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, *ccn))
            }
            CompressedInstructionInfo::dup => {
                this_function_ir.extend(dup(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::putfield { name, desc: _, target_class } => {
                this_function_ir.extend(putfield(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, recompile_conditions, *target_class, *name))
            }
            CompressedInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => {
                this_function_ir.extend(invokespecial(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, recompile_conditions, *method_name, descriptor, *classname_ref_type))
            }
            CompressedInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type } => {
                this_function_ir.extend(invokevirtual(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, recompile_conditions, *method_name, descriptor, *classname_ref_type))
            }
            CompressedInstructionInfo::putstatic { name, desc, target_class } => {
                this_function_ir.extend(putstatic(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, *target_class, *name,*desc))
            }
            CompressedInstructionInfo::anewarray(elem_type) => {
                this_function_ir.extend(anewarray(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, elem_type))
            }
            CompressedInstructionInfo::ldc(either) => {
                match either {
                    Either::Left(left) => {
                        match left {
                            CompressedLdcW::String { str } => {
                                let compressed = resolver.get_commpressed_version_of_wtf8(str);
                                this_function_ir.extend(ldc_string(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, compressed))
                            }
                            CompressedLdcW::Class { type_ } => {
                                this_function_ir.extend(ldc_class(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, type_))
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
                this_function_ir.extend(getfield(resolver, method_frame_data, current_instr_data, &mut restart_point_generator, recompile_conditions, *target_class, *name))
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
            CompressedInstructionInfo::astore_3 => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::astore(index) => {
                this_function_ir.extend(astore_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::athrow => {
                //todo monitor exit
                this_function_ir.extend(athrow(method_frame_data, &current_instr_data));
            }
            CompressedInstructionInfo::areturn => {
                synchronize_exit(resolver, method_frame_data, &current_instr_data, recompile_conditions, &mut restart_point_generator, class_cpdtype, &mut this_function_ir);
                this_function_ir.extend(areturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::lreturn => {
                synchronize_exit(resolver, method_frame_data, &current_instr_data, recompile_conditions, &mut restart_point_generator, class_cpdtype, &mut this_function_ir);
                this_function_ir.extend(lreturn(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dreturn => {
                synchronize_exit(resolver, method_frame_data, &current_instr_data, recompile_conditions, &mut restart_point_generator, class_cpdtype, &mut this_function_ir);
                this_function_ir.extend(dreturn(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::iload_1 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::newarray(atype) => {
                this_function_ir.extend(newarray(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, atype))
            }
            CompressedInstructionInfo::i2l => {
                this_function_ir.extend(i2l(method_frame_data, &current_instr_data));
            }
            CompressedInstructionInfo::i2c => {
                this_function_ir.extend(i2c(method_frame_data, &current_instr_data));
            }
            CompressedInstructionInfo::i2s => {
                this_function_ir.extend(i2s(method_frame_data, &current_instr_data));
            }
            CompressedInstructionInfo::i2b => {
                this_function_ir.extend(i2b(method_frame_data, &current_instr_data));
            }
            CompressedInstructionInfo::l2i => {
                this_function_ir.extend(l2i(method_frame_data, &current_instr_data));
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
                this_function_ir.extend(sipush(method_frame_data, &current_instr_data, *val))
            }
            CompressedInstructionInfo::iload_0 => {
                this_function_ir.extend(iload_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::if_icmpgt(offset) => {
                this_function_ir.extend(if_icmp(method_frame_data, current_instr_data, IntEqualityType::GT, *offset as i32));
            }
            CompressedInstructionInfo::getstatic { name, desc, target_class } => {
                this_function_ir.extend(getstatic(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, *target_class, *name, *desc));
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
            CompressedInstructionInfo::lsub => {
                this_function_ir.extend(lsub(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::lcmp => {
                this_function_ir.extend(lcmp(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::bipush(val_) => {
                this_function_ir.extend(bipush(method_frame_data, current_instr_data, *val_))
            }
            CompressedInstructionInfo::lshl => {
                this_function_ir.extend(lshl(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::lshr => {
                this_function_ir.extend(lshr(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::land => {
                this_function_ir.extend(land(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::lor => {
                this_function_ir.extend(lor(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::istore(index) => {
                this_function_ir.extend(istore_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::istore_0 => {
                this_function_ir.extend(istore_n(method_frame_data, &current_instr_data, 0))
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
            CompressedInstructionInfo::fstore(index) => {
                this_function_ir.extend(fstore_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::fstore_0 => {
                this_function_ir.extend(fstore_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::fstore_1 => {
                this_function_ir.extend(fstore_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::fstore_2 => {
                this_function_ir.extend(fstore_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::fstore_3 => {
                this_function_ir.extend(fstore_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::dstore(n) => {
                this_function_ir.extend(dstore_n(method_frame_data, &current_instr_data, *n as u16))
            }
            CompressedInstructionInfo::dstore_0 => {
                this_function_ir.extend(dstore_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::dstore_1 => {
                this_function_ir.extend(dstore_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::dstore_2 => {
                this_function_ir.extend(dstore_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::dstore_3 => {
                this_function_ir.extend(dstore_n(method_frame_data, &current_instr_data, 3))
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
            CompressedInstructionInfo::lxor => {
                this_function_ir.extend(lxor(method_frame_data, current_instr_data))
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
            CompressedInstructionInfo::baload => {
                this_function_ir.extend(baload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::aaload => {
                this_function_ir.extend(aaload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::iaload => {
                this_function_ir.extend(iaload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::laload => {
                this_function_ir.extend(laload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::daload => {
                this_function_ir.extend(daload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::faload => {
                this_function_ir.extend(faload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::castore => {
                this_function_ir.extend(castore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::sastore => {
                this_function_ir.extend(sastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::saload => {
                this_function_ir.extend(saload(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::bastore => {
                this_function_ir.extend(bastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::iastore => {
                this_function_ir.extend(iastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::fastore => {
                this_function_ir.extend(fastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::lastore => {
                this_function_ir.extend(lastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dastore => {
                this_function_ir.extend(dastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::aastore => {
                this_function_ir.extend(aastore(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::instanceof(cpdtype) => {
                this_function_ir.extend(instanceof(resolver, &mut restart_point_generator, recompile_conditions, method_frame_data, &current_instr_data, *cpdtype))
            }
            CompressedInstructionInfo::checkcast(cpdtype) => {
                this_function_ir.extend(checkcast(resolver, recompile_conditions, &mut restart_point_generator, method_frame_data, current_instr_data, *cpdtype))
            }
            CompressedInstructionInfo::iinc(IInc { index, const_ }) => {
                this_function_ir.extend(iinc(method_frame_data, current_instr_data, *index, *const_))
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
            CompressedInstructionInfo::dconst_1 => {
                this_function_ir.extend(dconst(method_frame_data, current_instr_data, 1.0))
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
            CompressedInstructionInfo::lload_0 => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::lload_1 => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::lload_2 => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::lload_3 => {
                this_function_ir.extend(lload_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::dload(index) => {
                this_function_ir.extend(dload_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::dload_0 => {
                this_function_ir.extend(dload_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::dload_1 => {
                this_function_ir.extend(dload_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::dload_2 => {
                this_function_ir.extend(dload_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::dload_3 => {
                this_function_ir.extend(dload_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::fload(index) => {
                this_function_ir.extend(fload_n(method_frame_data, &current_instr_data, *index as u16))
            }
            CompressedInstructionInfo::invokeinterface { method_name, descriptor, classname_ref_type, count: _ } => {
                this_function_ir.extend(invoke_interface(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, method_name, descriptor, classname_ref_type))
            }
            CompressedInstructionInfo::pop => {
                this_function_ir.extend(array_into_iter([]))
            }
            CompressedInstructionInfo::pop2 => {
                this_function_ir.extend(array_into_iter([]))
            }
            CompressedInstructionInfo::iadd => {
                this_function_ir.extend(iadd(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::imul => {
                this_function_ir.extend(imul(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::lmul => {
                this_function_ir.extend(lmul(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dup_x1 => {
                this_function_ir.extend(dup_x1(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dup2 => {
                this_function_ir.extend(dup2(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dup2_x1 => {
                this_function_ir.extend(dup2_x1(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::fcmpg => {
                this_function_ir.extend(fcmpg(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fcmpl => {
                this_function_ir.extend(fcmpl(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::dcmpl => {
                this_function_ir.extend(dcmpl(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::dcmpg => {
                this_function_ir.extend(dcmpg(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::i2f => {
                this_function_ir.extend(i2f(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::l2f => {
                this_function_ir.extend(l2f(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::l2d => {
                this_function_ir.extend(l2d(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::i2d => {
                this_function_ir.extend(i2d(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::d2i => {
                this_function_ir.extend(d2i(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::d2l => {
                this_function_ir.extend(d2l(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::f2d => {
                this_function_ir.extend(f2d(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::d2f => {
                this_function_ir.extend(d2f(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::fmul => {
                this_function_ir.extend(fmul(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::dmul => {
                this_function_ir.extend(dmul(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fadd => {
                this_function_ir.extend(fadd(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fsub => {
                this_function_ir.extend(fsub(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::dsub => {
                this_function_ir.extend(dsub(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::dadd => {
                this_function_ir.extend(dadd(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::fdiv => {
                this_function_ir.extend(fdiv(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::frem => {
                this_function_ir.extend(frem(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::drem => {
                this_function_ir.extend(drem(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::ddiv => {
                this_function_ir.extend(ddiv(method_frame_data, &current_instr_data))
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
            CompressedInstructionInfo::lrem => {
                this_function_ir.extend(lrem(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::idiv => {
                this_function_ir.extend(idiv(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ldiv => {
                this_function_ir.extend(ldiv(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::ineg => {
                this_function_ir.extend(ineg(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::lneg => {
                this_function_ir.extend(lneg(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::fneg => {
                this_function_ir.extend(fneg(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::dneg => {
                this_function_ir.extend(dneg(method_frame_data, &current_instr_data))
            }
            CompressedInstructionInfo::lstore_0 => {
                this_function_ir.extend(lstore_n(method_frame_data, &current_instr_data, 0))
            }
            CompressedInstructionInfo::lstore_1 => {
                this_function_ir.extend(lstore_n(method_frame_data, &current_instr_data, 1))
            }
            CompressedInstructionInfo::lstore_2 => {
                this_function_ir.extend(lstore_n(method_frame_data, &current_instr_data, 2))
            }
            CompressedInstructionInfo::lstore_3 => {
                this_function_ir.extend(lstore_n(method_frame_data, &current_instr_data, 3))
            }
            CompressedInstructionInfo::lstore(n) => {
                this_function_ir.extend(lstore_n(method_frame_data, &current_instr_data, *n as u16))
            }
            CompressedInstructionInfo::ldc_w(elem) => {
                match elem {
                    CompressedLdcW::String { str } => {
                        this_function_ir.extend(ldc_string(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, resolver.get_commpressed_version_of_wtf8(str)))
                    }
                    CompressedLdcW::Class { type_ } => {
                        this_function_ir.extend(ldc_class(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, type_))
                    }
                    CompressedLdcW::Float { float } => {
                        this_function_ir.extend(ldc_float(method_frame_data, &current_instr_data, *float))
                    }
                    CompressedLdcW::Integer { integer } => {
                        this_function_ir.extend(ldc_integer(method_frame_data, &current_instr_data, *integer))
                    }
                    CompressedLdcW::MethodType { .. } => {
                        todo!()
                    }
                    CompressedLdcW::MethodHandle { .. } => {
                        todo!()
                    }
                    CompressedLdcW::LiveObject(_) => {
                        todo!()
                    }
                }
            }
            CompressedInstructionInfo::lookupswitch(LookupSwitch { pairs, default }) => {
                this_function_ir.extend(lookup_switch(method_frame_data, current_instr_data, pairs, default));
            }
            CompressedInstructionInfo::tableswitch(box TableSwitch { default, low, high, offsets }) => {
                this_function_ir.extend(tableswitch(method_frame_data, current_instr_data, default, low, high, offsets));
            }
            CompressedInstructionInfo::swap => {
                this_function_ir.extend(swap(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::lushr => {
                this_function_ir.extend(lushr(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::dup_x2 => {
                this_function_ir.extend(dup_x2(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::multianewarray { type_, dimensions } => {
                this_function_ir.extend(multianewarray(resolver, method_frame_data, &current_instr_data, &mut restart_point_generator, recompile_conditions, *type_, *dimensions));
            }
            CompressedInstructionInfo::wide(wide) => {
                match wide {
                    Wide::Iload(_) => todo!(),
                    Wide::Fload(_) => todo!(),
                    Wide::Aload(_) => todo!(),
                    Wide::Lload(_) => todo!(),
                    Wide::Dload(_) => todo!(),
                    Wide::Istore(_) => todo!(),
                    Wide::Fstore(_) => todo!(),
                    Wide::Astore(_) => todo!(),
                    Wide::Lstore(_) => todo!(),
                    Wide::Dstore(_) => todo!(),
                    Wide::Ret(_) => todo!(),
                    Wide::IInc(IInc { index, const_ }) => {
                        this_function_ir.extend(iinc(method_frame_data, current_instr_data, *index, *const_))
                    }
                }
            }
            CompressedInstructionInfo::f2l => {
                this_function_ir.extend(f2l(method_frame_data, current_instr_data))
            }
            CompressedInstructionInfo::invokedynamic(_) => {
                todo!()
            }
            CompressedInstructionInfo::dup2_x2 => {
                this_function_ir.extend(dup2_x2(method_frame_data, current_instr_data));
            }
            CompressedInstructionInfo::goto_w(_) => {
                todo!()
            }
            CompressedInstructionInfo::jsr(_) => {
                todo!()
            }
            CompressedInstructionInfo::jsr_w(_) => {
                todo!()
            }
            CompressedInstructionInfo::nop => {
                todo!()
            }
            CompressedInstructionInfo::ret(_) => {
                todo!()
            }
            CompressedInstructionInfo::EndOfCode => {
                todo!()
            }
        }
        final_ir_without_labels.extend(repeat(compressed_instruction.offset).zip(this_function_ir.into_iter()));
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

fn monitor_enter_static<'gc>(
    resolver: &impl MethodResolver<'gc>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    recompile_conditions: &mut MethodRecompileConditions,
    restart_point_generator: &mut RestartPointGenerator,
    type_: CPDType,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let to_load_cpdtype = type_.clone();
    let cpd_type_id = resolver.get_cpdtype_id(to_load_cpdtype);
    //todo we could do this in the exit and cut down on recompilations
    match resolver.lookup_type_inited_initing(&to_load_cpdtype) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: to_load_cpdtype });
            Either::Right(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                }
            }]))
        }
        Some((_loaded_class, _loader)) => {
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::NewClassRegister {
                    res: Register(1),
                    type_: cpd_type_id,
                    java_pc: current_instr_data.current_offset,
                }
            }, IRInstr::VMExit2 {
                exit_type: IRVMExitType::MonitorEnterRegister { obj: Register(1), java_pc: ByteCodeOffset(0) }
            }]))
        }
    }
}

fn monitor_exit_static<'gc>(
    resolver: &impl MethodResolver<'gc>,
    method_frame_data: &JavaCompilerMethodAndFrameData,
    current_instr_data: &CurrentInstructionCompilerData,
    recompile_conditions: &mut MethodRecompileConditions,
    restart_point_generator: &mut RestartPointGenerator,
    type_: CPDType,
) -> impl Iterator<Item=IRInstr> {
    let restart_point_id = restart_point_generator.new_restart_point();
    let restart_point = IRInstr::RestartPoint(restart_point_id);
    let to_load_cpdtype = type_.clone();
    let cpd_type_id = resolver.get_cpdtype_id(to_load_cpdtype);
    //todo we could do this in the exit and cut down on recompilations
    match resolver.lookup_type_inited_initing(&to_load_cpdtype) {
        None => {
            recompile_conditions.add_condition(NeedsRecompileIf::ClassLoaded { class: to_load_cpdtype });
            Either::Right(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::InitClassAndRecompile {
                    class: cpd_type_id,
                    this_method_id: method_frame_data.current_method_id,
                    restart_point_id,
                    java_pc: current_instr_data.current_offset,
                }
            }]))
        }
        Some((_loaded_class, _loader)) => {
            Either::Left(array_into_iter([restart_point, IRInstr::VMExit2 {
                exit_type: IRVMExitType::NewClassRegister {
                    res: Register(1),
                    type_: cpd_type_id,
                    java_pc: current_instr_data.current_offset,
                }
            }, IRInstr::VMExit2 {
                exit_type: IRVMExitType::MonitorExitRegister { obj: Register(1), java_pc: ByteCodeOffset(0) }
            }]))
        }
    }
}

fn synchronize_exit<'gc>(resolver: &impl MethodResolver<'gc>,
                         method_frame_data: &JavaCompilerMethodAndFrameData,
                         current_instr_data: &CurrentInstructionCompilerData,
                         recompile_conditions: &mut MethodRecompileConditions,
                         restart_point_generator: &mut RestartPointGenerator,
                         type_: CPDType, this_function_ir: &mut Vec<IRInstr>) {
    if method_frame_data.should_synchronize {
        if method_frame_data.is_static {
            this_function_ir.extend(monitor_exit_static(resolver, method_frame_data, current_instr_data, recompile_conditions, restart_point_generator, type_));
        } else {
            this_function_ir.push(IRInstr::VMExit2 { exit_type: IRVMExitType::MonitorExit { obj: method_frame_data.local_var_entry(ByteCodeIndex(0), 0), java_pc: ByteCodeOffset(0) } });
        }
    }
}

fn swap(method_frame_data: &JavaCompilerMethodAndFrameData, current_instr_data: CurrentInstructionCompilerData) -> impl Iterator<Item=IRInstr> {
    let value1_register = Register(1);
    let value2_register = Register(2);

    let iter = array_into_iter([
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 0), to: value1_register, size: Size::pointer() },
        IRInstr::LoadFPRelative { from: method_frame_data.operand_stack_entry(current_instr_data.current_index, 1), to: value2_register, size: Size::pointer() },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 0), from: value2_register, size: Size::pointer() },
        IRInstr::StoreFPRelative { to: method_frame_data.operand_stack_entry(current_instr_data.next_index, 1), from: value1_register, size: Size::pointer() },
    ]);
    iter
}

pub mod intrinsics;
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
pub mod int_convert;

pub fn array_into_iter<T, const N: usize>(array: [T; N]) -> impl Iterator<Item=T> {
    <[T; N]>::into_iter(array)
}