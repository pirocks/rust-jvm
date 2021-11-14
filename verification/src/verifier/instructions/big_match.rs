use std::collections::HashMap;

use rust_jvm_common::classfile::{
    Wide, WideAload, WideAstore, WideDload, WideDstore, WideFload, WideFstore, WideIload,
    WideIstore, WideLload, WideLstore, WideRet,
};
use rust_jvm_common::compressed_classfile::code::{CInstruction, CInstructionInfo};
use rust_jvm_common::compressed_classfile::CPDType;

use crate::verifier::codecorrectness::Environment;
use crate::verifier::Frame;
use crate::verifier::instructions::*;
use crate::verifier::instructions::branches::*;
use crate::verifier::instructions::consts::*;
use crate::verifier::instructions::float::*;
use crate::verifier::instructions::loads::*;
use crate::verifier::instructions::special::*;
use crate::verifier::instructions::stores::*;
use crate::verifier::TypeSafetyError;

pub fn instruction_is_type_safe(
    instruction: &CInstruction,
    env: &mut Environment,
    offset: u16,
    stack_frame: Frame,
) -> Result<InstructionTypeSafe, TypeSafetyError> {
    env.vf
        .verification_types
        .entry(env.method.method_index as u16)
        .or_insert(HashMap::new())
        .insert(offset, stack_frame.clone());
    match &instruction.info {
        CInstructionInfo::aaload => instruction_is_type_safe_aaload(env, stack_frame),
        CInstructionInfo::aastore => instruction_is_type_safe_aastore(env, stack_frame),
        CInstructionInfo::aconst_null => instruction_is_type_safe_aconst_null(env, stack_frame),
        CInstructionInfo::aload(i) => instruction_is_type_safe_aload(*i as u16, env, stack_frame),
        CInstructionInfo::aload_0 => instruction_is_type_safe_aload(0, env, stack_frame),
        CInstructionInfo::aload_1 => instruction_is_type_safe_aload(1, env, stack_frame),
        CInstructionInfo::aload_2 => instruction_is_type_safe_aload(2, env, stack_frame),
        CInstructionInfo::aload_3 => instruction_is_type_safe_aload(3, env, stack_frame),
        CInstructionInfo::anewarray(cpdtype) => {
            instruction_is_type_safe_anewarray(cpdtype, env, stack_frame)
        }
        CInstructionInfo::areturn => instruction_is_type_safe_areturn(env, stack_frame),
        CInstructionInfo::arraylength => instruction_is_type_safe_arraylength(env, stack_frame),
        CInstructionInfo::astore(i) => instruction_is_type_safe_astore(*i as u16, env, stack_frame),
        CInstructionInfo::astore_0 => instruction_is_type_safe_astore(0 as u16, env, stack_frame),
        CInstructionInfo::astore_1 => instruction_is_type_safe_astore(1 as u16, env, stack_frame),
        CInstructionInfo::astore_2 => instruction_is_type_safe_astore(2 as u16, env, stack_frame),
        CInstructionInfo::astore_3 => instruction_is_type_safe_astore(3 as u16, env, stack_frame),
        CInstructionInfo::athrow => instruction_is_type_safe_athrow(env, stack_frame),
        CInstructionInfo::baload => instruction_is_type_safe_baload(env, stack_frame),
        CInstructionInfo::bastore => instruction_is_type_safe_bastore(env, stack_frame),
        CInstructionInfo::bipush(_) => instruction_is_type_safe_sipush(env, stack_frame),
        CInstructionInfo::caload => instruction_is_type_safe_caload(env, stack_frame),
        CInstructionInfo::castore => instruction_is_type_safe_castore(env, stack_frame),
        CInstructionInfo::checkcast(cpdtype) => {
            instruction_is_type_safe_checkcast(cpdtype, env, stack_frame)
        }
        CInstructionInfo::d2f => instruction_is_type_safe_d2f(env, stack_frame),
        CInstructionInfo::d2i => instruction_is_type_safe_d2i(env, stack_frame),
        CInstructionInfo::d2l => instruction_is_type_safe_d2l(env, stack_frame),
        CInstructionInfo::dadd => instruction_is_type_safe_dadd(env, stack_frame),
        CInstructionInfo::daload => instruction_is_type_safe_daload(env, stack_frame),
        CInstructionInfo::dastore => instruction_is_type_safe_dastore(env, stack_frame),
        CInstructionInfo::dcmpg => instruction_is_type_safe_dcmpg(env, stack_frame),
        CInstructionInfo::dcmpl => instruction_is_type_safe_dcmpg(env, stack_frame),
        CInstructionInfo::dconst_0 => instruction_is_type_safe_dconst_0(env, stack_frame),
        CInstructionInfo::dconst_1 => instruction_is_type_safe_dconst_0(env, stack_frame),
        CInstructionInfo::ddiv => instruction_is_type_safe_dadd(env, stack_frame),
        CInstructionInfo::dload(i) => instruction_is_type_safe_dload(*i as u16, env, stack_frame),
        CInstructionInfo::dload_0 => instruction_is_type_safe_dload(0, env, stack_frame),
        CInstructionInfo::dload_1 => instruction_is_type_safe_dload(1, env, stack_frame),
        CInstructionInfo::dload_2 => instruction_is_type_safe_dload(2, env, stack_frame),
        CInstructionInfo::dload_3 => instruction_is_type_safe_dload(3, env, stack_frame),
        CInstructionInfo::dmul => instruction_is_type_safe_dadd(env, stack_frame),
        CInstructionInfo::dneg => instruction_is_type_safe_dneg(env, stack_frame),
        CInstructionInfo::drem => instruction_is_type_safe_dadd(env, stack_frame),
        CInstructionInfo::dreturn => instruction_is_type_safe_dreturn(env, stack_frame),
        CInstructionInfo::dstore(i) => instruction_is_type_safe_dstore(*i as u16, env, stack_frame),
        CInstructionInfo::dstore_0 => instruction_is_type_safe_dstore(0, env, stack_frame),
        CInstructionInfo::dstore_1 => instruction_is_type_safe_dstore(1, env, stack_frame),
        CInstructionInfo::dstore_2 => instruction_is_type_safe_dstore(2, env, stack_frame),
        CInstructionInfo::dstore_3 => instruction_is_type_safe_dstore(3, env, stack_frame),
        CInstructionInfo::dsub => instruction_is_type_safe_dadd(env, stack_frame),
        CInstructionInfo::dup => instruction_is_type_safe_dup(env, stack_frame),
        CInstructionInfo::dup_x1 => instruction_is_type_safe_dup_x1(env, stack_frame),
        CInstructionInfo::dup_x2 => instruction_is_type_safe_dup_x2(env, stack_frame),
        CInstructionInfo::dup2 => instruction_is_type_safe_dup2(env, stack_frame),
        CInstructionInfo::dup2_x1 => instruction_is_type_safe_dup2_x1(env, stack_frame),
        CInstructionInfo::dup2_x2 => instruction_is_type_safe_dup2_x2(env, stack_frame),
        CInstructionInfo::f2d => instruction_is_type_safe_f2d(env, stack_frame),
        CInstructionInfo::f2i => instruction_is_type_safe_f2i(env, stack_frame),
        CInstructionInfo::f2l => instruction_is_type_safe_f2l(env, stack_frame),
        CInstructionInfo::fadd => instruction_is_type_safe_fadd(env, stack_frame),
        CInstructionInfo::faload => instruction_is_type_safe_faload(env, stack_frame),
        CInstructionInfo::fastore => instruction_is_type_safe_fastore(env, stack_frame),
        CInstructionInfo::fcmpg => instruction_is_type_safe_fcmpg(env, stack_frame),
        CInstructionInfo::fcmpl => instruction_is_type_safe_fcmpg(env, stack_frame),
        CInstructionInfo::fconst_0 => instruction_is_type_safe_fconst_0(env, stack_frame),
        CInstructionInfo::fconst_1 => instruction_is_type_safe_fconst_0(env, stack_frame),
        CInstructionInfo::fconst_2 => instruction_is_type_safe_fconst_0(env, stack_frame),
        CInstructionInfo::fdiv => instruction_is_type_safe_fadd(env, stack_frame),
        CInstructionInfo::fload(i) => instruction_is_type_safe_fload(*i as u16, env, stack_frame),
        CInstructionInfo::fload_0 => instruction_is_type_safe_fload(0, env, stack_frame),
        CInstructionInfo::fload_1 => instruction_is_type_safe_fload(1, env, stack_frame),
        CInstructionInfo::fload_2 => instruction_is_type_safe_fload(2, env, stack_frame),
        CInstructionInfo::fload_3 => instruction_is_type_safe_fload(3, env, stack_frame),
        CInstructionInfo::fmul => instruction_is_type_safe_fadd(env, stack_frame),
        CInstructionInfo::fneg => instruction_is_type_safe_fneg(env, stack_frame),
        CInstructionInfo::frem => instruction_is_type_safe_fadd(env, stack_frame),
        CInstructionInfo::freturn => instruction_is_type_safe_freturn(env, stack_frame),
        CInstructionInfo::fstore(i) => instruction_is_type_safe_fstore(*i as u16, env, stack_frame),
        CInstructionInfo::fstore_0 => instruction_is_type_safe_fstore(0, env, stack_frame),
        CInstructionInfo::fstore_1 => instruction_is_type_safe_fstore(1, env, stack_frame),
        CInstructionInfo::fstore_2 => instruction_is_type_safe_fstore(2, env, stack_frame),
        CInstructionInfo::fstore_3 => instruction_is_type_safe_fstore(3, env, stack_frame),
        CInstructionInfo::fsub => instruction_is_type_safe_fadd(env, stack_frame),
        CInstructionInfo::getfield {
            name,
            desc,
            target_class,
        } => instruction_is_type_safe_getfield(*target_class, *name, desc, env, stack_frame),
        CInstructionInfo::getstatic {
            name,
            desc,
            target_class,
        } => instruction_is_type_safe_getstatic(*target_class, *name, desc, env, stack_frame),
        CInstructionInfo::goto_(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_goto(final_target as u16, env, stack_frame)
        }
        CInstructionInfo::goto_w(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_goto(final_target as u16, env, stack_frame)
        }
        CInstructionInfo::i2b => instruction_is_type_safe_ineg(env, stack_frame),
        CInstructionInfo::i2c => instruction_is_type_safe_ineg(env, stack_frame),
        CInstructionInfo::i2d => instruction_is_type_safe_i2d(env, stack_frame),
        CInstructionInfo::i2f => instruction_is_type_safe_i2f(env, stack_frame),
        CInstructionInfo::i2l => instruction_is_type_safe_i2l(env, stack_frame),
        CInstructionInfo::i2s => instruction_is_type_safe_ineg(env, stack_frame),
        CInstructionInfo::iadd => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::iaload => instruction_is_type_safe_iaload(env, stack_frame),
        CInstructionInfo::iand => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::iastore => instruction_is_type_safe_iastore(env, stack_frame),
        CInstructionInfo::iconst_m1 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::iconst_0 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::iconst_1 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::iconst_2 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::iconst_3 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::iconst_4 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::iconst_5 => instruction_is_type_safe_iconst_m1(env, stack_frame),
        CInstructionInfo::idiv => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::if_acmpeq(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as u16, env, stack_frame)
            //same as eq case
        }
        CInstructionInfo::if_acmpne(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_if_acmpeq(final_target as u16, env, stack_frame)
            //same as eq case
        }
        CInstructionInfo::if_icmpeq(target) => {
            if_icmp_wrapper(instruction, env, stack_frame, *target)
        }
        CInstructionInfo::if_icmpne(target) => {
            if_icmp_wrapper(instruction, env, stack_frame, *target)
        }
        CInstructionInfo::if_icmplt(target) => {
            if_icmp_wrapper(instruction, env, stack_frame, *target)
        }
        CInstructionInfo::if_icmpge(target) => {
            if_icmp_wrapper(instruction, env, stack_frame, *target)
        }
        CInstructionInfo::if_icmpgt(target) => {
            if_icmp_wrapper(instruction, env, stack_frame, *target)
        }
        CInstructionInfo::if_icmple(target) => {
            if_icmp_wrapper(instruction, env, stack_frame, *target)
        }
        CInstructionInfo::ifeq(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        CInstructionInfo::ifne(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        CInstructionInfo::iflt(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        CInstructionInfo::ifge(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        CInstructionInfo::ifgt(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        CInstructionInfo::ifle(target) => ifeq_wrapper(instruction, env, stack_frame, *target),
        CInstructionInfo::ifnonnull(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_ifnonnull(final_target as u16, env, stack_frame)
        }
        CInstructionInfo::ifnull(target) => {
            let final_target = (*target as isize) + (instruction.offset as isize);
            assert!(final_target >= 0);
            instruction_is_type_safe_ifnonnull(final_target as u16, env, stack_frame)
        }
        CInstructionInfo::iinc(iinc) => {
            instruction_is_type_safe_iinc(iinc.index as u16, env, stack_frame)
        }
        CInstructionInfo::iload(index) => {
            instruction_is_type_safe_iload(*index as u16, env, stack_frame)
        }
        CInstructionInfo::iload_0 => instruction_is_type_safe_iload(0, env, stack_frame),
        CInstructionInfo::iload_1 => instruction_is_type_safe_iload(1, env, stack_frame),
        CInstructionInfo::iload_2 => instruction_is_type_safe_iload(2, env, stack_frame),
        CInstructionInfo::iload_3 => instruction_is_type_safe_iload(3, env, stack_frame),
        CInstructionInfo::imul => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::ineg => instruction_is_type_safe_ineg(env, stack_frame),
        CInstructionInfo::instanceof(_) => instruction_is_type_safe_instanceof(env, stack_frame),
        CInstructionInfo::invokedynamic(cp) => {
            instruction_is_type_safe_invokedynamic(*cp as usize, env, stack_frame)
        }
        CInstructionInfo::invokeinterface {
            method_name,
            descriptor,
            classname_ref_type,
            count,
        } => instruction_is_type_safe_invokeinterface(
            *method_name,
            descriptor,
            classname_ref_type,
            count.get() as usize,
            env,
            stack_frame,
        ),
        CInstructionInfo::invokespecial {
            method_name,
            descriptor,
            classname_ref_type,
        } => instruction_is_type_safe_invokespecial(
            &CPDType::Ref(classname_ref_type.clone()),
            *method_name,
            descriptor,
            env,
            stack_frame,
        ),
        CInstructionInfo::invokestatic {
            method_name,
            descriptor,
            classname_ref_type: _,
        } => instruction_is_type_safe_invokestatic(*method_name, descriptor, env, stack_frame),
        CInstructionInfo::invokevirtual {
            method_name,
            descriptor,
            classname_ref_type,
        } => instruction_is_type_safe_invokevirtual(
            &CPDType::Ref(classname_ref_type.clone()),
            *method_name,
            descriptor,
            env,
            stack_frame,
        ),
        CInstructionInfo::ior => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::irem => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::ireturn => instruction_is_type_safe_ireturn(env, stack_frame),
        CInstructionInfo::ishl => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::ishr => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::istore(i) => instruction_is_type_safe_istore(*i as u16, env, stack_frame),
        CInstructionInfo::istore_0 => instruction_is_type_safe_istore(0, env, stack_frame),
        CInstructionInfo::istore_1 => instruction_is_type_safe_istore(1, env, stack_frame),
        CInstructionInfo::istore_2 => instruction_is_type_safe_istore(2, env, stack_frame),
        CInstructionInfo::istore_3 => instruction_is_type_safe_istore(3, env, stack_frame),
        CInstructionInfo::isub => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::iushr => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::ixor => instruction_is_type_safe_iadd(env, stack_frame),
        CInstructionInfo::jsr(_) => instruction_is_type_safe_nop(stack_frame),
        CInstructionInfo::jsr_w(_) => instruction_is_type_safe_nop(stack_frame),
        CInstructionInfo::l2d => instruction_is_type_safe_l2d(env, stack_frame),
        CInstructionInfo::l2f => instruction_is_type_safe_l2f(env, stack_frame),
        CInstructionInfo::l2i => instruction_is_type_safe_l2i(env, stack_frame),
        CInstructionInfo::ladd => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::laload => instruction_is_type_safe_laload(env, stack_frame),
        CInstructionInfo::land => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::lastore => instruction_is_type_safe_lastore(env, stack_frame),
        CInstructionInfo::lcmp => instruction_is_type_safe_lcmp(env, stack_frame),
        CInstructionInfo::lconst_0 => instruction_is_type_safe_lconst_0(env, stack_frame),
        CInstructionInfo::lconst_1 => instruction_is_type_safe_lconst_0(env, stack_frame),
        CInstructionInfo::ldc(cldc) => {
            instruction_is_type_safe_ldc_w(&cldc.as_ref().left().unwrap(), env, stack_frame)
        }
        CInstructionInfo::ldc_w(cp) => instruction_is_type_safe_ldc_w(cp, env, stack_frame),
        CInstructionInfo::ldc2_w(cldc2w) => {
            instruction_is_type_safe_ldc2_w(cldc2w, env, stack_frame)
        }
        CInstructionInfo::ldiv => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::lload(i) => instruction_is_type_safe_lload(*i as u16, env, stack_frame),
        CInstructionInfo::lload_0 => instruction_is_type_safe_lload(0, env, stack_frame),
        CInstructionInfo::lload_1 => instruction_is_type_safe_lload(1, env, stack_frame),
        CInstructionInfo::lload_2 => instruction_is_type_safe_lload(2, env, stack_frame),
        CInstructionInfo::lload_3 => instruction_is_type_safe_lload(3, env, stack_frame),
        CInstructionInfo::lmul => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::lneg => instruction_is_type_safe_lneg(env, stack_frame),
        CInstructionInfo::lookupswitch(s) => {
            let targets: Vec<u16> = s
                .pairs
                .iter()
                .map(|(_, x)| (offset as i32 + *x as i32) as u16)
                .collect();
            let keys = s.pairs.iter().map(|(x, _)| *x).collect();
            instruction_is_type_safe_lookupswitch(targets, keys, env, stack_frame)
        }
        CInstructionInfo::lor => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::lrem => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::lreturn => instruction_is_type_safe_lreturn(env, stack_frame),
        CInstructionInfo::lshl => instruction_is_type_safe_lshl(env, stack_frame),
        CInstructionInfo::lshr => instruction_is_type_safe_lshl(env, stack_frame),
        CInstructionInfo::lstore(i) => instruction_is_type_safe_lstore(*i as u16, env, stack_frame),
        CInstructionInfo::lstore_0 => instruction_is_type_safe_lstore(0, env, stack_frame),
        CInstructionInfo::lstore_1 => instruction_is_type_safe_lstore(1, env, stack_frame),
        CInstructionInfo::lstore_2 => instruction_is_type_safe_lstore(2, env, stack_frame),
        CInstructionInfo::lstore_3 => instruction_is_type_safe_lstore(3, env, stack_frame),
        CInstructionInfo::lsub => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::lushr => instruction_is_type_safe_lshl(env, stack_frame),
        CInstructionInfo::lxor => instruction_is_type_safe_ladd(env, stack_frame),
        CInstructionInfo::monitorenter => instruction_is_type_safe_monitorenter(env, stack_frame),
        CInstructionInfo::monitorexit => instruction_is_type_safe_monitorenter(env, stack_frame),
        CInstructionInfo::multianewarray {
            type_,
            dimensions: dimesions,
        } => instruction_is_type_safe_multianewarray(
            type_,
            dimesions.get() as usize,
            env,
            stack_frame,
        ),
        CInstructionInfo::new(_) => instruction_is_type_safe_new(offset, env, stack_frame),
        CInstructionInfo::newarray(type_code) => {
            instruction_is_type_safe_newarray(*type_code as usize, env, stack_frame)
        }
        CInstructionInfo::nop => instruction_is_type_safe_nop(stack_frame),
        CInstructionInfo::pop => instruction_is_type_safe_pop(env, stack_frame),
        CInstructionInfo::pop2 => instruction_is_type_safe_pop2(env, stack_frame),
        CInstructionInfo::putfield {
            name,
            desc,
            target_class,
        } => {
            instruction_is_type_safe_putfield((*target_class).into(), *name, desc, env, stack_frame)
        }
        CInstructionInfo::putstatic {
            name: _,
            desc,
            target_class: _,
        } => instruction_is_type_safe_putstatic(desc, env, stack_frame),
        CInstructionInfo::ret(_) => instruction_is_type_safe_nop(stack_frame),
        CInstructionInfo::return_ => instruction_is_type_safe_return(env, stack_frame),
        CInstructionInfo::saload => instruction_is_type_safe_saload(env, stack_frame),
        CInstructionInfo::sastore => instruction_is_type_safe_sastore(env, stack_frame),
        CInstructionInfo::sipush(_) => instruction_is_type_safe_sipush(env, stack_frame),
        CInstructionInfo::swap => instruction_is_type_safe_swap(env, stack_frame),
        CInstructionInfo::tableswitch(s) => {
            let mut targets = vec![];
            for o in &s.offsets {
                targets.push((offset as i32 + *o as i32) as u16)
            }
            targets.push((offset as i32 + s.default as i32) as u16);
            instruction_is_type_safe_tableswitch(targets, env, stack_frame)
        }
        CInstructionInfo::wide(wide) => match wide {
            Wide::Iload(WideIload { index }) => {
                instruction_is_type_safe_iload(*index as u16, env, stack_frame)
            }
            Wide::Fload(WideFload { index }) => {
                instruction_is_type_safe_fload(*index as u16, env, stack_frame)
            }
            Wide::Aload(WideAload { index }) => {
                instruction_is_type_safe_aload(*index as u16, env, stack_frame)
            }
            Wide::Lload(WideLload { index }) => {
                instruction_is_type_safe_lload(*index as u16, env, stack_frame)
            }
            Wide::Dload(WideDload { index }) => {
                instruction_is_type_safe_dload(*index as u16, env, stack_frame)
            }
            Wide::Istore(WideIstore { index }) => {
                instruction_is_type_safe_istore(*index as u16, env, stack_frame)
            }
            Wide::Fstore(WideFstore { index }) => {
                instruction_is_type_safe_fstore(*index as u16, env, stack_frame)
            }
            Wide::Astore(WideAstore { index }) => {
                instruction_is_type_safe_astore(*index as u16, env, stack_frame)
            }
            Wide::Lstore(WideLstore { index }) => {
                instruction_is_type_safe_lstore(*index as u16, env, stack_frame)
            }
            Wide::Dstore(WideDstore { index }) => {
                instruction_is_type_safe_dstore(*index as u16, env, stack_frame)
            }
            Wide::Ret(WideRet { index: _ }) => instruction_is_type_safe_nop(stack_frame),
            Wide::IInc(iinc) => instruction_is_type_safe_iinc(iinc.index as u16, env, stack_frame),
        },
        CInstructionInfo::EndOfCode => Result::Err(unknown_error_verifying!()),
    }
}

fn if_icmp_wrapper(
    instruction: &CInstruction,
    env: &Environment,
    stack_frame: Frame,
    target: i16,
) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let final_target = (target as isize) + (instruction.offset as isize);
    assert!(final_target >= 0);
    instruction_is_type_safe_if_icmpeq(final_target as u16, env, stack_frame)
}

fn ifeq_wrapper(
    instruction: &CInstruction,
    env: &Environment,
    stack_frame: Frame,
    target: i16,
) -> Result<InstructionTypeSafe, TypeSafetyError> {
    let final_target = (target as isize) + (instruction.offset as isize);
    assert!(final_target >= 0);
    instruction_is_type_safe_ifeq(final_target as u16, env, stack_frame)
}