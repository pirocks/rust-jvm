use rust_jvm_common::classfile::IInc;
use rust_jvm_common::compressed_classfile::code::{CompressedInstruction, CompressedInstructionInfo};
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use crate::ir_compiler_common::branching::IntegerCompareKind;
use crate::ir_compiler_common::special::IRCompilerState;
use crate::java_compiler::arithmetic::emit_iadd;
use crate::java_compiler::array_load::array_load_impl;
use crate::java_compiler::branching::{emit_integer_if_compare_two_values, emit_integer_if_compare_with_zero};
use crate::java_compiler::constants::emit_iconst;
use crate::java_compiler::local_var::{emit_aload_n, emit_iload_n};
use crate::java_compiler::return_::emit_ireturn;

pub(crate) fn emit_single_instruction(compiler_state: &mut IRCompilerState, instr: &CompressedInstruction) {
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
        CompressedInstructionInfo::aload(n) => {
            emit_aload_n(compiler_state, n as u16);
        }
        CompressedInstructionInfo::aload_0 => {
            emit_aload_n(compiler_state, 0);
        }
        CompressedInstructionInfo::aload_1 => {
            emit_aload_n(compiler_state, 1);
        }
        CompressedInstructionInfo::aload_2 => {
            emit_aload_n(compiler_state, 2);
        }
        CompressedInstructionInfo::aload_3 => {
            emit_aload_n(compiler_state, 3);
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
        CompressedInstructionInfo::getfield { name: _, desc: _, target_class: _ } => {
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
            emit_iadd(compiler_state);
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
            emit_iconst(compiler_state, -1);
        }
        CompressedInstructionInfo::iconst_0 => {
            emit_iconst(compiler_state, 0);
        }
        CompressedInstructionInfo::iconst_1 => {
            emit_iconst(compiler_state, 1);
        }
        CompressedInstructionInfo::iconst_2 => {
            emit_iconst(compiler_state, 2);
        }
        CompressedInstructionInfo::iconst_3 => {
            emit_iconst(compiler_state, 3);
        }
        CompressedInstructionInfo::iconst_4 => {
            emit_iconst(compiler_state, 4);
        }
        CompressedInstructionInfo::iconst_5 => {
            emit_iconst(compiler_state, 5);
        }
        CompressedInstructionInfo::idiv => {
            todo!()
        }
        CompressedInstructionInfo::if_acmpeq(_offset) => {
            todo!()
        }
        CompressedInstructionInfo::if_acmpne(_offset) => {
            todo!()
        }
        CompressedInstructionInfo::if_icmpeq(offset) => {
            emit_integer_if_compare_two_values(compiler_state, instr, offset, IntegerCompareKind::Equal);
        }
        CompressedInstructionInfo::if_icmpne(offset) => {
            emit_integer_if_compare_two_values(compiler_state, instr, offset, IntegerCompareKind::NotEqual);
        }
        CompressedInstructionInfo::if_icmplt(offset) => {
            emit_integer_if_compare_two_values(compiler_state, instr, offset, IntegerCompareKind::LessThan);
        }
        CompressedInstructionInfo::if_icmpge(offset) => {
            emit_integer_if_compare_two_values(compiler_state, instr, offset, IntegerCompareKind::GreaterThanEqual);
        }
        CompressedInstructionInfo::if_icmpgt(offset) => {
            emit_integer_if_compare_two_values(compiler_state, instr, offset, IntegerCompareKind::GreaterThan);
        }
        CompressedInstructionInfo::if_icmple(offset) => {
            emit_integer_if_compare_two_values(compiler_state, instr, offset, IntegerCompareKind::LessThanEqual);
        }
        CompressedInstructionInfo::ifeq(offset) => {
            emit_integer_if_compare_with_zero(compiler_state, instr, offset, IntegerCompareKind::Equal);
        }
        CompressedInstructionInfo::ifne(offset) => {
            emit_integer_if_compare_with_zero(compiler_state, instr, offset, IntegerCompareKind::NotEqual);
        }
        CompressedInstructionInfo::iflt(offset) => {
            emit_integer_if_compare_with_zero(compiler_state, instr, offset, IntegerCompareKind::LessThan);
        }
        CompressedInstructionInfo::ifge(offset) => {
            emit_integer_if_compare_with_zero(compiler_state, instr, offset, IntegerCompareKind::GreaterThanEqual);
        }
        CompressedInstructionInfo::ifgt(offset) => {
            emit_integer_if_compare_with_zero(compiler_state, instr, offset, IntegerCompareKind::GreaterThan);
        }
        CompressedInstructionInfo::ifle(offset) => {
            emit_integer_if_compare_with_zero(compiler_state, instr, offset, IntegerCompareKind::LessThanEqual);
        }
        CompressedInstructionInfo::ifnonnull(_) => {
            todo!()
        }
        CompressedInstructionInfo::ifnull(_) => {
            todo!()
        }
        CompressedInstructionInfo::iinc(IInc{ index: _, const_: _ }) => {
            todo!()
        }
        CompressedInstructionInfo::iload(n) => {
            emit_iload_n(compiler_state, n as u16);
        }
        CompressedInstructionInfo::iload_0 => {
            emit_iload_n(compiler_state, 0);
        }
        CompressedInstructionInfo::iload_1 => {
            emit_iload_n(compiler_state, 1);
        }
        CompressedInstructionInfo::iload_2 => {
            emit_iload_n(compiler_state, 2);
        }
        CompressedInstructionInfo::iload_3 => {
            emit_iload_n(compiler_state, 3);
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
            emit_ireturn(compiler_state);
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


pub mod return_;
pub mod local_var;
pub mod constants;
pub mod array_load;
pub mod branching;
pub mod arithmetic;
