use std::ops::Rem;
use std::sync::Arc;

use num::Zero;

use classfile_parser::code::{CodeParserContext, parse_instruction};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::JVM_ACC_SYNCHRONIZED;
use rust_jvm_common::classfile::{Code, InstructionInfo};

use crate::class_loading::check_resolved_class;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::arithmetic::*;
use crate::instructions::branch::*;
use crate::instructions::cmp::*;
use crate::instructions::constant::*;
use crate::instructions::conversion::*;
use crate::instructions::dup::*;
use crate::instructions::fields::*;
use crate::instructions::invoke::dynamic::invoke_dynamic;
use crate::instructions::invoke::interface::invoke_interface;
use crate::instructions::invoke::special::invoke_special;
use crate::instructions::invoke::static_::run_invoke_static;
use crate::instructions::invoke::virtual_::invoke_virtual_instruction;
use crate::instructions::ldc::*;
use crate::instructions::load::*;
use crate::instructions::new::*;
use crate::instructions::pop::{pop, pop2};
use crate::instructions::return_::*;
use crate::instructions::special::*;
use crate::instructions::store::*;
use crate::instructions::switch::*;
use crate::interpreter_state::InterpreterStateGuard;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::stack_entry::StackEntry;
use crate::threading::monitors::Monitor;

#[derive(Debug)]
pub struct WasException;

pub fn run_function(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
    let view = interpreter_state.current_class_view().clone();
    let method_i = interpreter_state.current_method_i();
    let method = view.method_view_i(method_i as usize);
    let synchronized = method.access_flags() & JVM_ACC_SYNCHRONIZED as u16 > 0;
    let code = method.code_attribute().unwrap();
    let meth_name = method.name();
    let class_name__ = view.type_();

    let method_desc = method.desc_str();
    let current_depth = interpreter_state.call_stack_depth();
    let current_thread_tid = jvm.thread_state.try_get_current_thread().map(|t| t.java_tid).unwrap_or(-1);
    let function_enter_guard = jvm.tracing.trace_function_enter(&class_name__, &meth_name, &method_desc, current_depth, current_thread_tid);
    assert!(!*interpreter_state.function_return_mut());
    let class_pointer = interpreter_state.current_frame().class_pointer();
    let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
    //so figuring out which monitor to use is prob not this funcitions problem, like its already quite busy
    let monitor = monitor_for_function(jvm, interpreter_state, &method, synchronized);


    while !*interpreter_state.function_return() && interpreter_state.throw().is_none() {
        let (instruct, instruction_size) = current_instruction(interpreter_state.current_frame_mut(), &code);
        *interpreter_state.current_pc_offset_mut() = instruction_size as isize;
        breakpoint_check(jvm, interpreter_state, method_id);
        if let Ok(()) = safepoint_check(jvm, interpreter_state) {
            run_single_instruction(jvm, interpreter_state, instruct);
        };
        if interpreter_state.throw().is_some() {
            let throw_class = interpreter_state.throw().as_ref().unwrap().unwrap_normal_object().class_pointer.clone();
            for excep_table in &code.exception_table {
                let pc = interpreter_state.current_pc();
                if excep_table.start_pc as usize <= pc && pc < (excep_table.end_pc as usize) {//todo exclusive
                    if excep_table.catch_type == 0 {
                        //todo dup
                        interpreter_state.push_current_operand_stack(JavaValue::Object(interpreter_state.throw()));
                        interpreter_state.set_throw(None);
                        *interpreter_state.current_pc_mut() = excep_table.handler_pc as usize;
                        // println!("Caught Exception:{}", &throw_class.view().name().get_referred_name());
                        break;
                    } else {
                        let catch_runtime_name = interpreter_state.current_class_view().constant_pool_view(excep_table.catch_type as usize).unwrap_class().class_ref_type().unwrap_name();
                        let saved_throw = interpreter_state.throw().clone();
                        interpreter_state.set_throw(None);
                        let catch_class = check_resolved_class(jvm, interpreter_state, catch_runtime_name.into())?;
                        interpreter_state.set_throw(saved_throw);
                        if inherits_from(jvm, interpreter_state, &throw_class, &catch_class)? {
                            interpreter_state.push_current_operand_stack(JavaValue::Object(interpreter_state.throw()));
                            interpreter_state.set_throw(None);
                            *interpreter_state.current_pc_mut() = excep_table.handler_pc as usize;
                            // println!("Caught Exception:{}", throw_class.view().name().get_referred_name());
                            break;
                        }
                    }
                }
            }
            if interpreter_state.throw().is_some() {
                //need to propogate to caller
                break;
            }
        } else {
            //todo need to figure out where return res ends up on next stack
            update_pc_for_next_instruction(interpreter_state);
        }
    }
    if synchronized {//todo synchronize better so that natives are synced
        monitor.unwrap().unlock(jvm);
    }
    jvm.tracing.function_exit_guard(function_enter_guard);
    if interpreter_state.throw().is_some() {
        return Err(WasException);
    }
    Ok(())
}

