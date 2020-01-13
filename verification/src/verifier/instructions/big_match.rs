use rust_jvm_common::classfile::{InstructionInfo, Instruction};
use crate::verifier::instructions::loads::{instruction_is_type_safe_aload, instruction_is_type_safe_lload, instruction_is_type_safe_fload};
use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::TypeSafetyError;
use crate::verifier::instructions::{InstructionTypeSafe, instruction_is_type_safe_ldc, instruction_is_type_safe_dup, instruction_is_type_safe_i2f};
use crate::verifier::instructions::branches::{instruction_is_type_safe_goto, instruction_is_type_safe_invokespecial, instruction_is_type_safe_invokedynamic, instruction_is_type_safe_areturn, instruction_is_type_safe_ifeq};
use crate::verifier::instructions::branches::instruction_is_type_safe_if_acmpeq;
use crate::verifier::instructions::branches::instruction_is_type_safe_invokestatic;
use crate::verifier::instructions::branches::instruction_is_type_safe_ireturn;
use crate::verifier::instructions::instruction_is_type_safe_lcmp;
use crate::verifier::instructions::consts::{instruction_is_type_safe_lconst_0, instruction_is_type_safe_fconst_0};
use crate::verifier::instructions::branches::instruction_is_type_safe_return;
use crate::verifier::instructions::consts::instruction_is_type_safe_iconst_m1;
use crate::verifier::instructions::branches::instruction_is_type_safe_invokevirtual;
use crate::verifier::instructions::special::{instruction_is_type_safe_putfield, instruction_is_type_safe_getfield, instruction_is_type_safe_new, instruction_is_type_safe_athrow, instruction_is_type_safe_checkcast};
use crate::verifier::instructions::special::instruction_is_type_safe_getstatic;
use crate::verifier::instructions::loads::instruction_is_type_safe_iload;
use crate::verifier::instructions::branches::instruction_is_type_safe_if_icmpeq;
use crate::verifier::instructions::instruction_is_type_safe_ldc2_w;
use crate::verifier::instructions::instruction_is_type_safe_pop;
use crate::verifier::instructions::instruction_is_type_safe_ladd;
use crate::verifier::instructions::branches::instruction_is_type_safe_ifnonnull;
use crate::verifier::instructions::consts::instruction_is_type_safe_aconst_null;
use crate::verifier::instructions::stores::{instruction_is_type_safe_lstore, instruction_is_type_safe_fstore};
use crate::verifier::instructions::stores::instruction_is_type_safe_istore;
use crate::verifier::instructions::consts::instruction_is_type_safe_dconst_0;
use crate::verifier::instructions::float::{instruction_is_type_safe_dcmpg, instruction_is_type_safe_fcmpg, instruction_is_type_safe_fadd, instruction_is_type_safe_f2i};
use crate::verifier::instructions::float::instruction_is_type_safe_dadd;
use crate::verifier::instructions::float::instruction_is_type_safe_d2f;
use crate::verifier::instructions::float::instruction_is_type_safe_f2d;
use crate::verifier::instructions::stores::instruction_is_type_safe_astore;
use crate::verifier::instructions::instruction_is_type_safe_iadd;
use crate::verifier::instructions::instruction_is_type_safe_sipush;
use crate::verifier::instructions::instruction_is_type_safe_i2d;
use crate::verifier::instructions::stores::instruction_is_type_safe_dstore;
use crate::verifier::instructions::instruction_is_type_safe_iinc;
use crate::verifier::instructions::branches::instruction_is_type_safe_invokeinterface;
use crate::verifier::instructions::special::instruction_is_type_safe_monitorenter;
use crate::verifier::instructions::special::instruction_is_type_safe_arraylength;
use crate::verifier::instructions::loads::instruction_is_type_safe_aaload;
use crate::verifier::instructions::special::instruction_is_type_safe_putstatic;
use crate::verifier::instructions::special::instruction_is_type_safe_anewarray;
use crate::verifier::instructions::stores::instruction_is_type_safe_aastore;
use crate::verifier::instructions::special::instruction_is_type_safe_instanceof;
use crate::verifier::instructions::instruction_is_type_safe_dup_x1;

