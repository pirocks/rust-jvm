use std::ops::Deref;
use itertools::Either;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::ByteCodeOffset;
use rust_jvm_common::compressed_classfile::code::{CInstructionInfo, CompressedCode};
use rust_jvm_common::runtime_type::RuntimeType;
use crate::{JVMState};
use crate::function_instruction_count::FunctionExecutionCounter;
use crate::instructions::invoke::interface::invoke_interface;
use crate::instructions::invoke::special::invoke_special;
use crate::instructions::invoke::static_::run_invoke_static;
use crate::instructions::invoke::virtual_::invoke_virtual_instruction;
use crate::instructions::special::invoke_instanceof;
use crate::interpreter::arithmetic::{dadd, ddiv, dmul, dneg, drem, dsub, fadd, fdiv, fmul, fneg, frem, fsub, iadd, iand, idiv, imul, ineg, ior, irem, ishl, ishr, isub, iushr, ixor, ladd, land, lcmp, ldiv, lmul, lneg, lor, lrem, lshl, lshr, lsub, lushr, lxor};
use crate::interpreter::branch::{goto_, if_acmpeq, if_acmpne, if_icmpeq, if_icmpge, if_icmpgt, if_icmple, if_icmplt, if_icmpne, ifeq, ifge, ifgt, ifle, iflt, ifne, ifnonnull, ifnull};
use crate::interpreter::cmp::{dcmpg, dcmpl, fcmpg, fcmpl};
use crate::interpreter::consts::{aconst_null, bipush, dconst_0, dconst_1, fconst_0, fconst_1, fconst_2, iconst_0, iconst_1, iconst_2, iconst_3, iconst_4, iconst_5, iconst_m1, lconst, sipush};
use crate::interpreter::conversion::{d2f, d2i, d2l, f2d, f2i, i2b, i2c, i2d, i2f, i2l, i2s, l2f, l2i};
use crate::interpreter::dup::{dup, dup2, dup2_x1, dup_x1, dup_x2};
use crate::interpreter::fields::{get_field, get_static, putfield, putstatic};
use crate::interpreter::ldc::{ldc2_w, ldc_w};
use crate::interpreter::load::{aaload, aload, baload, caload, daload, dload, faload, fload, iaload, iload, laload, lload, saload};
use crate::interpreter::new::{anewarray, multi_a_new_array, new, newarray};
use crate::interpreter::pop::pop2;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::{InterpreterJavaValue, RealInterpreterStateGuard};
use crate::interpreter::special::{arraylength, checkcast};
use crate::interpreter::store::{aastore, astore, bastore, castore, dastore, dstore, fastore, fstore, iastore, istore, lastore, lstore, sastore};
use crate::interpreter::switch::{invoke_lookupswitch, tableswitch};
use crate::interpreter::throw::athrow;

