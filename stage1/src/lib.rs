#![feature(const_option)]
use another_jit_vm::{IRMethodID};
use compiler_common::{JavaCompilerMethodAndFrameData, MethodResolver};
use rust_jvm_common::{ByteCodeIndex, ByteCodeOffset, MethodId};
use rust_jvm_common::compressed_classfile::code::{CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::ir_compiler_common::{array_load_impl, Stage1IRInstr};
use crate::ir_compiler_common::branching::IntegerCompareKind;
use crate::ir_compiler_common::special::IRCompilerState;

//todo fix instanceof/checkcast
//todo fix class loaders
//todo make a get object class fast path

//todo maybe an r15 offset consts makes sense here as well
pub mod native_compiler_common;
pub mod ir_compiler_common;
pub mod frame_layout;

pub fn compile_to_ir<'vm>(resolver: &impl MethodResolver<'vm>, method_frame_data: &JavaCompilerMethodAndFrameData, method_id: MethodId, ir_method_id: IRMethodID) -> Vec<Stage1IRInstr> {
    //todo use ir emit functions
    let mut compiler_state = IRCompilerState::new(method_id, ir_method_id, method_frame_data, false);
    compiler_state.emit_ir_start();
    if method_frame_data.should_synchronize {
        if method_frame_data.is_static {
            let class_object = compiler_state.emit_get_class_object();
            compiler_state.emit_monitor_enter(class_object);
        } else {
            let this_object = compiler_state.emit_load_arg_pointer(0);
            compiler_state.emit_monitor_enter(this_object);
        }
    }
    let code = resolver.get_compressed_code(method_id);
    for (i,(java_pc, instr)) in code.instructions.iter().enumerate() {
        compiler_state.notify_before_instruction(*java_pc, ByteCodeIndex(i as u16));
        emit_single_instruction(&mut compiler_state, instr);
        compiler_state.notify_after_instruction(*java_pc);
    }
    //todo returns need to handle monitor_enter_exit
    compiler_state.complete()
}

