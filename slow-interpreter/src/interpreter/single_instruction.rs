use rust_jvm_common::compressed_classfile::code::CInstructionInfo;
use crate::{JVMState};
use crate::function_instruction_count::FunctionExecutionCounter;
use crate::instructions::invoke::special::invoke_special;
use crate::instructions::invoke::static_::run_invoke_static;
use crate::instructions::invoke::virtual_::invoke_virtual_instruction;
use crate::interpreter::consts::{aconst_null, iconst_0, iconst_1, iconst_2, iconst_3, iconst_4, iconst_5, iconst_m1};
use crate::interpreter::dup::dup;
use crate::interpreter::fields::{putfield, putstatic};
use crate::interpreter::ldc::ldc_w;
use crate::interpreter::load::aload;
use crate::interpreter::new::{anewarray, new};
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;

pub fn run_single_instruction<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    interpreter_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>,
    instruct: &CInstructionInfo,
    function_counter: &FunctionExecutionCounter,
) -> PostInstructionAction<'gc> {
    function_counter.increment();
    match instruct {
        CInstructionInfo::aload(n) => aload(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::aload_0 => aload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::aload_1 => aload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::aload_2 => aload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::aload_3 => aload(interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::aaload => aaload(jvm, interpreter_state),
        // CInstructionInfo::aastore => aastore(jvm, interpreter_state),
        CInstructionInfo::aconst_null => aconst_null(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        // CInstructionInfo::areturn => areturn(jvm, interpreter_state),
        // CInstructionInfo::arraylength => arraylength(jvm, interpreter_state),
        // CInstructionInfo::astore(n) => astore(interpreter_state.current_frame_mut(), *n as u16),
        // CInstructionInfo::astore_0 => astore(interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::astore_1 => astore(interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::astore_2 => astore(interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::astore_3 => astore(interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::athrow => athrow(jvm, interpreter_state),
        // CInstructionInfo::baload => baload(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::bastore => bastore(jvm, interpreter_state),
        // CInstructionInfo::bipush(b) => bipush(jvm, interpreter_state.current_frame_mut(), *b),
        // CInstructionInfo::caload => caload(jvm, interpreter_state),
        // CInstructionInfo::castore => castore(jvm, interpreter_state),
        // CInstructionInfo::checkcast(cp) => invoke_checkcast(jvm, interpreter_state, cp),
        // CInstructionInfo::d2f => d2f(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::d2i => d2i(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::d2l => d2l(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dadd => dadd(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::daload => daload(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dastore => dastore(jvm, interpreter_state),
        // CInstructionInfo::dcmpg => dcmpg(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dcmpl => dcmpl(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dconst_0 => dconst_0(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dconst_1 => dconst_1(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::ddiv => ddiv(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dload(i) => dload(jvm, interpreter_state.current_frame_mut(), *i as u16),
        // CInstructionInfo::dload_0 => dload(jvm, interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::dload_1 => dload(jvm, interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::dload_2 => dload(jvm, interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::dload_3 => dload(jvm, interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::dmul => dmul(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dneg => dneg(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::drem => drem(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dreturn => dreturn(jvm, interpreter_state),
        // CInstructionInfo::dstore(i) => dstore(jvm, interpreter_state.current_frame_mut(), *i as u16),
        // CInstructionInfo::dstore_0 => dstore(jvm, interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::dstore_1 => dstore(jvm, interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::dstore_2 => dstore(jvm, interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::dstore_3 => dstore(jvm, interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::dsub => dsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup => dup(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dup_x1 => dup_x1(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dup_x2 => dup_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dup2 => dup2(jvm, method_id, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dup2_x1 => dup2_x1(jvm, method_id, interpreter_state.current_frame_mut()),
        // CInstructionInfo::dup2_x2 => dup2_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        // CInstructionInfo::f2d => f2d(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::f2i => f2i(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::f2l => f2l(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fadd => fadd(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::faload => faload(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fastore => fastore(jvm, interpreter_state),
        // CInstructionInfo::fcmpg => fcmpg(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fcmpl => fcmpl(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fconst_0 => fconst_0(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fconst_1 => fconst_1(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fconst_2 => fconst_2(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fdiv => fdiv(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fload(n) => fload(jvm, interpreter_state.current_frame_mut(), *n as u16),
        // CInstructionInfo::fload_0 => fload(jvm, interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::fload_1 => fload(jvm, interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::fload_2 => fload(jvm, interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::fload_3 => fload(jvm, interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::fmul => fmul(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::fneg => fneg(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::frem => frem(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::freturn => freturn(jvm, interpreter_state),
        // CInstructionInfo::fstore(i) => fstore(jvm, interpreter_state.current_frame_mut(), *i as u16),
        // CInstructionInfo::fstore_0 => fstore(jvm, interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::fstore_1 => fstore(jvm, interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::fstore_2 => fstore(jvm, interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::fstore_3 => fstore(jvm, interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::fsub => fsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::getfield { desc, target_class, name } => get_field(jvm, interpreter_state, *target_class, *name, desc, false),
        // CInstructionInfo::getstatic { name, target_class, desc } => get_static(jvm, interpreter_state, *target_class, *name, desc),
        // CInstructionInfo::goto_(target) => goto_(jvm, interpreter_state.current_frame_mut(), *target as i32),
        // CInstructionInfo::goto_w(target) => goto_(jvm, interpreter_state.current_frame_mut(), *target),
        // CInstructionInfo::i2b => i2b(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::i2c => i2c(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::i2d => i2d(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::i2f => i2f(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::i2l => i2l(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::i2s => i2s(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::iadd => iadd(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::iaload => iaload(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::iand => iand(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::iastore => iastore(jvm, interpreter_state),
        CInstructionInfo::iconst_m1 => iconst_m1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_0 => iconst_0(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_1 => iconst_1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_2 => iconst_2(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_3 => iconst_3(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_4 => iconst_4(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iconst_5 => iconst_5(jvm, interpreter_state.current_frame_mut()),
        /*
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
            let val = current_frame.local_vars().get(iinc.index, RuntimeType::IntType).unwrap_int();
            let res = val + iinc.const_ as i32;
            current_frame.local_vars_mut().set(iinc.index, JavaValue::Int(res));
        }
        CInstructionInfo::iload(n) => iload(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::iload_0 => iload(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::iload_1 => iload(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::iload_2 => iload(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::iload_3 => iload(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::imul => imul(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ineg => ineg(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::instanceof(cp) => invoke_instanceof(jvm, interpreter_state, cp),
        CInstructionInfo::invokedynamic(cp) => invoke_dynamic(jvm, interpreter_state, *cp),
        CInstructionInfo::invokeinterface { classname_ref_type, descriptor, method_name, count } => invoke_interface(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor, *count),*/
        CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => invoke_special(jvm, interpreter_state, classname_ref_type.unwrap_object_name(), *method_name, descriptor),
        CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => run_invoke_static(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor),
        CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type: _ } => invoke_virtual_instruction(jvm, interpreter_state, *method_name, descriptor),
        /*CInstructionInfo::ior => ior(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::irem => irem(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ireturn => ireturn(jvm, interpreter_state),
        CInstructionInfo::ishl => ishl(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ishr => ishr(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::istore(n) => istore(jvm, interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::istore_0 => istore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::istore_1 => istore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::istore_2 => istore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::istore_3 => istore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::isub => isub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iushr => iushr(interpreter_state),
        CInstructionInfo::ixor => ixor(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::jsr(target) => jsr(interpreter_state, *target as i32),
        CInstructionInfo::jsr_w(target) => jsr(interpreter_state, *target),
        CInstructionInfo::l2d => l2d(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::l2f => l2f(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::l2i => l2i(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::ladd => ladd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::laload => laload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::land => land(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lastore => lastore(jvm, interpreter_state),
        CInstructionInfo::lcmp => lcmp(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::lconst_0 => lconst(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::lconst_1 => lconst(jvm, interpreter_state.current_frame_mut(), 1),*/
        CInstructionInfo::ldc(cldc2w) => ldc_w(jvm, interpreter_state, &cldc2w.as_ref()),
        // CInstructionInfo::ldc_w(cldcw) => ldc_w(jvm, interpreter_state, &Either::Left(cldcw)),
        // CInstructionInfo::ldc2_w(cldc2w) => ldc2_w(jvm, interpreter_state.current_frame_mut(), cldc2w),
        // CInstructionInfo::ldiv => ldiv(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lload(i) => lload(jvm, interpreter_state.current_frame_mut(), *i as u16),
        // CInstructionInfo::lload_0 => lload(jvm, interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::lload_1 => lload(jvm, interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::lload_2 => lload(jvm, interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::lload_3 => lload(jvm, interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::lmul => lmul(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lneg => lneg(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lor => lor(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lrem => lrem(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lreturn => lreturn(jvm, interpreter_state),
        // CInstructionInfo::lshl => lshl(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lshr => lshr(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lstore(n) => lstore(jvm, interpreter_state.current_frame_mut(), *n as u16),
        // CInstructionInfo::lstore_0 => lstore(jvm, interpreter_state.current_frame_mut(), 0),
        // CInstructionInfo::lstore_1 => lstore(jvm, interpreter_state.current_frame_mut(), 1),
        // CInstructionInfo::lstore_2 => lstore(jvm, interpreter_state.current_frame_mut(), 2),
        // CInstructionInfo::lstore_3 => lstore(jvm, interpreter_state.current_frame_mut(), 3),
        // CInstructionInfo::lsub => lsub(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lushr => lushr(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::lxor => lxor(jvm, interpreter_state.current_frame_mut()),
        // CInstructionInfo::monitorenter => {
        //     let mut stack_entry_mut: StackEntryMut<'gc_life, '_> = interpreter_state.current_frame_mut();
        //     let popped: JavaValue<'gc_life> = stack_entry_mut.pop(Some(RuntimeType::object()));
        //     let gc_managed_object: GcManagedObject<'gc_life> = popped.unwrap_object_nonnull();
        //     gc_managed_object.monitor_lock(jvm, interpreter_state);
        // }
        // CInstructionInfo::monitorexit => {
        //     interpreter_state.current_frame_mut().pop(Some(RuntimeType::object())).unwrap_object_nonnull().monitor_unlock(jvm, interpreter_state);
        // }
        // CInstructionInfo::multianewarray { type_, dimensions } => multi_a_new_array(jvm, interpreter_state, dimensions.get(), type_),
        CInstructionInfo::new(cn) => new(jvm, interpreter_state, *cn),
        /*CInstructionInfo::newarray(a_type) => newarray(jvm, interpreter_state, *a_type),
        CInstructionInfo::nop => {}
        CInstructionInfo::pop => pop(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::pop2 => pop2(jvm, method_id, interpreter_state.current_frame_mut()),*/
        CInstructionInfo::putfield { name, desc, target_class } => putfield(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::putstatic { name, desc, target_class } => putstatic(jvm, interpreter_state, *target_class, *name, desc),
        /*CInstructionInfo::ret(local_var_index) => ret(jvm, interpreter_state.current_frame_mut(), *local_var_index as u16),
        CInstructionInfo::return_ => return_(interpreter_state),
        CInstructionInfo::saload => saload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::sastore => sastore(jvm, interpreter_state),
        CInstructionInfo::sipush(val) => sipush(jvm, interpreter_state.current_frame_mut(), *val),
        CInstructionInfo::swap => swap(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::tableswitch(switch) => tableswitch(switch.deref(), jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::wide(w) => wide(jvm, interpreter_state.current_frame_mut(), w),
        CInstructionInfo::EndOfCode => panic!(),*/
        CInstructionInfo::return_ => {
            PostInstructionAction::Return { res: None }
        }
        instruct => {
            dbg!(instruct);
            todo!()
        }
    }
}