pub fn safepoint_check(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) -> Result<(), WasException> {
    interpreter_state.thread.safepoint_state.check(jvm, interpreter_state)
}

fn update_pc_for_next_instruction(interpreter_state: &mut InterpreterStateGuard) {
    let offset = interpreter_state.current_pc_offset();
    let mut pc = interpreter_state.current_pc();
    if offset > 0 {
        pc += offset as usize;
    } else {
        pc -= (-offset) as usize;//todo perhaps i don't have to do this bs if I use u64 instead of usize
    }
    *interpreter_state.current_pc_mut() = pc;
}

fn breakpoint_check(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard, methodid: MethodId) {
    let pc = *interpreter_state.current_pc_mut() as isize;
    let stop = match &jvm.jvmti_state {
        None => false,
        Some(jvmti) => {
            let breakpoints = &jvmti.break_points.read().unwrap();
            let function_breakpoints = breakpoints.get(&methodid);
            function_breakpoints.map(|points| {
                points.contains(&pc)
            }).unwrap_or(false)
        }
    };
    if stop {
        let jdwp = &jvm.jvmti_state.as_ref().unwrap().built_in_jdwp;
        jdwp.breakpoint(jvm, methodid, pc as i64, interpreter_state);
    }
}

fn current_instruction(current_frame: &StackEntry, code: &Code) -> (InstructionInfo, usize) {
    let current = &code.code_raw[current_frame.pc()..];
    let mut context = CodeParserContext { offset: current_frame.pc(), iter: current.iter() };
    let parsedq = parse_instruction(&mut context).expect("but this parsed the first time round");
    (parsedq, context.offset - current_frame.pc())
}

pub fn monitor_for_function(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    method: &MethodView,
    synchronized: bool,
) -> Option<Arc<Monitor>> {
    if synchronized {
        let monitor = if method.is_static() {
            let class_object = get_or_create_class_object(
                jvm,
                method.classview().type_(),
                int_state,
            ).unwrap();
            class_object.unwrap_normal_object().monitor.clone()
        } else {
            int_state.current_frame_mut().local_vars()[0].unwrap_normal_object().monitor.clone()
        };
        monitor.lock(jvm);
        monitor.into()
    } else {
        None
    }
}