fn emit_single_instruction(mut compiler_state: &mut IRCompilerState, instr: &CompressedInstruction) {
    match instr.info {
        CompressedInstructionInfo::aaload => {
            array_load_impl(compiler_state,CPDType::object())
        }
        CompressedInstructionInfo::aastore => {
            todo!()
        }
        CompressedInstructionInfo::aconst_null => {
            todo!()
        }
        CompressedInstructionInfo::aload(_) => {
            todo!()
        }
        CompressedInstructionInfo::aload_0 => {
            todo!()
        }
        CompressedInstructionInfo::aload_1 => {
            todo!()
        }
        CompressedInstructionInfo::aload_2 => {
            todo!()
        }
        CompressedInstructionInfo::aload_3 => {
            todo!()
        }
        CompressedInstructionInfo::anewarray(_) => {
            todo!()
        }
        CompressedInstructionInfo::areturn => {
            todo!()
        }
        CompressedInstructionInfo::arraylength => {
            todo!()
        }
        CompressedInstructionInfo::astore(_) => {
            todo!()
        }
        CompressedInstructionInfo::astore_0 => {
            todo!()
        }
        CompressedInstructionInfo::astore_1 => {
            todo!()
        }
        CompressedInstructionInfo::astore_2 => {
            todo!()
        }
        CompressedInstructionInfo::astore_3 => {
            todo!()
        }
        CompressedInstructionInfo::athrow => {
            todo!()
        }
        CompressedInstructionInfo::baload => {
            todo!()
        }
        CompressedInstructionInfo::bastore => {
            todo!()
        }
        CompressedInstructionInfo::bipush(_) => {
            todo!()
        }
        CompressedInstructionInfo::caload => {
            todo!()
        }
        CompressedInstructionInfo::castore => {
            todo!()
        }
        CompressedInstructionInfo::checkcast(_) => {
            todo!()
        }
        CompressedInstructionInfo::d2f => {
            todo!()
        }
        CompressedInstructionInfo::d2i => {
            todo!()
        }
        CompressedInstructionInfo::d2l => {
            todo!()
        }
        CompressedInstructionInfo::dadd => {
            todo!()
        }
        CompressedInstructionInfo::daload => {
            todo!()
        }
        CompressedInstructionInfo::dastore => {
            todo!()
        }
        CompressedInstructionInfo::dcmpg => {
            todo!()
        }
        CompressedInstructionInfo::dcmpl => {
            todo!()
        }
        CompressedInstructionInfo::dconst_0 => {
            todo!()
        }
        CompressedInstructionInfo::dconst_1 => {
            todo!()
        }
        CompressedInstructionInfo::ddiv => {
            todo!()
        }
        CompressedInstructionInfo::dload(_) => {
            todo!()
        }
        CompressedInstructionInfo::dload_0 => {
            todo!()
        }
        CompressedInstructionInfo::dload_1 => {
            todo!()
        }
        CompressedInstructionInfo::dload_2 => {
            todo!()
        }
        CompressedInstructionInfo::dload_3 => {
            todo!()
        }
        CompressedInstructionInfo::dmul => {
            todo!()
        }
        CompressedInstructionInfo::dneg => {
            todo!()
        }
        CompressedInstructionInfo::drem => {
            todo!()
        }
        CompressedInstructionInfo::dreturn => {
            todo!()
        }
        CompressedInstructionInfo::dstore(_) => {
            todo!()
        }
        CompressedInstructionInfo::dstore_0 => {
            todo!()
        }
        CompressedInstructionInfo::dstore_1 => {
            todo!()
        }
        CompressedInstructionInfo::dstore_2 => {
            todo!()
        }
        CompressedInstructionInfo::dstore_3 => {
            todo!()
        }
        CompressedInstructionInfo::dsub => {
            todo!()
        }
        CompressedInstructionInfo::dup => {
            todo!()
        }
        CompressedInstructionInfo::dup_x1 => {
            todo!()
        }
        CompressedInstructionInfo::dup_x2 => {
            todo!()
        }
        CompressedInstructionInfo::dup2 => {
            todo!()
        }
        CompressedInstructionInfo::dup2_x1 => {
            todo!()
        }
        CompressedInstructionInfo::dup2_x2 => {
            todo!()
        }
        CompressedInstructionInfo::f2d => {
            todo!()
        }
        CompressedInstructionInfo::f2i => {
            todo!()
        }
        CompressedInstructionInfo::f2l => {
            todo!()
        }
        CompressedInstructionInfo::fadd => {
            todo!()
        }
        CompressedInstructionInfo::faload => {
            todo!()
        }
        CompressedInstructionInfo::fastore => {
            todo!()
        }
        CompressedInstructionInfo::fcmpg => {
            todo!()
        }
        CompressedInstructionInfo::fcmpl => {
            todo!()
        }
        CompressedInstructionInfo::fconst_0 => {
            todo!()
        }
        CompressedInstructionInfo::fconst_1 => {
            todo!()
        }
        CompressedInstructionInfo::fconst_2 => {
            todo!()
        }
        CompressedInstructionInfo::fdiv => {
            todo!()
        }
        CompressedInstructionInfo::fload(_) => {
            todo!()
        }
        CompressedInstructionInfo::fload_0 => {
            todo!()
        }
        CompressedInstructionInfo::fload_1 => {
            todo!()
        }
        CompressedInstructionInfo::fload_2 => {
            todo!()
        }
        CompressedInstructionInfo::fload_3 => {
            todo!()
        }
        CompressedInstructionInfo::fmul => {
            todo!()
        }
        CompressedInstructionInfo::fneg => {
            todo!()
        }
        CompressedInstructionInfo::frem => {
            todo!()
        }
        CompressedInstructionInfo::freturn => {
            todo!()
        }
        CompressedInstructionInfo::fstore(_) => {
            todo!()
        }
        CompressedInstructionInfo::fstore_0 => {
            todo!()
        }
        CompressedInstructionInfo::fstore_1 => {
            todo!()
        }
        CompressedInstructionInfo::fstore_2 => {
            todo!()
        }
        CompressedInstructionInfo::fstore_3 => {
            todo!()
        }
        CompressedInstructionInfo::fsub => {
            todo!()
        }
        CompressedInstructionInfo::getfield { .. } => {
            todo!()
        }
        CompressedInstructionInfo::getstatic { .. } => {
            todo!()
        }
        CompressedInstructionInfo::goto_(_) => {
            todo!()
        }
        CompressedInstructionInfo::goto_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::i2b => {
            todo!()
        }
        CompressedInstructionInfo::i2c => {
            todo!()
        }
        CompressedInstructionInfo::i2d => {
            todo!()
        }
        CompressedInstructionInfo::i2f => {
            todo!()
        }
        CompressedInstructionInfo::i2l => {
            todo!()
        }
        CompressedInstructionInfo::i2s => {
            todo!()
        }
        CompressedInstructionInfo::iadd => {
            let a = compiler_state.emit_stack_load_int(0);
            let b = compiler_state.emit_stack_load_int(1);
            let res = compiler_state.emit_add_integer(a, b);
            compiler_state.emit_stack_store_int(0, res);
        }
        CompressedInstructionInfo::iaload => {
            todo!()
        }
        CompressedInstructionInfo::iand => {
            todo!()
        }
        CompressedInstructionInfo::iastore => {
            todo!()
        }
        CompressedInstructionInfo::iconst_m1 => {
            todo!()
        }
        CompressedInstructionInfo::iconst_0 => {
            todo!()
        }
        CompressedInstructionInfo::iconst_1 => {
            todo!()
        }
        CompressedInstructionInfo::iconst_2 => {
            todo!()
        }
        CompressedInstructionInfo::iconst_3 => {
            todo!()
        }
        CompressedInstructionInfo::iconst_4 => {
            todo!()
        }
        CompressedInstructionInfo::iconst_5 => {
            todo!()
        }
        CompressedInstructionInfo::idiv => {
            todo!()
        }
        CompressedInstructionInfo::if_acmpeq(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_acmpne(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpeq(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpne(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmplt(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpge(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpgt(_) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmple(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifeq(offset) => {
            emit_integer_if(compiler_state, instr, offset, IntegerCompareKind::Equal);
        }
        CompressedInstructionInfo::ifne(offset) => {
            emit_integer_if(compiler_state, instr, offset, IntegerCompareKind::NotEqual);
        }
        CompressedInstructionInfo::iflt(offset) => {
            emit_integer_if(compiler_state, instr, offset, IntegerCompareKind::LessThan);
        }
        CompressedInstructionInfo::ifge(offset) => {
            emit_integer_if(compiler_state, instr, offset, IntegerCompareKind::GreaterThanEqual);
        }
        CompressedInstructionInfo::ifgt(offset) => {
            emit_integer_if(compiler_state, instr, offset, IntegerCompareKind::GreaterThan);
        }
        CompressedInstructionInfo::ifle(offset) => {
            emit_integer_if(compiler_state, instr, offset, IntegerCompareKind::LessThanEqual);
        }
        CompressedInstructionInfo::ifnonnull(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifnull(_) => {
            todo!()
        }
        CompressedInstructionInfo::iinc(_) => {
            todo!()
        }
        CompressedInstructionInfo::iload(_) => {
            todo!()
        }
        CompressedInstructionInfo::iload_0 => {
            todo!()
        }
        CompressedInstructionInfo::iload_1 => {
            todo!()
        }
        CompressedInstructionInfo::iload_2 => {
            todo!()
        }
        CompressedInstructionInfo::iload_3 => {
            todo!()
        }
        CompressedInstructionInfo::imul => {
            todo!()
        }
        CompressedInstructionInfo::ineg => {
            todo!()
        }
        CompressedInstructionInfo::instanceof(_) => {
            todo!()
        }
        CompressedInstructionInfo::invokedynamic(_) => {
            todo!()
        }
        CompressedInstructionInfo::invokeinterface { .. } => {
            todo!()
        }
        CompressedInstructionInfo::invokespecial { .. } => {
            todo!()
        }
        CompressedInstructionInfo::invokestatic { .. } => {
            todo!()
        }
        CompressedInstructionInfo::invokevirtual { .. } => {
            todo!()
        }
        CompressedInstructionInfo::ior => {
            todo!()
        }
        CompressedInstructionInfo::irem => {
            todo!()
        }
        CompressedInstructionInfo::ireturn => {
            todo!()
        }
        CompressedInstructionInfo::ishl => {
            todo!()
        }
        CompressedInstructionInfo::ishr => {
            todo!()
        }
        CompressedInstructionInfo::istore(_) => {
            todo!()
        }
        CompressedInstructionInfo::istore_0 => {
            todo!()
        }
        CompressedInstructionInfo::istore_1 => {
            todo!()
        }
        CompressedInstructionInfo::istore_2 => {
            todo!()
        }
        CompressedInstructionInfo::istore_3 => {
            todo!()
        }
        CompressedInstructionInfo::isub => {
            todo!()
        }
        CompressedInstructionInfo::iushr => {
            todo!()
        }
        CompressedInstructionInfo::ixor => {
            todo!()
        }
        CompressedInstructionInfo::jsr(_) => {
            todo!()
        }
        CompressedInstructionInfo::jsr_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::l2d => {
            todo!()
        }
        CompressedInstructionInfo::l2f => {
            todo!()
        }
        CompressedInstructionInfo::l2i => {
            todo!()
        }
        CompressedInstructionInfo::ladd => {
            todo!()
        }
        CompressedInstructionInfo::laload => {
            todo!()
        }
        CompressedInstructionInfo::land => {
            todo!()
        }
        CompressedInstructionInfo::lastore => {
            todo!()
        }
        CompressedInstructionInfo::lcmp => {
            todo!()
        }
        CompressedInstructionInfo::lconst_0 => {
            todo!()
        }
        CompressedInstructionInfo::lconst_1 => {
            todo!()
        }
        CompressedInstructionInfo::ldc(_) => {
            todo!()
        }
        CompressedInstructionInfo::ldc_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::ldc2_w(_) => {
            todo!()
        }
        CompressedInstructionInfo::ldiv => {
            todo!()
        }
        CompressedInstructionInfo::lload(_) => {
            todo!()
        }
        CompressedInstructionInfo::lload_0 => {
            todo!()
        }
        CompressedInstructionInfo::lload_1 => {
            todo!()
        }
        CompressedInstructionInfo::lload_2 => {
            todo!()
        }
        CompressedInstructionInfo::lload_3 => {
            todo!()
        }
        CompressedInstructionInfo::lmul => {
            todo!()
        }
        CompressedInstructionInfo::lneg => {
            todo!()
        }
        CompressedInstructionInfo::lookupswitch(_) => {
            todo!()
        }
        CompressedInstructionInfo::lor => {
            todo!()
        }
        CompressedInstructionInfo::lrem => {
            todo!()
        }
        CompressedInstructionInfo::lreturn => {
            todo!()
        }
        CompressedInstructionInfo::lshl => {
            todo!()
        }
        CompressedInstructionInfo::lshr => {
            todo!()
        }
        CompressedInstructionInfo::lstore(_) => {
            todo!()
        }
        CompressedInstructionInfo::lstore_0 => {
            todo!()
        }
        CompressedInstructionInfo::lstore_1 => {
            todo!()
        }
        CompressedInstructionInfo::lstore_2 => {
            todo!()
        }
        CompressedInstructionInfo::lstore_3 => {
            todo!()
        }
        CompressedInstructionInfo::lsub => {
            todo!()
        }
        CompressedInstructionInfo::lushr => {
            todo!()
        }
        CompressedInstructionInfo::lxor => {
            todo!()
        }
        CompressedInstructionInfo::monitorenter => {
            todo!()
        }
        CompressedInstructionInfo::monitorexit => {
            todo!()
        }
        CompressedInstructionInfo::multianewarray { .. } => {
            todo!()
        }
        CompressedInstructionInfo::new(_) => {
            todo!()
        }
        CompressedInstructionInfo::newarray(_) => {
            todo!()
        }
        CompressedInstructionInfo::nop => {
            todo!()
        }
        CompressedInstructionInfo::pop => {
            todo!()
        }
        CompressedInstructionInfo::pop2 => {
            todo!()
        }
        CompressedInstructionInfo::putfield { .. } => {
            todo!()
        }
        CompressedInstructionInfo::putstatic { .. } => {
            todo!()
        }
        CompressedInstructionInfo::ret(_) => {
            todo!()
        }
        CompressedInstructionInfo::return_ => {
            todo!()
        }
        CompressedInstructionInfo::saload => {
            todo!()
        }
        CompressedInstructionInfo::sastore => {
            todo!()
        }
        CompressedInstructionInfo::sipush(_) => {
            todo!()
        }
        CompressedInstructionInfo::swap => {
            todo!()
        }
        CompressedInstructionInfo::tableswitch(_) => {
            todo!()
        }
        CompressedInstructionInfo::wide(_) => {
            todo!()
        }
        CompressedInstructionInfo::EndOfCode => {
            todo!()
        }
    }
}

fn emit_integer_if(compiler_state: &mut IRCompilerState, instr: &CompressedInstruction, offset: i16, integer_compare_kind: IntegerCompareKind) {
    let (branch_to, label_target) = compiler_state.create_label();
    let target_offset = ByteCodeOffset((instr.offset.0 as i32 + offset as i32) as u16);
    compiler_state.set_label_target_pending(target_offset, label_target);
    let zero = compiler_state.emit_constant_int(0);
    let value = compiler_state.emit_stack_load_int(0);
    compiler_state.emit_branch_compare_int(branch_to, value, zero, integer_compare_kind);
}


pub struct CompilerState {}

impl CompilerState {
    pub fn new() -> Self {
        Self {}
    }
}