pub fn instruction_is_type_safe(instruction: &Instruction, env: &Environment, offset: usize, stack_frame: &Frame) -> Result<InstructionTypeSafe, TypeSafetyError> {
    dbg!(&stack_frame.stack_map);
    dbg!(instruction);
    match &instruction.instruction {
        InstructionInfo::aaload => instruction_is_type_safe_aaload(env,offset,stack_frame),
        InstructionInfo::aastore => instruction_is_type_safe_aastore(env,offset,stack_frame),
        InstructionInfo::aconst_null => instruction_is_type_safe_aconst_null(env, offset, stack_frame),
        InstructionInfo::aload(i) => instruction_is_type_safe_aload(*i as usize, env, offset, stack_frame),
        InstructionInfo::aload_0 => instruction_is_type_safe_aload(0, env, offset, stack_frame),
        InstructionInfo::aload_1 => instruction_is_type_safe_aload(1, env, offset, stack_frame),
        InstructionInfo::aload_2 => instruction_is_type_safe_aload(2, env, offset, stack_frame),
        InstructionInfo::aload_3 => instruction_is_type_safe_aload(3, env, offset, stack_frame),
        InstructionInfo::anewarray(cp) => instruction_is_type_safe_anewarray(*cp,env,offset,stack_frame),
        InstructionInfo::areturn => instruction_is_type_safe_areturn(env, offset, stack_frame),
        InstructionInfo::arraylength => instruction_is_type_safe_arraylength(env,offset,stack_frame),
        InstructionInfo::astore(i) => instruction_is_type_safe_astore(*i as usize, env, offset, stack_frame),
        InstructionInfo::astore_0 => instruction_is_type_safe_astore(0 as usize, env, offset, stack_frame),
        InstructionInfo::astore_1 => instruction_is_type_safe_astore(1 as usize, env, offset, stack_frame),
        InstructionInfo::astore_2 => instruction_is_type_safe_astore(2 as usize, env, offset, stack_frame),
        InstructionInfo::astore_3 => instruction_is_type_safe_astore(3 as usize, env, offset, stack_frame),
        InstructionInfo::athrow => instruction_is_type_safe_athrow(env, offset, stack_frame),
        InstructionInfo::baload => { unimplemented!() }
        InstructionInfo::bastore => { unimplemented!() }
        InstructionInfo::bipush(_) => instruction_is_type_safe_sipush(env, offset, stack_frame),
        InstructionInfo::caload => { unimplemented!() }
        InstructionInfo::castore => { unimplemented!() }
        InstructionInfo::checkcast(cp) => instruction_is_type_safe_checkcast(*cp as usize,env,offset,stack_frame),
        InstructionInfo::d2f => instruction_is_type_safe_d2f(env, offset, stack_frame),
        InstructionInfo::d2i => { unimplemented!() }
        InstructionInfo::d2l => { unimplemented!() }
        InstructionInfo::dadd => { unimplemented!() }
        InstructionInfo::daload => { unimplemented!() }
        InstructionInfo::dastore => { unimplemented!() }
        InstructionInfo::dcmpg => instruction_is_type_safe_dcmpg(env, offset, stack_frame),
        InstructionInfo::dcmpl => instruction_is_type_safe_dcmpg(env, offset, stack_frame),
        InstructionInfo::dconst_0 => instruction_is_type_safe_dconst_0(env, offset, stack_frame),
        InstructionInfo::dconst_1 => instruction_is_type_safe_dconst_0(env, offset, stack_frame),
        InstructionInfo::ddiv => { unimplemented!() }
        InstructionInfo::dload(_) => { unimplemented!() }
        InstructionInfo::dload_0 => { unimplemented!() }
        InstructionInfo::dload_1 => { unimplemented!() }
        InstructionInfo::dload_2 => { unimplemented!() }
        InstructionInfo::dload_3 => { unimplemented!() }
        InstructionInfo::dmul => instruction_is_type_safe_dadd(env, offset, stack_frame),
        InstructionInfo::dneg => { unimplemented!() }
        InstructionInfo::drem => { unimplemented!() }
        InstructionInfo::dreturn => { unimplemented!() }
        InstructionInfo::dstore(i) => instruction_is_type_safe_dstore(*i as usize, env, offset, stack_frame),
        InstructionInfo::dstore_0 => instruction_is_type_safe_dstore(0,env,offset,stack_frame),
        InstructionInfo::dstore_1 => instruction_is_type_safe_dstore(1,env,offset,stack_frame),
        InstructionInfo::dstore_2 => instruction_is_type_safe_dstore(2,env,offset,stack_frame),
        InstructionInfo::dstore_3 => instruction_is_type_safe_dstore(3,env,offset,stack_frame),
        InstructionInfo::dsub => { unimplemented!() }
        InstructionInfo::dup => instruction_is_type_safe_dup(env, offset, stack_frame),
        InstructionInfo::dup_x1 => instruction_is_type_safe_dup_x1(env,offset,stack_frame),
        InstructionInfo::dup_x2 => { unimplemented!() }
        InstructionInfo::dup2 => { unimplemented!() }
        InstructionInfo::dup2_x1 => { unimplemented!() }
        InstructionInfo::dup2_x2 => { unimplemented!() }
        InstructionInfo::f2d => instruction_is_type_safe_f2d(env, offset, stack_frame),
        InstructionInfo::f2i => instruction_is_type_safe_f2i(env,offset,stack_frame),
        InstructionInfo::f2l => { unimplemented!() }
        InstructionInfo::fadd => instruction_is_type_safe_fadd(env,offset,stack_frame),
        InstructionInfo::faload => { unimplemented!() }
        InstructionInfo::fastore => { unimplemented!() }
        InstructionInfo::fcmpg => instruction_is_type_safe_fcmpg(env,offset,stack_frame),
        InstructionInfo::fcmpl => instruction_is_type_safe_fcmpg(env,offset,stack_frame),
        InstructionInfo::fconst_0 => instruction_is_type_safe_fconst_0(env,offset,stack_frame),
        InstructionInfo::fconst_1 => instruction_is_type_safe_fconst_0(env,offset,stack_frame),
        InstructionInfo::fconst_2 => instruction_is_type_safe_fconst_0(env,offset,stack_frame),
        InstructionInfo::fdiv => instruction_is_type_safe_fadd(env,offset,stack_frame),
        InstructionInfo::fload(i) => instruction_is_type_safe_fload(*i as usize,env,offset,stack_frame),
        InstructionInfo::fload_0 => instruction_is_type_safe_fload(0,env,offset,stack_frame),
        InstructionInfo::fload_1 => instruction_is_type_safe_fload(1,env,offset,stack_frame),
        InstructionInfo::fload_2 => instruction_is_type_safe_fload(2,env,offset,stack_frame),
        InstructionInfo::fload_3 => instruction_is_type_safe_fload(3,env,offset,stack_frame),
        InstructionInfo::fmul => { unimplemented!() }
        InstructionInfo::fneg => { unimplemented!() }
        InstructionInfo::frem => { unimplemented!() }
        InstructionInfo::freturn => { unimplemented!() }
        InstructionInfo::fstore(i) => instruction_is_type_safe_fstore(*i as usize,env,offset,stack_frame),
        InstructionInfo::fstore_0 => instruction_is_type_safe_fstore(0,env,offset,stack_frame),
        InstructionInfo::fstore_1 => instruction_is_type_safe_fstore(1,env,offset,stack_frame),
        InstructionInfo::fstore_2 => instruction_is_type_safe_fstore(2,env,offset,stack_frame),
        InstructionInfo::fstore_3 => instruction_is_type_safe_fstore(3,env,offset,stack_frame),
        InstructionInfo::fsub => { unimplemented!() }
        InstructionInfo::getfield(cp) => instruction_is_type_safe_getfield(*cp, env, offset, stack_frame),
        InstructionInfo::getstatic(cp) => instruction_is_type_safe_getstatic(*cp, env, offset, stack_frame),
        InstructionInfo::goto_(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_goto(final_target as usize, env, offset, stack_frame)
        }
        InstructionInfo::goto_w(_) => { unimplemented!() }
        InstructionInfo::i2b => { unimplemented!() }
        InstructionInfo::i2c => { unimplemented!() }
        InstructionInfo::i2d => instruction_is_type_safe_i2d(env, offset, stack_frame),
        InstructionInfo::i2f => instruction_is_type_safe_i2f(env,offset,stack_frame),
        InstructionInfo::i2l => { unimplemented!() }
        InstructionInfo::i2s => instruction_is_type_safe_i2f(env,offset,stack_frame),
        InstructionInfo::iadd => instruction_is_type_safe_iadd(env, offset, stack_frame),
        InstructionInfo::iaload => { unimplemented!() }
        InstructionInfo::iand => instruction_is_type_safe_iadd(env,offset,stack_frame),
        InstructionInfo::iastore => { unimplemented!() }
        InstructionInfo::iconst_m1 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_0 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_1 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_2 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_3 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_4 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::iconst_5 => instruction_is_type_safe_iconst_m1(env, offset, stack_frame),
        InstructionInfo::idiv => { unimplemented!() }
        InstructionInfo::if_acmpeq(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as usize, env, offset, stack_frame)//same as eq case
        }
        InstructionInfo::if_acmpne(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as usize, env, offset, stack_frame)//same as eq case
        }
        InstructionInfo::if_icmpeq(target) => if_icmp_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::if_icmpne(target) => if_icmp_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::if_icmplt(target) => if_icmp_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::if_icmpge(target) => if_icmp_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::if_icmpgt(target) => if_icmp_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::if_icmple(target) => if_icmp_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::ifeq(target) => ifeq_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::ifne(target) => ifeq_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::iflt(target) => ifeq_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::ifge(target) => ifeq_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::ifgt(target) => ifeq_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::ifle(target) => ifeq_wrapper(instruction, env, offset, stack_frame, *target),
        InstructionInfo::ifnonnull(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_ifnonnull(final_target as usize, env, offset, stack_frame)
        }
        InstructionInfo::ifnull(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_ifnonnull(final_target as usize, env, offset, stack_frame)
        }
        InstructionInfo::iinc(iinc) => instruction_is_type_safe_iinc(iinc.index as usize, env, offset, stack_frame),
        InstructionInfo::iload(index) => instruction_is_type_safe_iload(*index as usize, env, offset, stack_frame),
        InstructionInfo::iload_0 => instruction_is_type_safe_iload(0, env, offset, stack_frame),
        InstructionInfo::iload_1 => instruction_is_type_safe_iload(1, env, offset, stack_frame),
        InstructionInfo::iload_2 => instruction_is_type_safe_iload(2, env, offset, stack_frame),
        InstructionInfo::iload_3 => instruction_is_type_safe_iload(3, env, offset, stack_frame),
        InstructionInfo::imul => { unimplemented!() }
        InstructionInfo::ineg => { unimplemented!() }
        InstructionInfo::instanceof(cp) => instruction_is_type_safe_instanceof(*cp,env,offset,stack_frame),
        InstructionInfo::invokedynamic(cp) => instruction_is_type_safe_invokedynamic(*cp as usize, env, offset, stack_frame),
        InstructionInfo::invokeinterface(ii) => instruction_is_type_safe_invokeinterface(ii.index as usize,ii.count as usize,env,offset,stack_frame),
        InstructionInfo::invokespecial(cp) => instruction_is_type_safe_invokespecial(*cp as usize, env, offset, stack_frame),
        InstructionInfo::invokestatic(cp) => instruction_is_type_safe_invokestatic(*cp as usize, env, offset, stack_frame),
        InstructionInfo::invokevirtual(v) => instruction_is_type_safe_invokevirtual(*v as usize, env, offset, stack_frame),
        InstructionInfo::ior => { unimplemented!() }
        InstructionInfo::irem => { unimplemented!() }
        InstructionInfo::ireturn => instruction_is_type_safe_ireturn(env, offset, stack_frame),
        InstructionInfo::ishl => instruction_is_type_safe_iadd(env,offset,stack_frame),
        InstructionInfo::ishr => instruction_is_type_safe_iadd(env,offset,stack_frame),
        InstructionInfo::istore(i) => instruction_is_type_safe_istore(*i as usize, env, offset, stack_frame),
        InstructionInfo::istore_0 => instruction_is_type_safe_istore(0, env, offset, stack_frame),
        InstructionInfo::istore_1 => instruction_is_type_safe_istore(1, env, offset, stack_frame),
        InstructionInfo::istore_2 => instruction_is_type_safe_istore(2, env, offset, stack_frame),
        InstructionInfo::istore_3 => instruction_is_type_safe_istore(3, env, offset, stack_frame),
        InstructionInfo::isub => instruction_is_type_safe_iadd(env,offset,stack_frame),
        InstructionInfo::iushr => instruction_is_type_safe_iadd(env,offset,stack_frame),
        InstructionInfo::ixor => instruction_is_type_safe_iadd(env,offset,stack_frame),
        InstructionInfo::jsr(_) => { unimplemented!() }
        InstructionInfo::jsr_w(_) => { unimplemented!() }
        InstructionInfo::l2d => { unimplemented!() }
        InstructionInfo::l2f => { unimplemented!() }
        InstructionInfo::l2i => { unimplemented!() }
        InstructionInfo::ladd => instruction_is_type_safe_ladd(env, offset, stack_frame),
        InstructionInfo::laload => { unimplemented!() }
        InstructionInfo::land => { unimplemented!() }
        InstructionInfo::lastore => { unimplemented!() }
        InstructionInfo::lcmp => instruction_is_type_safe_lcmp(env, offset, stack_frame),
        InstructionInfo::lconst_0 => instruction_is_type_safe_lconst_0(env, offset, stack_frame),
        InstructionInfo::lconst_1 => instruction_is_type_safe_lconst_0(env, offset, stack_frame),
        InstructionInfo::ldc(i) => instruction_is_type_safe_ldc(*i, env, offset, stack_frame),
        InstructionInfo::ldc_w(_) => { unimplemented!() }
        InstructionInfo::ldc2_w(cp) => instruction_is_type_safe_ldc2_w(*cp, env, offset, stack_frame),
        InstructionInfo::ldiv => { unimplemented!() }
        InstructionInfo::lload(_) => { unimplemented!() }
        InstructionInfo::lload_0 => instruction_is_type_safe_lload(0, env, offset, stack_frame),
        InstructionInfo::lload_1 => instruction_is_type_safe_lload(1, env, offset, stack_frame),
        InstructionInfo::lload_2 => instruction_is_type_safe_lload(2, env, offset, stack_frame),
        InstructionInfo::lload_3 => instruction_is_type_safe_lload(3, env, offset, stack_frame),
        InstructionInfo::lmul => instruction_is_type_safe_ladd(env, offset, stack_frame),
        InstructionInfo::lneg => { unimplemented!() }
        InstructionInfo::lookupswitch(_) => { unimplemented!() }
        InstructionInfo::lor => { unimplemented!() }
        InstructionInfo::lrem => { unimplemented!() }
        InstructionInfo::lreturn => { unimplemented!() }
        InstructionInfo::lshl => { unimplemented!() }
        InstructionInfo::lshr => { unimplemented!() }
        //todo offtopic but offset is like the most useless param ever.
        InstructionInfo::lstore(i) => instruction_is_type_safe_lstore(*i as usize, env, offset, stack_frame),
        InstructionInfo::lstore_0 => instruction_is_type_safe_lstore(0, env, offset, stack_frame),
        InstructionInfo::lstore_1 => instruction_is_type_safe_lstore(1, env, offset, stack_frame),
        InstructionInfo::lstore_2 => instruction_is_type_safe_lstore(2, env, offset, stack_frame),
        InstructionInfo::lstore_3 => instruction_is_type_safe_lstore(3, env, offset, stack_frame),
        InstructionInfo::lsub => { unimplemented!() }
        InstructionInfo::lushr => { unimplemented!() }
        InstructionInfo::lxor => { unimplemented!() }
        InstructionInfo::monitorenter => instruction_is_type_safe_monitorenter(env,offset,stack_frame),
        InstructionInfo::monitorexit => instruction_is_type_safe_monitorenter(env,offset,stack_frame),
        InstructionInfo::multianewarray(_) => { unimplemented!() }
        InstructionInfo::new(cp) => instruction_is_type_safe_new(*cp as usize, env, offset, stack_frame),
        InstructionInfo::newarray(_) => { unimplemented!() }
        InstructionInfo::nop => { unimplemented!() }
        InstructionInfo::pop => instruction_is_type_safe_pop(env, offset, stack_frame),
        InstructionInfo::pop2 => { unimplemented!() }
        InstructionInfo::putfield(cp) => instruction_is_type_safe_putfield(*cp, env, offset, stack_frame),
        InstructionInfo::putstatic(cp) => instruction_is_type_safe_putstatic(*cp, env, offset, stack_frame),
        InstructionInfo::ret(_) => { unimplemented!() }
        InstructionInfo::return_ => instruction_is_type_safe_return(env, offset, stack_frame),
        InstructionInfo::saload => { unimplemented!() }
        InstructionInfo::sastore => { unimplemented!() }
        InstructionInfo::sipush(_) => instruction_is_type_safe_sipush(env, offset, stack_frame),
        InstructionInfo::swap => { unimplemented!() }
        InstructionInfo::tableswitch(_) => { unimplemented!() }
        InstructionInfo::wide(_) => { unimplemented!() }
        _ => unimplemented!()
    }
}

fn if_icmp_wrapper(instruction: &Instruction, env: &Environment, offset: usize, stack_frame: &Frame, target: i16) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let final_target = (target as isize) + (instruction.offset as isize);
    assert!(final_target >= 0);
    instruction_is_type_safe_if_icmpeq(final_target as usize, env, offset, stack_frame)
}

fn ifeq_wrapper(instruction: &Instruction, env: &Environment, offset: usize, stack_frame: &Frame, target: i16) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let final_target = (target as isize) + (instruction.offset as isize);
    assert!(final_target >= 0);
    instruction_is_type_safe_ifeq(final_target as usize, env, offset, stack_frame)
}