fn run_single_instruction(
    jvm: &JVMState,
    interpreter_state: &mut InterpreterStateGuard,
    instruct: InstructionInfo,
) {
    match instruct {
        InstructionInfo::aaload => aaload(interpreter_state),
        InstructionInfo::aastore => aastore(jvm, interpreter_state),
        InstructionInfo::aconst_null => aconst_null(interpreter_state.current_frame_mut()),
        InstructionInfo::aload(n) => aload(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::aload_0 => aload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::aload_1 => aload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::aload_2 => aload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::aload_3 => aload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        InstructionInfo::areturn => areturn(jvm, interpreter_state),
        InstructionInfo::arraylength => arraylength(jvm, interpreter_state),
        InstructionInfo::astore(n) => astore(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::astore_0 => astore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::astore_1 => astore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::astore_2 => astore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::astore_3 => astore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::athrow => athrow(jvm, interpreter_state),
        InstructionInfo::baload => baload(interpreter_state.current_frame_mut()),
        InstructionInfo::bastore => bastore(jvm, interpreter_state),
        InstructionInfo::bipush(b) => bipush(interpreter_state.current_frame_mut(), b),
        InstructionInfo::caload => caload(jvm, interpreter_state),
        InstructionInfo::castore => castore(jvm, interpreter_state),
        InstructionInfo::checkcast(cp) => invoke_checkcast(jvm, interpreter_state, cp),
        InstructionInfo::d2f => d2f(interpreter_state.current_frame_mut()),
        InstructionInfo::d2i => d2i(interpreter_state.current_frame_mut()),
        InstructionInfo::d2l => d2l(interpreter_state.current_frame_mut()),
        InstructionInfo::dadd => dadd(interpreter_state.current_frame_mut()),
        InstructionInfo::daload => daload(interpreter_state.current_frame_mut()),
        InstructionInfo::dastore => dastore(jvm, interpreter_state),
        InstructionInfo::dcmpg => dcmpg(interpreter_state.current_frame_mut()),
        InstructionInfo::dcmpl => dcmpl(interpreter_state.current_frame_mut()),
        InstructionInfo::dconst_0 => dconst_0(interpreter_state.current_frame_mut()),
        InstructionInfo::dconst_1 => dconst_1(interpreter_state.current_frame_mut()),
        InstructionInfo::ddiv => ddiv(interpreter_state.current_frame_mut()),
        InstructionInfo::dload(i) => dload(interpreter_state.current_frame_mut(), i as usize),
        InstructionInfo::dload_0 => dload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::dload_1 => dload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::dload_2 => dload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::dload_3 => dload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::dmul => dmul(interpreter_state.current_frame_mut()),
        InstructionInfo::dneg => dneg(interpreter_state.current_frame_mut()),
        InstructionInfo::drem => drem(interpreter_state.current_frame_mut()),
        InstructionInfo::dreturn => dreturn(jvm, interpreter_state),
        InstructionInfo::dstore(i) => dstore(interpreter_state.current_frame_mut(), i as usize),
        InstructionInfo::dstore_0 => dstore(interpreter_state.current_frame_mut(), 0 as usize),
        InstructionInfo::dstore_1 => dstore(interpreter_state.current_frame_mut(), 1 as usize),
        InstructionInfo::dstore_2 => dstore(interpreter_state.current_frame_mut(), 2 as usize),
        InstructionInfo::dstore_3 => dstore(interpreter_state.current_frame_mut(), 3 as usize),
        InstructionInfo::dsub => dsub(interpreter_state.current_frame_mut()),
        InstructionInfo::dup => dup(interpreter_state.current_frame_mut()),
        InstructionInfo::dup_x1 => dup_x1(interpreter_state.current_frame_mut()),
        InstructionInfo::dup_x2 => dup_x2(interpreter_state.current_frame_mut()),
        InstructionInfo::dup2 => dup2(interpreter_state.current_frame_mut()),
        InstructionInfo::dup2_x1 => dup2_x1(interpreter_state.current_frame_mut()),
        InstructionInfo::dup2_x2 => dup2_x2(interpreter_state.current_frame_mut()),
        InstructionInfo::f2d => f2d(interpreter_state.current_frame_mut()),
        InstructionInfo::f2i => f2i(interpreter_state.current_frame_mut()),
        InstructionInfo::f2l => f2l(interpreter_state.current_frame_mut()),
        InstructionInfo::fadd => fadd(interpreter_state.current_frame_mut()),
        InstructionInfo::faload => faload(interpreter_state.current_frame_mut()),
        InstructionInfo::fastore => fastore(jvm, interpreter_state),
        InstructionInfo::fcmpg => fcmpg(interpreter_state.current_frame_mut()),
        InstructionInfo::fcmpl => fcmpl(interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_0 => fconst_0(interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_1 => fconst_1(interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_2 => fconst_2(interpreter_state.current_frame_mut()),
        InstructionInfo::fdiv => fdiv(interpreter_state.current_frame_mut()),
        InstructionInfo::fload(n) => fload(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::fload_0 => fload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::fload_1 => fload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::fload_2 => fload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::fload_3 => fload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::fmul => fmul(interpreter_state.current_frame_mut()),
        InstructionInfo::fneg => fneg(interpreter_state.current_frame_mut()),
        InstructionInfo::frem => frem(interpreter_state.current_frame_mut()),
        InstructionInfo::freturn => freturn(jvm, interpreter_state),
        InstructionInfo::fstore(i) => fstore(interpreter_state.current_frame_mut(), i as usize),
        InstructionInfo::fstore_0 => fstore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::fstore_1 => fstore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::fstore_2 => fstore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::fstore_3 => fstore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::fsub => fsub(interpreter_state.current_frame_mut()),
        InstructionInfo::getfield(cp) => get_field(interpreter_state, cp, false),
        InstructionInfo::getstatic(cp) => get_static(jvm, interpreter_state, cp),
        InstructionInfo::goto_(target) => goto_(interpreter_state.current_frame_mut(), target as i32),
        InstructionInfo::goto_w(target) => goto_(interpreter_state.current_frame_mut(), target),
        InstructionInfo::i2b => i2b(interpreter_state.current_frame_mut()),
        InstructionInfo::i2c => i2c(interpreter_state.current_frame_mut()),
        InstructionInfo::i2d => i2d(interpreter_state.current_frame_mut()),
        InstructionInfo::i2f => i2f(interpreter_state.current_frame_mut()),
        InstructionInfo::i2l => i2l(interpreter_state.current_frame_mut()),
        InstructionInfo::i2s => i2s(interpreter_state.current_frame_mut()),
        InstructionInfo::iadd => iadd(interpreter_state.current_frame_mut()),
        InstructionInfo::iaload => iaload(interpreter_state.current_frame_mut()),
        InstructionInfo::iand => iand(interpreter_state.current_frame_mut()),
        InstructionInfo::iastore => iastore(jvm, interpreter_state),
        InstructionInfo::iconst_m1 => iconst_m1(interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_0 => iconst_0(interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_1 => iconst_1(interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_2 => iconst_2(interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_3 => iconst_3(interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_4 => iconst_4(interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_5 => iconst_5(interpreter_state.current_frame_mut()),
        InstructionInfo::idiv => idiv(interpreter_state.current_frame_mut()),
        InstructionInfo::if_acmpeq(offset) => if_acmpeq(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_acmpne(offset) => if_acmpne(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpeq(offset) => if_icmpeq(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpne(offset) => if_icmpne(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmplt(offset) => if_icmplt(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpge(offset) => if_icmpge(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpgt(offset) => if_icmpgt(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmple(offset) => if_icmple(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifeq(offset) => ifeq(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifne(offset) => ifne(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::iflt(offset) => iflt(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifge(offset) => ifge(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifgt(offset) => ifgt(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifle(offset) => ifle(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifnonnull(offset) => ifnonnull(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifnull(offset) => ifnull(interpreter_state.current_frame_mut(), offset),
        InstructionInfo::iinc(iinc) => {
            let current_frame = interpreter_state.current_frame_mut();
            let val = current_frame.local_vars()[iinc.index as usize].unwrap_int();
            let res = val + iinc.const_ as i32;
            current_frame.local_vars_mut()[iinc.index as usize] = JavaValue::Int(res);
        }
        InstructionInfo::iload(n) => iload(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::iload_0 => iload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::iload_1 => iload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::iload_2 => iload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::iload_3 => iload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::imul => imul(interpreter_state.current_frame_mut()),
        InstructionInfo::ineg => ineg(interpreter_state.current_frame_mut()),
        InstructionInfo::instanceof(cp) => invoke_instanceof(jvm, interpreter_state, cp),
        InstructionInfo::invokedynamic(cp) => {
            invoke_dynamic(jvm, interpreter_state, cp)
        }
        InstructionInfo::invokeinterface(invoke_i) => invoke_interface(jvm, interpreter_state, invoke_i),
        InstructionInfo::invokespecial(cp) => invoke_special(jvm, interpreter_state, cp),
        InstructionInfo::invokestatic(cp) => run_invoke_static(jvm, interpreter_state, cp),
        InstructionInfo::invokevirtual(cp) => invoke_virtual_instruction(jvm, interpreter_state, cp),
        InstructionInfo::ior => ior(interpreter_state.current_frame_mut()),
        InstructionInfo::irem => irem(interpreter_state.current_frame_mut()),
        InstructionInfo::ireturn => ireturn(jvm, interpreter_state),
        InstructionInfo::ishl => ishl(interpreter_state.current_frame_mut()),
        InstructionInfo::ishr => ishr(interpreter_state.current_frame_mut()),
        InstructionInfo::istore(n) => istore(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::istore_0 => istore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::istore_1 => istore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::istore_2 => istore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::istore_3 => istore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::isub => isub(interpreter_state.current_frame_mut()),
        InstructionInfo::iushr => iushr(interpreter_state),
        InstructionInfo::ixor => ixor(interpreter_state.current_frame_mut()),
        InstructionInfo::jsr(target) => jsr(interpreter_state, target as i32),
        InstructionInfo::jsr_w(target) => jsr(interpreter_state, target),
        InstructionInfo::l2d => l2d(interpreter_state.current_frame_mut()),
        InstructionInfo::l2f => l2f(interpreter_state.current_frame_mut()),
        InstructionInfo::l2i => l2i(interpreter_state.current_frame_mut()),
        InstructionInfo::ladd => ladd(interpreter_state.current_frame_mut()),
        InstructionInfo::laload => laload(interpreter_state.current_frame_mut()),
        InstructionInfo::land => land(interpreter_state.current_frame_mut()),
        InstructionInfo::lastore => lastore(jvm, interpreter_state),
        InstructionInfo::lcmp => lcmp(interpreter_state.current_frame_mut()),
        InstructionInfo::lconst_0 => lconst(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::lconst_1 => lconst(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::ldc(cp) => ldc_w(jvm, interpreter_state, cp as u16),
        InstructionInfo::ldc_w(cp) => ldc_w(jvm, interpreter_state, cp),
        InstructionInfo::ldc2_w(cp) => ldc2_w(interpreter_state.current_frame_mut(), cp),
        InstructionInfo::ldiv => ldiv(interpreter_state.current_frame_mut()),
        InstructionInfo::lload(i) => lload(interpreter_state.current_frame_mut(), i as usize),
        InstructionInfo::lload_0 => lload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::lload_1 => lload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::lload_2 => lload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::lload_3 => lload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::lmul => lmul(interpreter_state.current_frame_mut()),
        InstructionInfo::lneg => lneg(interpreter_state.current_frame_mut()),
        InstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, interpreter_state.current_frame_mut()),
        InstructionInfo::lor => lor(interpreter_state.current_frame_mut()),
        InstructionInfo::lrem => lrem(interpreter_state.current_frame_mut()),
        InstructionInfo::lreturn => lreturn(jvm, interpreter_state),
        InstructionInfo::lshl => lshl(interpreter_state.current_frame_mut()),
        InstructionInfo::lshr => lshr(interpreter_state.current_frame_mut()),
        InstructionInfo::lstore(n) => lstore(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::lstore_0 => lstore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::lstore_1 => lstore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::lstore_2 => lstore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::lstore_3 => lstore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::lsub => lsub(interpreter_state.current_frame_mut()),
        InstructionInfo::lushr => lushr(interpreter_state.current_frame_mut()),
        InstructionInfo::lxor => lxor(interpreter_state.current_frame_mut()),
        InstructionInfo::monitorenter => {
            interpreter_state.current_frame_mut().pop().unwrap_object_nonnull().monitor_lock(jvm);
        }
        InstructionInfo::monitorexit => {
            interpreter_state.current_frame_mut().pop().unwrap_object_nonnull().monitor_unlock(jvm);
        }
        InstructionInfo::multianewarray(cp) => multi_a_new_array(jvm, interpreter_state, cp),
        InstructionInfo::new(cp) => new(jvm, interpreter_state, cp as usize),
        InstructionInfo::newarray(a_type) => newarray(jvm, interpreter_state, a_type),
        InstructionInfo::nop => {}
        InstructionInfo::pop => pop(interpreter_state.current_frame_mut()),
        InstructionInfo::pop2 => pop2(interpreter_state.current_frame_mut()),
        InstructionInfo::putfield(cp) => putfield(jvm, interpreter_state, cp),
        InstructionInfo::putstatic(cp) => putstatic(jvm, interpreter_state, cp),
        InstructionInfo::ret(local_var_index) => ret(interpreter_state.current_frame_mut(), local_var_index as usize),
        InstructionInfo::return_ => return_(interpreter_state),
        InstructionInfo::saload => saload(interpreter_state.current_frame_mut()),
        InstructionInfo::sastore => sastore(jvm, interpreter_state),
        InstructionInfo::sipush(val) => sipush(interpreter_state.current_frame_mut(), val),
        InstructionInfo::swap => swap(interpreter_state.current_frame_mut()),
        InstructionInfo::tableswitch(switch) => tableswitch(switch, interpreter_state.current_frame_mut()),
        InstructionInfo::wide(w) => wide(interpreter_state.current_frame_mut(), w),
        InstructionInfo::EndOfCode => panic!(),
    }
}

fn l2d(current_frame: &mut StackEntry) {
    let val = current_frame.pop().unwrap_long();
    current_frame.push(JavaValue::Double(val as f64))
}

fn jsr(interpreter_state: &mut InterpreterStateGuard, target: i32) {
    let next_instruct = (interpreter_state.current_pc() as isize + interpreter_state.current_pc_offset()) as i64;
    interpreter_state.push_current_operand_stack(JavaValue::Long(next_instruct));
    *interpreter_state.current_pc_offset_mut() = target as isize
}

fn f2l(current_frame: &mut StackEntry) {
    let val = current_frame.pop().unwrap_float();
    let res = if val.is_infinite() {
        if val.is_sign_positive() {
            i64::MAX
        } else {
            i64::MIN
        }
    } else if val.is_nan() {
        0i64
    } else {
        val as i64
    };
    current_frame.push(JavaValue::Long(res))
}

fn dup2_x2(current_frame: &mut StackEntry) {
    let value1 = current_frame.pop();
    let value2 = current_frame.pop();
    if value1.is_size_2() {
        if value2.is_size_2() {
            //form 4
            current_frame.push(value1.clone());
            current_frame.push(value2);
            current_frame.push(value1);
        } else {
            //form 2
            let value3 = current_frame.pop();
            assert!(value3.is_size_1());
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    } else {
        assert!(value2.is_size_1());
        let value3 = current_frame.pop();
        if value3.is_size_2() {
            //form 3
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        } else {
            //form 1
            let value4 = current_frame.pop();
            assert!(value4.is_size_1());
            current_frame.push(value2.clone());
            current_frame.push(value1.clone());
            current_frame.push(value4);
            current_frame.push(value3);
            current_frame.push(value2);
            current_frame.push(value1);
        }
    }
}

fn frem(current_frame: &mut StackEntry) {
    let value2 = current_frame.pop().unwrap_float();
    let value1 = current_frame.pop().unwrap_float();
    let res = drem_impl(value2 as f64, value1 as f64) as f32;
    current_frame.push(JavaValue::Float(res));
}

fn fneg(current_frame: &mut StackEntry) {
    let val = current_frame.pop().unwrap_float();
    current_frame.push(JavaValue::Float(-val))
}

fn drem(current_frame: &mut StackEntry) {
    let value2 = current_frame.pop().unwrap_double();//divisor
    let value1 = current_frame.pop().unwrap_double();
    let res = drem_impl(value2, value1);
    current_frame.push(JavaValue::Double(res))
}

fn drem_impl(value2: f64, value1: f64) -> f64 {
    let res = if value1.is_nan() || value2.is_nan() {
        f64::NAN
    } else if value2.is_zero() || value1.is_infinite() {
        if value1.is_sign_negative() {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        }
    } else if value2.is_infinite() {
        value1
    } else if value1.is_zero() {
        value1
    } else {
        if (value1 / value2).is_sign_negative() {
            -value1.rem(value2).abs()
        } else {
            value1.rem(value2).abs()
        }
    };
    res
}

fn dneg(current_frame: &mut StackEntry) {
    let val = current_frame.pop().unwrap_double();
    current_frame.push(JavaValue::Double(-val))
}

fn swap(current_frame: &mut StackEntry) {
    let first = current_frame.pop();
    let second = current_frame.pop();
    current_frame.push(first);
    current_frame.push(second);
}

pub fn ret(current_frame: &mut StackEntry, local_var_index: usize) {
    let ret = current_frame.local_vars()[local_var_index].unwrap_long();
    *current_frame.pc_mut() = ret as usize;
    *current_frame.pc_offset_mut() = 0;
}

fn dcmpl(current_frame: &mut StackEntry) {
    let val2 = current_frame.pop().unwrap_double();
    let val1 = current_frame.pop().unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(JavaValue::Int(-1));
    }
    dcmp_common(current_frame, val2, val1);
}

fn dcmp_common(current_frame: &mut StackEntry, val2: f64, val1: f64) {
    let res = if val1 > val2 {
        1
    } else if val1 == val2 {
        0
    } else if val1 < val2 {
        -1
    } else {
        unreachable!()
    };
    current_frame.push(JavaValue::Int(res));
}

fn dcmpg(current_frame: &mut StackEntry) {
    let val2 = current_frame.pop().unwrap_double();
    let val1 = current_frame.pop().unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(JavaValue::Int(-1));
    }
    dcmp_common(current_frame, val2, val1)
}

fn athrow(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let exception_obj = {
        let value = interpreter_state.pop_current_operand_stack();
        // let value = interpreter_state.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap();
        value.unwrap_object_nonnull()
    };
    if jvm.debug_print_exceptions {
        println!("EXCEPTION:");
        dbg!(exception_obj.lookup_field("detailMessage"));
        interpreter_state.debug_print_stack_trace();
    }

    interpreter_state.set_throw(exception_obj.into());
}
