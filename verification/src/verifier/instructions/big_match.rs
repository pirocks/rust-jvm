use rust_jvm_common::classfile::{InstructionInfo, Instruction};
use crate::verifier::instructions::loads::{instruction_is_type_safe_aload, instruction_is_type_safe_lload};
use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::InstructionIsTypeSafeResult;
use crate::verifier::instructions::branches::{instruction_is_type_safe_goto, instruction_is_type_safe_invokespecial};
use crate::verifier::instructions::branches::instruction_is_type_safe_if_acmpeq;
use crate::verifier::instructions::branches::instruction_is_type_safe_invokestatic;
use crate::verifier::instructions::branches::instruction_is_type_safe_ireturn;
use crate::verifier::instructions::instruction_is_type_safe_lcmp;
use crate::verifier::instructions::consts::instruction_is_type_safe_lconst_0;
use crate::verifier::instructions::branches::instruction_is_type_safe_return;
use crate::verifier::instructions::consts::instruction_is_type_safe_iconst_m1;
use crate::verifier::instructions::branches::instruction_is_type_safe_invokevirtual;
use crate::verifier::instructions::special::instruction_is_type_safe_putfield;

pub fn instruction_is_type_safe(instruction: &Instruction, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionIsTypeSafeResult, TypeSafetyError> {
    match instruction.instruction {
        InstructionInfo::aaload => { unimplemented!() }
        InstructionInfo::aastore => { unimplemented!() }
        InstructionInfo::aconst_null => { unimplemented!() }
        InstructionInfo::aload(_) => { unimplemented!() }
        InstructionInfo::aload_0 => instruction_is_type_safe_aload(0, env, offset, stack_frame),
        InstructionInfo::aload_1 => instruction_is_type_safe_aload(1, env, offset, stack_frame),
        InstructionInfo::aload_2 => instruction_is_type_safe_aload(2, env, offset, stack_frame),
        InstructionInfo::aload_3 => instruction_is_type_safe_aload(3, env, offset, stack_frame),
        InstructionInfo::anewarray(_) => { unimplemented!() }
        InstructionInfo::areturn => { unimplemented!() }
        InstructionInfo::arraylength => { unimplemented!() }
        InstructionInfo::astore(_) => { unimplemented!() }
        InstructionInfo::astore_0 => { unimplemented!() }
        InstructionInfo::astore_1 => { unimplemented!() }
        InstructionInfo::astore_2 => { unimplemented!() }
        InstructionInfo::astore_3 => { unimplemented!() }
        InstructionInfo::athrow => { unimplemented!() }
        InstructionInfo::baload => { unimplemented!() }
        InstructionInfo::bastore => { unimplemented!() }
        InstructionInfo::bipush(_) => { unimplemented!() }
        InstructionInfo::caload => { unimplemented!() }
        InstructionInfo::castore => { unimplemented!() }
        InstructionInfo::checkcast(_) => { unimplemented!() }
        InstructionInfo::d2f => { unimplemented!() }
        InstructionInfo::d2i => { unimplemented!() }
        InstructionInfo::d2l => { unimplemented!() }
        InstructionInfo::dadd => { unimplemented!() }
        InstructionInfo::daload => { unimplemented!() }
        InstructionInfo::dastore => { unimplemented!() }
        InstructionInfo::dcmpg => { unimplemented!() }
        InstructionInfo::dcmpl => { unimplemented!() }
        InstructionInfo::dconst_0 => { unimplemented!() }
        InstructionInfo::dconst_1 => { unimplemented!() }
        InstructionInfo::ddiv => { unimplemented!() }
        InstructionInfo::dload(_) => { unimplemented!() }
        InstructionInfo::dload_0 => { unimplemented!() }
        InstructionInfo::dload_1 => { unimplemented!() }
        InstructionInfo::dload_2 => { unimplemented!() }
        InstructionInfo::dload_3 => { unimplemented!() }
        InstructionInfo::dmul => { unimplemented!() }
        InstructionInfo::dneg => { unimplemented!() }
        InstructionInfo::drem => { unimplemented!() }
        InstructionInfo::dreturn => { unimplemented!() }
        InstructionInfo::dstore(_) => { unimplemented!() }
        InstructionInfo::dstore_0 => { unimplemented!() }
        InstructionInfo::dstore_1 => { unimplemented!() }
        InstructionInfo::dstore_2 => { unimplemented!() }
        InstructionInfo::dstore_3 => { unimplemented!() }
        InstructionInfo::dsub => { unimplemented!() }
        InstructionInfo::dup => { unimplemented!() }
        InstructionInfo::dup_x1 => { unimplemented!() }
        InstructionInfo::dup_x2 => { unimplemented!() }
        InstructionInfo::dup2 => { unimplemented!() }
        InstructionInfo::dup2_x1 => { unimplemented!() }
        InstructionInfo::dup2_x2 => { unimplemented!() }
        InstructionInfo::f2d => { unimplemented!() }
        InstructionInfo::f2i => { unimplemented!() }
        InstructionInfo::f2l => { unimplemented!() }
        InstructionInfo::fadd => { unimplemented!() }
        InstructionInfo::faload => { unimplemented!() }
        InstructionInfo::fastore => { unimplemented!() }
        InstructionInfo::fcmpg => { unimplemented!() }
        InstructionInfo::fcmpl => { unimplemented!() }
        InstructionInfo::fconst_0 => { unimplemented!() }
        InstructionInfo::fconst_1 => { unimplemented!() }
        InstructionInfo::fconst_2 => { unimplemented!() }
        InstructionInfo::fdiv => { unimplemented!() }
        InstructionInfo::fload(_) => { unimplemented!() }
        InstructionInfo::fload_0 => { unimplemented!() }
        InstructionInfo::fload_1 => { unimplemented!() }
        InstructionInfo::fload_2 => { unimplemented!() }
        InstructionInfo::fload_3 => { unimplemented!() }
        InstructionInfo::fmul => { unimplemented!() }
        InstructionInfo::fneg => { unimplemented!() }
        InstructionInfo::frem => { unimplemented!() }
        InstructionInfo::freturn => { unimplemented!() }
        InstructionInfo::fstore(_) => { unimplemented!() }
        InstructionInfo::fstore_0 => { unimplemented!() }
        InstructionInfo::fstore_1 => { unimplemented!() }
        InstructionInfo::fstore_2 => { unimplemented!() }
        InstructionInfo::fstore_3 => { unimplemented!() }
        InstructionInfo::fsub => { unimplemented!() }
        InstructionInfo::getfield(_) => { unimplemented!() }
        InstructionInfo::getstatic(_) => { unimplemented!() }
        InstructionInfo::goto_(target) => {
            let final_target = (target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_goto(final_target as usize, env, offset, stack_frame)
        }
        InstructionInfo::goto_w(_) => { unimplemented!() }
        InstructionInfo::i2b => { unimplemented!() }
        InstructionInfo::i2c => { unimplemented!() }
        InstructionInfo::i2d => { unimplemented!() }
        InstructionInfo::i2f => { unimplemented!() }
        InstructionInfo::i2l => { unimplemented!() }
        InstructionInfo::i2s => { unimplemented!() }
        InstructionInfo::iadd => { unimplemented!() }
        InstructionInfo::iaload => { unimplemented!() }
        InstructionInfo::iand => { unimplemented!() }
        InstructionInfo::iastore => { unimplemented!() }
        InstructionInfo::iconst_m1 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_0 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_1 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_2 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_3 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_4 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_5 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::idiv => { unimplemented!() }
        InstructionInfo::if_acmpeq(_) => { unimplemented!() }
        InstructionInfo::if_acmpne(target) => {
            let final_target = (target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as usize, env, offset, stack_frame)//same as eq case
        }
        InstructionInfo::if_icmpeq(_) => { unimplemented!() }
        InstructionInfo::if_icmpne(_) => { unimplemented!() }
        InstructionInfo::if_icmplt(_) => { unimplemented!() }
        InstructionInfo::if_icmpge(_) => { unimplemented!() }
        InstructionInfo::if_icmpgt(_) => { unimplemented!() }
        InstructionInfo::if_icmple(_) => { unimplemented!() }
        InstructionInfo::ifeq(_) => { unimplemented!() }
        InstructionInfo::ifne(_) => { unimplemented!() }
        InstructionInfo::iflt(_) => { unimplemented!() }
        InstructionInfo::ifge(_) => { unimplemented!() }
        InstructionInfo::ifgt(_) => { unimplemented!() }
        InstructionInfo::ifle(_) => { unimplemented!() }
        InstructionInfo::ifnonnull(_) => { unimplemented!() }
        InstructionInfo::ifnull(_) => { unimplemented!() }
        InstructionInfo::iinc(_) => { unimplemented!() }
        InstructionInfo::iload(_) => { unimplemented!() }
        InstructionInfo::iload_0 => { unimplemented!() }
        InstructionInfo::iload_1 => { unimplemented!() }
        InstructionInfo::iload_2 => { unimplemented!() }
        InstructionInfo::iload_3 => { unimplemented!() }
        InstructionInfo::imul => { unimplemented!() }
        InstructionInfo::ineg => { unimplemented!() }
        InstructionInfo::instanceof(_) => { unimplemented!() }
        InstructionInfo::invokedynamic(_) => { unimplemented!() }
        InstructionInfo::invokeinterface(_) => { unimplemented!() }
        InstructionInfo::invokespecial(cp) => instruction_is_type_safe_invokespecial(cp as usize, env, offset, stack_frame),
        InstructionInfo::invokestatic(cp) => instruction_is_type_safe_invokestatic(cp as usize, env, offset, stack_frame),
        InstructionInfo::invokevirtual(v) => instruction_is_type_safe_invokevirtual(v as usize, env, offset, stack_frame),
        InstructionInfo::ior => { unimplemented!() }
        InstructionInfo::irem => { unimplemented!() }
        InstructionInfo::ireturn => instruction_is_type_safe_ireturn(env, offset, stack_frame),
        InstructionInfo::ishl => { unimplemented!() }
        InstructionInfo::ishr => { unimplemented!() }
        InstructionInfo::istore(_) => { unimplemented!() }
        InstructionInfo::istore_0 => { unimplemented!() }
        InstructionInfo::istore_1 => { unimplemented!() }
        InstructionInfo::istore_2 => { unimplemented!() }
        InstructionInfo::istore_3 => { unimplemented!() }
        InstructionInfo::isub => { unimplemented!() }
        InstructionInfo::iushr => { unimplemented!() }
        InstructionInfo::ixor => { unimplemented!() }
        InstructionInfo::jsr(_) => { unimplemented!() }
        InstructionInfo::jsr_w(_) => { unimplemented!() }
        InstructionInfo::l2d => { unimplemented!() }
        InstructionInfo::l2f => { unimplemented!() }
        InstructionInfo::l2i => { unimplemented!() }
        InstructionInfo::ladd => { unimplemented!() }
        InstructionInfo::laload => { unimplemented!() }
        InstructionInfo::land => { unimplemented!() }
        InstructionInfo::lastore => { unimplemented!() }
        InstructionInfo::lcmp => instruction_is_type_safe_lcmp(env, offset, stack_frame),
        InstructionInfo::lconst_0 => instruction_is_type_safe_lconst_0(env, offset, stack_frame),
        InstructionInfo::lconst_1 => { unimplemented!() }
        InstructionInfo::ldc(_) => { unimplemented!() }
        InstructionInfo::ldc_w(_) => { unimplemented!() }
        InstructionInfo::ldc2_w(_) => { unimplemented!() }
        InstructionInfo::ldiv => { unimplemented!() }
        InstructionInfo::lload(_) => { unimplemented!() }
        InstructionInfo::lload_0 => instruction_is_type_safe_lload(0, env, offset, stack_frame),
        InstructionInfo::lload_1 => instruction_is_type_safe_lload(1, env, offset, stack_frame),
        InstructionInfo::lload_2 => instruction_is_type_safe_lload(2, env, offset, stack_frame),
        InstructionInfo::lload_3 => instruction_is_type_safe_lload(3, env, offset, stack_frame),
        InstructionInfo::lmul => { unimplemented!() }
        InstructionInfo::lneg => { unimplemented!() }
        InstructionInfo::lookupswitch(_) => { unimplemented!() }
        InstructionInfo::lor => { unimplemented!() }
        InstructionInfo::lrem => { unimplemented!() }
        InstructionInfo::lreturn => { unimplemented!() }
        InstructionInfo::lshl => { unimplemented!() }
        InstructionInfo::lshr => { unimplemented!() }
        InstructionInfo::lstore(_) => { unimplemented!() }
        InstructionInfo::lstore_0 => { unimplemented!() }
        InstructionInfo::lstore_1 => { unimplemented!() }
        InstructionInfo::lstore_2 => { unimplemented!() }
        InstructionInfo::lstore_3 => { unimplemented!() }
        InstructionInfo::lsub => { unimplemented!() }
        InstructionInfo::lushr => { unimplemented!() }
        InstructionInfo::lxor => { unimplemented!() }
        InstructionInfo::monitorenter => { unimplemented!() }
        InstructionInfo::monitorexit => { unimplemented!() }
        InstructionInfo::multianewarray(_) => { unimplemented!() }
        InstructionInfo::new(_) => { unimplemented!() }
        InstructionInfo::newarray(_) => { unimplemented!() }
        InstructionInfo::nop => { unimplemented!() }
        InstructionInfo::pop => { unimplemented!() }
        InstructionInfo::pop2 => { unimplemented!() }
        InstructionInfo::putfield(cp) => instruction_is_type_safe_putfield(cp, env, offset, stack_frame),
        InstructionInfo::putstatic(_) => { unimplemented!() }
        InstructionInfo::ret(_) => { unimplemented!() }
        InstructionInfo::return_ => instruction_is_type_safe_return(env, offset, stack_frame),
        InstructionInfo::saload => { unimplemented!() }
        InstructionInfo::sastore => { unimplemented!() }
        InstructionInfo::sipush(_) => { unimplemented!() }
        InstructionInfo::swap => { unimplemented!() }
        InstructionInfo::tableswitch(_) => { unimplemented!() }
        InstructionInfo::wide(_) => { unimplemented!() }
        _ => unimplemented!()
    }
}
