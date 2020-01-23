use rust_jvm_common::classfile::{Instruction, InstructionInfo};

use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::{instruction_is_type_safe_dup, instruction_is_type_safe_dup_x1, instruction_is_type_safe_dup_x2, instruction_is_type_safe_i2d, instruction_is_type_safe_i2f, instruction_is_type_safe_i2l, instruction_is_type_safe_iadd, instruction_is_type_safe_iinc, instruction_is_type_safe_ineg, instruction_is_type_safe_l2i, instruction_is_type_safe_ladd, instruction_is_type_safe_lcmp, instruction_is_type_safe_ldc, instruction_is_type_safe_ldc2_w, instruction_is_type_safe_ldc_w, instruction_is_type_safe_lneg, instruction_is_type_safe_lshl, instruction_is_type_safe_pop, instruction_is_type_safe_sipush, InstructionTypeSafe,
};
use crate::verifier::instructions::branches::*;
use crate::verifier::instructions::consts::*;
use crate::verifier::instructions::float::*;
use crate::verifier::instructions::loads::*;
use crate::verifier::instructions::special::*;
use crate::verifier::instructions::stores::*;
use crate::verifier::TypeSafetyError;

pub fn instruction_is_type_safe(instruction: &Instruction, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
//    dbg!(&stack_frame.stack_map);
//    dbg!(instruction);
    match &instruction.instruction {
        InstructionInfo::aaload => instruction_is_type_safe_aaload(env, stack_frame),
        InstructionInfo::aastore => instruction_is_type_safe_aastore(env, stack_frame),
        InstructionInfo::aconst_null => instruction_is_type_safe_aconst_null(env, stack_frame),
        InstructionInfo::aload(i) => instruction_is_type_safe_aload(*i as usize, env, stack_frame),
        InstructionInfo::aload_0 => instruction_is_type_safe_aload(0, env, stack_frame),
        InstructionInfo::aload_1 => instruction_is_type_safe_aload(1, env, stack_frame),
        InstructionInfo::aload_2 => instruction_is_type_safe_aload(2, env, stack_frame),
        InstructionInfo::aload_3 => instruction_is_type_safe_aload(3, env, stack_frame),
        InstructionInfo::anewarray(cp) => instruction_is_type_safe_anewarray(*cp, env, stack_frame),
        InstructionInfo::areturn => instruction_is_type_safe_areturn(env, stack_frame),
        InstructionInfo::arraylength => instruction_is_type_safe_arraylength(env, stack_frame),
        InstructionInfo::astore(i) => instruction_is_type_safe_astore(*i as usize, env, stack_frame),
        InstructionInfo::astore_0 => instruction_is_type_safe_astore(0 as usize, env, stack_frame),
        InstructionInfo::astore_1 => instruction_is_type_safe_astore(1 as usize, env, stack_frame),
        InstructionInfo::astore_2 => instruction_is_type_safe_astore(2 as usize, env, stack_frame),
        InstructionInfo::astore_3 => instruction_is_type_safe_astore(3 as usize, env, stack_frame),
        InstructionInfo::athrow => instruction_is_type_safe_athrow(env, stack_frame),
        InstructionInfo::baload => instruction_is_type_safe_baload(env, stack_frame),
        InstructionInfo::bastore => instruction_is_type_safe_bastore(env, stack_frame),
        InstructionInfo::bipush(_) => instruction_is_type_safe_sipush(env, stack_frame),
        InstructionInfo::caload => instruction_is_type_safe_caload(env, stack_frame),
        InstructionInfo::castore => instruction_is_type_safe_castore(env, stack_frame),
        InstructionInfo::checkcast(cp) => instruction_is_type_safe_checkcast(*cp as usize, env, stack_frame),
        InstructionInfo::d2f => instruction_is_type_safe_d2f(env, stack_frame),
        InstructionInfo::d2i => instruction_is_type_safe_d2i(env,stack_frame),
        InstructionInfo::d2l => instruction_is_type_safe_d2l(env, stack_frame),
        InstructionInfo::dadd => instruction_is_type_safe_dadd(env, stack_frame),
        InstructionInfo::daload => instruction_is_type_safe_daload(env, stack_frame),
        InstructionInfo::dastore => instruction_is_type_safe_dastore(env, stack_frame),
        InstructionInfo::dcmpg => instruction_is_type_safe_dcmpg(env, stack_frame),
        InstructionInfo::dcmpl => instruction_is_type_safe_dcmpg(env, stack_frame),
        InstructionInfo::dconst_0 => instruction_is_type_safe_dconst_0(env, stack_frame),
        InstructionInfo::dconst_1 => instruction_is_type_safe_dconst_0(env, stack_frame),
        InstructionInfo::ddiv => instruction_is_type_safe_dadd(env, stack_frame),
        InstructionInfo::dload(i) => instruction_is_type_safe_dload(*i as usize, env, stack_frame),
        InstructionInfo::dload_0 => instruction_is_type_safe_dload(0, env, stack_frame),
        InstructionInfo::dload_1 => instruction_is_type_safe_dload(1, env, stack_frame),
        InstructionInfo::dload_2 => instruction_is_type_safe_dload(2, env, stack_frame),
        InstructionInfo::dload_3 => instruction_is_type_safe_dload(3, env, stack_frame),
        InstructionInfo::dmul => instruction_is_type_safe_dadd(env, stack_frame),
        InstructionInfo::dneg => { unimplemented!() }
        InstructionInfo::drem => { unimplemented!() }
        InstructionInfo::dreturn => instruction_is_type_safe_dreturn(env, stack_frame),
        InstructionInfo::dstore(i) => instruction_is_type_safe_dstore(*i as usize, env, stack_frame),
        InstructionInfo::dstore_0 => instruction_is_type_safe_dstore(0, env, stack_frame),
        InstructionInfo::dstore_1 => instruction_is_type_safe_dstore(1, env, stack_frame),
        InstructionInfo::dstore_2 => instruction_is_type_safe_dstore(2, env, stack_frame),
        InstructionInfo::dstore_3 => instruction_is_type_safe_dstore(3, env, stack_frame),
        InstructionInfo::dsub => instruction_is_type_safe_dadd(env, stack_frame),
        InstructionInfo::dup => instruction_is_type_safe_dup(env, stack_frame),
        InstructionInfo::dup_x1 => instruction_is_type_safe_dup_x1(env, stack_frame),
        InstructionInfo::dup_x2 => instruction_is_type_safe_dup_x2(env, stack_frame),
        InstructionInfo::dup2 => { unimplemented!() }
        InstructionInfo::dup2_x1 => { unimplemented!() }
        InstructionInfo::dup2_x2 => { unimplemented!() }
        InstructionInfo::f2d => instruction_is_type_safe_f2d(env, stack_frame),
        InstructionInfo::f2i => instruction_is_type_safe_f2i(env, stack_frame),
        InstructionInfo::f2l => instruction_is_type_safe_f2l(env, stack_frame),
        InstructionInfo::fadd => instruction_is_type_safe_fadd(env, stack_frame),
        InstructionInfo::faload => instruction_is_type_safe_faload(env, stack_frame),
        InstructionInfo::fastore => instruction_is_type_safe_fastore(env, stack_frame),
        InstructionInfo::fcmpg => instruction_is_type_safe_fcmpg(env, stack_frame),
        InstructionInfo::fcmpl => instruction_is_type_safe_fcmpg(env, stack_frame),
        InstructionInfo::fconst_0 => instruction_is_type_safe_fconst_0(env, stack_frame),
        InstructionInfo::fconst_1 => instruction_is_type_safe_fconst_0(env, stack_frame),
        InstructionInfo::fconst_2 => instruction_is_type_safe_fconst_0(env, stack_frame),
        InstructionInfo::fdiv => instruction_is_type_safe_fadd(env, stack_frame),
        InstructionInfo::fload(i) => instruction_is_type_safe_fload(*i as usize, env, stack_frame),
        InstructionInfo::fload_0 => instruction_is_type_safe_fload(0, env, stack_frame),
        InstructionInfo::fload_1 => instruction_is_type_safe_fload(1, env, stack_frame),
        InstructionInfo::fload_2 => instruction_is_type_safe_fload(2, env, stack_frame),
        InstructionInfo::fload_3 => instruction_is_type_safe_fload(3, env, stack_frame),
        InstructionInfo::fmul => instruction_is_type_safe_fadd(env, stack_frame),
        InstructionInfo::fneg => { unimplemented!() }
        InstructionInfo::frem => { unimplemented!() }
        InstructionInfo::freturn => instruction_is_type_safe_freturn(env, stack_frame),
        InstructionInfo::fstore(i) => instruction_is_type_safe_fstore(*i as usize, env, stack_frame),
        InstructionInfo::fstore_0 => instruction_is_type_safe_fstore(0, env, stack_frame),
        InstructionInfo::fstore_1 => instruction_is_type_safe_fstore(1, env, stack_frame),
        InstructionInfo::fstore_2 => instruction_is_type_safe_fstore(2, env, stack_frame),
        InstructionInfo::fstore_3 => instruction_is_type_safe_fstore(3, env, stack_frame),
        InstructionInfo::fsub => instruction_is_type_safe_fadd(env, stack_frame),
        InstructionInfo::getfield(cp) => instruction_is_type_safe_getfield(*cp, env, stack_frame),
        InstructionInfo::getstatic(cp) => instruction_is_type_safe_getstatic(*cp, env, stack_frame),
        InstructionInfo::goto_(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_goto(final_target as usize, env, stack_frame)
        }
        InstructionInfo::goto_w(_) => { unimplemented!() }
        InstructionInfo::i2b => instruction_is_type_safe_ineg(env, stack_frame),
        InstructionInfo::i2c => instruction_is_type_safe_ineg(env, stack_frame),
        InstructionInfo::i2d => instruction_is_type_safe_i2d(env, stack_frame),
        InstructionInfo::i2f => instruction_is_type_safe_i2f(env, stack_frame),
        InstructionInfo::i2l => instruction_is_type_safe_i2l(env, stack_frame),
        InstructionInfo::i2s => instruction_is_type_safe_ineg(env, stack_frame),
        InstructionInfo::iadd => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::iaload => instruction_is_type_safe_iaload(env, stack_frame),
        InstructionInfo::iand => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::iastore => instruction_is_type_safe_iastore(env, stack_frame),
        InstructionInfo::iconst_m1 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::iconst_0 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::iconst_1 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::iconst_2 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::iconst_3 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::iconst_4 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::iconst_5 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        InstructionInfo::idiv => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::if_acmpeq(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as usize, env, stack_frame)//same as eq case
        }
        InstructionInfo::if_acmpne(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as usize, env, stack_frame)//same as eq case
        }
        InstructionInfo::if_icmpeq(target) => if_icmp_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::if_icmpne(target) => if_icmp_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::if_icmplt(target) => if_icmp_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::if_icmpge(target) => if_icmp_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::if_icmpgt(target) => if_icmp_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::if_icmple(target) => if_icmp_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::ifeq(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::ifne(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::iflt(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::ifge(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::ifgt(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::ifle(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        InstructionInfo::ifnonnull(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_ifnonnull(final_target as usize, env, stack_frame)
        }
        InstructionInfo::ifnull(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_ifnonnull(final_target as usize, env, stack_frame)
        }
        InstructionInfo::iinc(iinc) => instruction_is_type_safe_iinc(iinc.index as usize, env, stack_frame),
        InstructionInfo::iload(index) => instruction_is_type_safe_iload(*index as usize, env, stack_frame),
        InstructionInfo::iload_0 => instruction_is_type_safe_iload(0, env, stack_frame),
        InstructionInfo::iload_1 => instruction_is_type_safe_iload(1, env, stack_frame),
        InstructionInfo::iload_2 => instruction_is_type_safe_iload(2, env, stack_frame),
        InstructionInfo::iload_3 => instruction_is_type_safe_iload(3, env, stack_frame),
        InstructionInfo::imul => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::ineg => instruction_is_type_safe_ineg(env, stack_frame),
        InstructionInfo::instanceof(cp) => instruction_is_type_safe_instanceof(*cp, env, stack_frame),
        InstructionInfo::invokedynamic(cp) => instruction_is_type_safe_invokedynamic(*cp as usize, env, stack_frame),
        InstructionInfo::invokeinterface(ii) => instruction_is_type_safe_invokeinterface(ii.index as usize, ii.count as usize, env, stack_frame),
        InstructionInfo::invokespecial(cp) => instruction_is_type_safe_invokespecial(*cp as usize, env, stack_frame),
        InstructionInfo::invokestatic(cp) => instruction_is_type_safe_invokestatic(*cp as usize, env, stack_frame),
        InstructionInfo::invokevirtual(v) => instruction_is_type_safe_invokevirtual(*v as usize, env, stack_frame),
        InstructionInfo::ior => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::irem => { unimplemented!() }
        InstructionInfo::ireturn => instruction_is_type_safe_ireturn(env, stack_frame),
        InstructionInfo::ishl => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::ishr => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::istore(i) => instruction_is_type_safe_istore(*i as usize, env, stack_frame),
        InstructionInfo::istore_0 => instruction_is_type_safe_istore(0, env, stack_frame),
        InstructionInfo::istore_1 => instruction_is_type_safe_istore(1, env, stack_frame),
        InstructionInfo::istore_2 => instruction_is_type_safe_istore(2, env, stack_frame),
        InstructionInfo::istore_3 => instruction_is_type_safe_istore(3, env, stack_frame),
        InstructionInfo::isub => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::iushr => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::ixor => instruction_is_type_safe_iadd(env, stack_frame),
        InstructionInfo::jsr(_) => { unimplemented!() }
        InstructionInfo::jsr_w(_) => { unimplemented!() }
        InstructionInfo::l2d => { unimplemented!() }
        InstructionInfo::l2f => { unimplemented!() }
        InstructionInfo::l2i => instruction_is_type_safe_l2i(env, stack_frame),
        InstructionInfo::ladd => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::laload => instruction_is_type_safe_laload(env, stack_frame),
        InstructionInfo::land => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::lastore => instruction_is_type_safe_lastore(env, stack_frame),
        InstructionInfo::lcmp => instruction_is_type_safe_lcmp(env, stack_frame),
        InstructionInfo::lconst_0 => instruction_is_type_safe_lconst_0(env, stack_frame),
        InstructionInfo::lconst_1 => instruction_is_type_safe_lconst_0(env, stack_frame),
        InstructionInfo::ldc(i) => instruction_is_type_safe_ldc(*i, env, stack_frame),
        InstructionInfo::ldc_w(cp) => instruction_is_type_safe_ldc_w(*cp, env, stack_frame),
        InstructionInfo::ldc2_w(cp) => instruction_is_type_safe_ldc2_w(*cp, env, stack_frame),
        InstructionInfo::ldiv => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::lload(i) => instruction_is_type_safe_lload(*i as usize, env, stack_frame),
        InstructionInfo::lload_0 => instruction_is_type_safe_lload(0, env, stack_frame),
        InstructionInfo::lload_1 => instruction_is_type_safe_lload(1, env, stack_frame),
        InstructionInfo::lload_2 => instruction_is_type_safe_lload(2, env, stack_frame),
        InstructionInfo::lload_3 => instruction_is_type_safe_lload(3, env, stack_frame),
        InstructionInfo::lmul => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::lneg => instruction_is_type_safe_lneg(env, stack_frame),
        InstructionInfo::lookupswitch(s) => {
            let targets: Vec<usize> = s.pairs.iter().map(|(_, x)| {
                (offset as isize + *x as isize) as usize//todo create correct typedefs for usize etc.
            }).collect();
            let keys = s.pairs.iter().map(|(x, _)| {
                *x
            }).collect();
            instruction_is_type_safe_lookupswitch(targets, keys, env, stack_frame)
        }
        InstructionInfo::lor => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::lrem => { unimplemented!() }
        InstructionInfo::lreturn => instruction_is_type_safe_lreturn(env, stack_frame),
        InstructionInfo::lshl => instruction_is_type_safe_lshl(env, stack_frame),
        InstructionInfo::lshr => instruction_is_type_safe_lshl(env, stack_frame),
        //todo offtopic but offset is like the most useless param ever.
        InstructionInfo::lstore(i) => instruction_is_type_safe_lstore(*i as usize, env, stack_frame),
        InstructionInfo::lstore_0 => instruction_is_type_safe_lstore(0, env, stack_frame),
        InstructionInfo::lstore_1 => instruction_is_type_safe_lstore(1, env, stack_frame),
        InstructionInfo::lstore_2 => instruction_is_type_safe_lstore(2, env, stack_frame),
        InstructionInfo::lstore_3 => instruction_is_type_safe_lstore(3, env, stack_frame),
        InstructionInfo::lsub => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::lushr => instruction_is_type_safe_lshl(env, stack_frame),
        InstructionInfo::lxor => instruction_is_type_safe_ladd(env, stack_frame),
        InstructionInfo::monitorenter => instruction_is_type_safe_monitorenter(env, stack_frame),
        InstructionInfo::monitorexit => instruction_is_type_safe_monitorenter(env, stack_frame),
        InstructionInfo::multianewarray(_) => { unimplemented!() }
        InstructionInfo::new(cp) => instruction_is_type_safe_new(*cp as usize, offset, env, stack_frame),
        InstructionInfo::newarray(type_code) => instruction_is_type_safe_newarray(*type_code as usize, env, stack_frame),
        InstructionInfo::nop => { unimplemented!() }
        InstructionInfo::pop => instruction_is_type_safe_pop(env, stack_frame),
        InstructionInfo::pop2 => { unimplemented!() }
        InstructionInfo::putfield(cp) => instruction_is_type_safe_putfield(*cp, env, stack_frame),
        InstructionInfo::putstatic(cp) => instruction_is_type_safe_putstatic(*cp, env, stack_frame),
        InstructionInfo::ret(_) => { unimplemented!() }
        InstructionInfo::return_ => instruction_is_type_safe_return(env, stack_frame),
        InstructionInfo::saload => instruction_is_type_safe_saload(env, stack_frame),
        InstructionInfo::sastore => instruction_is_type_safe_sastore(env, stack_frame),
        InstructionInfo::sipush(_) => instruction_is_type_safe_sipush(env, stack_frame),
        InstructionInfo::swap => { unimplemented!() }
        InstructionInfo::tableswitch(s) => {
            let mut targets = vec![];
            for o in &s.offsets {
                targets.push((offset as isize + *o as isize) as usize)
            }
            targets.push((offset as isize + s.default as isize) as usize);
//            dbg!(&targets);
            instruction_is_type_safe_tableswitch(targets, env, stack_frame)
        }
        InstructionInfo::wide(_) => { unimplemented!() }
        _ => unimplemented!()
    }
}

fn if_icmp_wrapper(instruction: &Instruction, env: &Environment, stack_frame: &Frame, target: i16) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let final_target = (target as isize) + (instruction.offset as isize);
    assert!(final_target >= 0);
    instruction_is_type_safe_if_icmpeq(final_target as usize, env, stack_frame)
}

fn ifeq_wrapper(instruction: &Instruction, env: &Environment, stack_frame: &Frame, target: i16) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let final_target = (target as isize) + (instruction.offset as isize);
    assert!(final_target >= 0);
    instruction_is_type_safe_ifeq(final_target as usize, env, stack_frame)
}