pub fn run_single_instruction<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    interpreter_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>,
    instruct: &CInstructionInfo,
    function_counter: &FunctionExecutionCounter,
    method: &MethodView,
    code: &CompressedCode,
    current_pc: ByteCodeOffset,
) -> PostInstructionAction<'gc> {
    function_counter.increment();
    // dbg!(method.classview().name().jvm_representation(&jvm.string_pool));
    // dbg!(method.method_shape().to_jvm_representation(&jvm.string_pool));
    // if method.classview().name().unwrap_name() == CClassName::properties() || method.name().0.to_str(&jvm.string_pool) == "getProperty" {
    // if method.classview().name().unwrap_name().0.to_str(&jvm.string_pool) =="com/google/common/collect/StandardTable" &&
    //     method.name().0.to_str(&jvm.string_pool) == "put"
    //     {
        dump_frame(interpreter_state, method, code);
        eprintln!("{}", instruct.better_debug_string(&jvm.string_pool));
    // }
    // }
    // eprintln!("{}", instruct.better_debug_string(&jvm.string_pool));
    match instruct {
        CInstructionInfo::aload(n) => aload(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::aload_0 => aload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::aload_1 => aload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::aload_2 => aload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::aload_3 => aload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::aaload => aaload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::aastore => aastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::aconst_null => aconst_null(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        CInstructionInfo::areturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::object()).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::arraylength => {
            arraylength(jvm, interpreter_state.current_frame_mut())
        }
        CInstructionInfo::astore(n) => astore(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::astore_0 => astore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::astore_1 => astore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::astore_2 => astore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::astore_3 => astore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::athrow => athrow(jvm, interpreter_state),
        CInstructionInfo::baload => baload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::bastore => bastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::bipush(b) => bipush(jvm, interpreter_state.current_frame_mut(), *b),
        CInstructionInfo::caload => caload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::castore => castore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::checkcast(cp) => checkcast(jvm, interpreter_state, *cp),
        CInstructionInfo::d2f => d2f(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::d2i => d2i(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::d2l => d2l(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dadd => dadd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::daload => daload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dastore => dastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dcmpg => dcmpg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dcmpl => dcmpl(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dconst_0 => dconst_0(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dconst_1 => dconst_1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ddiv => ddiv(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dload(i) => dload(jvm, interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::dload_0 => dload(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::dload_1 => dload(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::dload_2 => dload(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::dload_3 => dload(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::dmul => dmul(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dneg => dneg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::drem => drem(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dreturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::DoubleType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::dstore(i) => dstore(jvm, interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::dstore_0 => dstore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::dstore_1 => dstore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::dstore_2 => dstore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::dstore_3 => dstore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::dsub => dsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup => dup(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup_x1 => dup_x1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup_x2 => dup_x2(jvm, interpreter_state.inner().current_frame().frame_view.ir_ref.method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::dup2 => dup2(jvm, interpreter_state.inner().current_frame().frame_view.ir_ref.method_id().unwrap(), interpreter_state.current_frame_mut(), current_pc),
        CInstructionInfo::dup2_x1 => dup2_x1(jvm, interpreter_state.inner().current_frame().frame_view.ir_ref.method_id().unwrap(), interpreter_state.current_frame_mut(),current_pc),
        // CInstructionInfo::dup2_x2 => dup2_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        CInstructionInfo::f2d => f2d(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::f2i => f2i(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::f2l => f2l(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fadd => fadd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::faload => faload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fastore => fastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fcmpg => fcmpg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fcmpl => fcmpl(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fconst_0 => fconst_0(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fconst_1 => fconst_1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fconst_2 => fconst_2(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fdiv => fdiv(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fload(n) => fload(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::fload_0 => fload(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::fload_1 => fload(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::fload_2 => fload(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::fload_3 => fload(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::fmul => fmul(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fneg => fneg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::frem => frem(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::freturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::FloatType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::fstore(i) => fstore(jvm, interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::fstore_0 => fstore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::fstore_1 => fstore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::fstore_2 => fstore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::fstore_3 => fstore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::fsub => fsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::getfield { desc, target_class, name } => get_field(jvm, interpreter_state.current_frame_mut(), *target_class, *name, desc),
        CInstructionInfo::getstatic { name, target_class, desc } => get_static(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::goto_(target) => goto_(jvm, *target as i32),
        CInstructionInfo::goto_w(target) => goto_(jvm, *target),
        CInstructionInfo::i2b => i2b(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2c => i2c(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2d => i2d(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2f => i2f(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2l => i2l(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2s => i2s(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iadd => iadd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iaload => iaload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iand => iand(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iastore => iastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_m1 => iconst_m1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_0 => iconst_0(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_1 => iconst_1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_2 => iconst_2(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_3 => iconst_3(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_4 => iconst_4(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_5 => iconst_5(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::idiv => idiv(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::if_acmpeq(offset) => if_acmpeq(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_acmpne(offset) => if_acmpne(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpeq(offset) => if_icmpeq(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpne(offset) => if_icmpne(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmplt(offset) => if_icmplt(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpge(offset) => if_icmpge(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmpgt(offset) => if_icmpgt(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::if_icmple(offset) => if_icmple(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifeq(offset) => ifeq(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifne(offset) => ifne(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::iflt(offset) => iflt(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifge(offset) => ifge(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifgt(offset) => ifgt(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifle(offset) => ifle(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifnonnull(offset) => ifnonnull(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::ifnull(offset) => ifnull(jvm, interpreter_state.current_frame_mut(), *offset),
        CInstructionInfo::iinc(iinc) => {
            let mut current_frame = interpreter_state.current_frame_mut();
            let val = current_frame.local_get(iinc.index, RuntimeType::IntType).unwrap_int();
            let res = val + iinc.const_ as i32;
            current_frame.local_set(iinc.index, InterpreterJavaValue::Int(res));
            PostInstructionAction::Next {}
        }
        CInstructionInfo::iload(n) => iload(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::iload_0 => iload(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::iload_1 => iload(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::iload_2 => iload(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::iload_3 => iload(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::imul => imul(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ineg => ineg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::instanceof(cp) => invoke_instanceof(jvm, interpreter_state.current_frame_mut(), *cp),
        // CInstructionInfo::invokedynamic(cp) => invoke_dynamic(jvm, interpreter_state, *cp),
        CInstructionInfo::invokeinterface { classname_ref_type, descriptor, method_name, count } => invoke_interface(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor, *count),
        CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => invoke_special(jvm, interpreter_state, classname_ref_type.unwrap_object_name(), *method_name, descriptor),
        CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => run_invoke_static(jvm, interpreter_state, method, code, classname_ref_type.clone(), *method_name, descriptor),
        CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type: _ } => {
            // dump_frame(interpreter_state,method,code);
            invoke_virtual_instruction(jvm, interpreter_state, *method_name, descriptor)
        }
        CInstructionInfo::ior => ior(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::irem => irem(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ireturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::IntType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::ishl => ishl(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ishr => ishr(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::istore(n) => istore(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::istore_0 => istore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::istore_1 => istore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::istore_2 => istore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::istore_3 => istore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::isub => isub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iushr => iushr(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ixor => ixor(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::jsr(target) => jsr(interpreter_state, *target as i32),
        // CInstructionInfo::jsr_w(target) => jsr(interpreter_state, *target),
        // CInstructionInfo::l2d => l2d(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::l2f => l2f(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::l2i => l2i(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ladd => ladd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::laload => laload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::land => land(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lastore => lastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lcmp => lcmp(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lconst_0 => lconst(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lconst_1 => lconst(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::ldc(cldc2w) => ldc_w(jvm, interpreter_state, &cldc2w.as_ref()),
        CInstructionInfo::ldc_w(cldcw) => ldc_w(jvm, interpreter_state, &Either::Left(cldcw)),
        CInstructionInfo::ldc2_w(cldc2w) => ldc2_w(jvm, interpreter_state, cldc2w),
        CInstructionInfo::ldiv => ldiv(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lload(i) => lload(jvm, interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::lload_0 => lload(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lload_1 => lload(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::lload_2 => lload(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::lload_3 => lload(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::lmul => lmul(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lneg => lneg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lor => lor(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lrem => lrem(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lreturn => {
            PostInstructionAction::Return { res: Some(interpreter_state.current_frame_mut().pop(RuntimeType::LongType).to_new_java_handle(jvm)) }
        }
        CInstructionInfo::lshl => lshl(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lshr => lshr(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lstore(n) => lstore(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::lstore_0 => lstore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lstore_1 => lstore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::lstore_2 => lstore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::lstore_3 => lstore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::lsub => lsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lushr => lushr(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lxor => lxor(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::monitorenter => {
            let obj = interpreter_state.current_frame_mut().pop(RuntimeType::object());
            // gc_managed_object.monitor_lock(jvm, interpreter_state);
            PostInstructionAction::Next {}
        }
        CInstructionInfo::monitorexit => {
            let obj = interpreter_state.current_frame_mut().pop(RuntimeType::object());
            // interpreter_state.current_frame_mut().pop(Some(RuntimeType::object())).unwrap_object_nonnull().monitor_unlock(jvm, interpreter_state);
            PostInstructionAction::Next {}
        }
        CInstructionInfo::multianewarray { type_, dimensions } => multi_a_new_array(jvm, interpreter_state, dimensions.get(), *type_),
        CInstructionInfo::new(cn) => new(jvm, interpreter_state, *cn),
        CInstructionInfo::newarray(a_type) => newarray(jvm, interpreter_state, *a_type),
        CInstructionInfo::nop => {
            PostInstructionAction::Next {}
        }
        CInstructionInfo::pop => {
            interpreter_state.current_frame_mut().pop(RuntimeType::LongType);
            PostInstructionAction::Next {}
        }
        CInstructionInfo::pop2 => pop2(jvm, interpreter_state.inner().current_frame().frame_view.ir_ref.method_id().unwrap(), interpreter_state.current_frame_mut(),current_pc),
        CInstructionInfo::putfield { name, desc, target_class } => putfield(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::putstatic { name, desc, target_class } => putstatic(jvm, interpreter_state, *target_class, *name, desc),
        // CInstructionInfo::ret(local_var_index) => ret(jvm, interpreter_state.current_frame_mut(), *local_var_index as u16),
        // CInstructionInfo::return_ => return_(interpreter_state),
        CInstructionInfo::saload => saload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::sastore => sastore(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::sipush(val) => sipush(jvm, interpreter_state.current_frame_mut(), *val),
        // CInstructionInfo::swap => swap(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::tableswitch(switch) => tableswitch(switch.deref(), jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::wide(w) => wide(jvm, interpreter_state.current_frame_mut(), w),
        // CInstructionInfo::EndOfCode => panic!(),
        CInstructionInfo::return_ => {
            PostInstructionAction::Return { res: None }
        }
        instruct => {
            dbg!(instruct);
            todo!()
        }
    }
}

pub fn dump_frame(interpreter_state: &mut RealInterpreterStateGuard, method: &MethodView, code: &CompressedCode) {
    let local_var_slots = method.local_var_slots();
    eprint!("Local Vars:");
    for i in 0..local_var_slots {
        let raw = interpreter_state.current_frame_mut().local_get(i, RuntimeType::object()).to_raw();
        eprint!(" {:X}", raw);
    }
    eprintln!();
    eprint!("Operand Stack:");
    for i in 0..interpreter_state.current_stack_depth_from_start {
        let raw = interpreter_state.current_frame_mut().operand_stack_get(i, RuntimeType::object()).to_raw();
        eprint!(" {:X}", raw);
    }
    eprintln!();
}
