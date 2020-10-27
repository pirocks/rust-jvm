use std::sync::Arc;

use classfile_parser::code::{CodeParserContext, parse_instruction};
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::ACC_SYNCHRONIZED;
use rust_jvm_common::classfile::{Code, InstructionInfo};
use rust_jvm_common::classnames::ClassName;

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
use crate::interpreter_state::{InterpreterStateGuard, SuspendedStatus};
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::method_table::MethodId;
use crate::stack_entry::StackEntry;
use crate::threading::monitors::Monitor;

pub fn run_function(jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    let view = interpreter_state.current_class_view().clone();
    let method_i = interpreter_state.current_method_i();
    let method = view.method_view_i(method_i as usize);
    let synchronized = method.access_flags() & ACC_SYNCHRONIZED as u16 > 0;
    let code = method.code_attribute().unwrap();
    let meth_name = method.name();
    let class_name__ = view.name();

    let method_desc = method.desc_str();
    let current_depth = interpreter_state.call_stack_depth();
    let current_thread_tid = jvm.thread_state.try_get_current_thread().map(|t| t.java_tid).unwrap_or(-1);
    let function_enter_guard = jvm.tracing.trace_function_enter(&class_name__, &meth_name, &method_desc, current_depth, current_thread_tid);
    assert!(!*interpreter_state.function_return_mut());
    let class_pointer = interpreter_state.current_class_pointer().clone();
    let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer, method_i);
    //so figuring out which monitor to use is prob not this funcitions problem, like its already quite busy
    let monitor = monitor_for_function(jvm, interpreter_state, &method, synchronized, &class_name__);
    while !*interpreter_state.terminate() && !*interpreter_state.function_return() && interpreter_state.throw().is_none() {
        let (instruct, instruction_size) = current_instruction(interpreter_state.current_frame_mut(), &code, &meth_name);
        *interpreter_state.current_pc_offset_mut() = instruction_size as isize;
        breakpoint_check(jvm, interpreter_state, method_id);
        suspend_check(interpreter_state);
        if meth_name == "makePreparedLambdaForm" /*&& class_name__.get_referred_name().starts_with("java/lang/invoke/LambdaForm/Name")*/ {
            dbg!(interpreter_state.current_frame().local_vars());
            dbg!(interpreter_state.current_frame().operand_stack());
            dbg!(&instruct);
        }
        run_single_instruction(jvm, interpreter_state, instruct);
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
                        let catch_runtime_name = interpreter_state.current_class_view().constant_pool_view(excep_table.catch_type as usize).unwrap_class().class_name().unwrap_name();
                        let catch_class = check_inited_class(jvm, interpreter_state, &catch_runtime_name.into(), interpreter_state.current_loader(jvm).clone());
                        if inherits_from(jvm, interpreter_state, &throw_class, &catch_class) {
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
    // jvm.tracing.trace_function_exit(
    //     &class_name__,
    //     &meth_name,
    //     &method_desc,
    //     current_depth,
    //     current_thread_tid,
    // )
}

pub fn suspend_check(interpreter_state: &mut InterpreterStateGuard) {
    let SuspendedStatus { suspended, suspend_condvar } = &interpreter_state.thread.suspended;
    let suspended_guard = suspended.lock().unwrap();
    if *suspended_guard {
        drop(interpreter_state.int_state.take());
        drop(suspend_condvar.wait(suspended_guard).unwrap());
        interpreter_state.int_state = interpreter_state.thread.interpreter_state.write().unwrap().into();
    }
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

fn current_instruction(current_frame: &StackEntry, code: &Code, meth_name: &str) -> (InstructionInfo, usize) {
    let current = &code.code_raw[current_frame.pc()..];
    let mut context = CodeParserContext { offset: current_frame.pc(), iter: current.iter() };
    let parsedq = parse_instruction(&mut context);
    match &parsedq {
        None => {
            dbg!(&context.offset);
            dbg!(&meth_name);
            // dbg!(class_name_);
            dbg!(&code.code_raw);
            dbg!(&code.code);
            panic!();
        }
        Some(_) => {}
    };
    (parsedq.unwrap(), context.offset - current_frame.pc())
}

pub fn monitor_for_function(
    jvm: &JVMState,
    int_state: &mut InterpreterStateGuard,
    method: &MethodView,
    synchronized: bool,
    class_name: &ClassName,
) -> Option<Arc<Monitor>> {
    if synchronized {
        let monitor = if method.is_static() {
            let class_object = get_or_create_class_object(
                jvm,
                &class_name.clone().into(),
                int_state,
                int_state.current_loader(jvm).clone(),
            );
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
        InstructionInfo::aaload => aaload(interpreter_state.current_frame_mut()),
        InstructionInfo::aastore => aastore(interpreter_state.current_frame_mut()),
        InstructionInfo::aconst_null => aconst_null(interpreter_state.current_frame_mut()),
        InstructionInfo::aload(n) => aload(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::aload_0 => aload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::aload_1 => aload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::aload_2 => aload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::aload_3 => aload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        InstructionInfo::areturn => areturn(jvm, interpreter_state),
        InstructionInfo::arraylength => arraylength(interpreter_state),
        InstructionInfo::astore(n) => astore(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::astore_0 => astore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::astore_1 => astore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::astore_2 => astore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::astore_3 => astore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::athrow => athrow(jvm, interpreter_state),
        InstructionInfo::baload => baload(interpreter_state.current_frame_mut()),
        InstructionInfo::bastore => bastore(interpreter_state.current_frame_mut()),
        InstructionInfo::bipush(b) => bipush(interpreter_state.current_frame_mut(), b),
        InstructionInfo::caload => caload(jvm, interpreter_state),
        InstructionInfo::castore => castore(interpreter_state.current_frame_mut()),
        InstructionInfo::checkcast(cp) => invoke_checkcast(jvm, interpreter_state, cp),
        InstructionInfo::d2f => unimplemented!(),
        InstructionInfo::d2i => d2i(interpreter_state.current_frame_mut()),
        InstructionInfo::d2l => d2l(interpreter_state.current_frame_mut()),
        InstructionInfo::dadd => dadd(interpreter_state.current_frame_mut()),
        InstructionInfo::daload => unimplemented!(),
        InstructionInfo::dastore => unimplemented!(),
        InstructionInfo::dcmpg => unimplemented!(),
        InstructionInfo::dcmpl => unimplemented!(),
        InstructionInfo::dconst_0 => dconst_0(interpreter_state.current_frame_mut()),
        InstructionInfo::dconst_1 => dconst_1(interpreter_state.current_frame_mut()),
        InstructionInfo::ddiv => unimplemented!(),
        InstructionInfo::dload(i) => dload(interpreter_state.current_frame_mut(), i as usize),
        InstructionInfo::dload_0 => dload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::dload_1 => dload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::dload_2 => dload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::dload_3 => dload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::dmul => dmul(interpreter_state.current_frame_mut()),
        InstructionInfo::dneg => unimplemented!(),
        InstructionInfo::drem => unimplemented!(),
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
        InstructionInfo::dup2_x2 => unimplemented!(),
        InstructionInfo::f2d => f2d(interpreter_state.current_frame_mut()),
        InstructionInfo::f2i => f2i(interpreter_state.current_frame_mut()),
        InstructionInfo::f2l => unimplemented!(),
        InstructionInfo::fadd => fadd(interpreter_state.current_frame_mut()),
        InstructionInfo::faload => unimplemented!(),
        InstructionInfo::fastore => unimplemented!(),
        InstructionInfo::fcmpg => fcmpg(interpreter_state.current_frame_mut()),
        InstructionInfo::fcmpl => fcmpl(interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_0 => fconst_0(interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_1 => fconst_1(interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_2 => unimplemented!(),
        InstructionInfo::fdiv => fdiv(interpreter_state.current_frame_mut()),
        InstructionInfo::fload(n) => fload(interpreter_state.current_frame_mut(), n as usize),
        InstructionInfo::fload_0 => fload(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::fload_1 => fload(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::fload_2 => fload(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::fload_3 => fload(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::fmul => fmul(interpreter_state.current_frame_mut()),
        InstructionInfo::fneg => unimplemented!(),
        InstructionInfo::frem => unimplemented!(),
        InstructionInfo::freturn => freturn(jvm, interpreter_state),
        InstructionInfo::fstore(i) => fstore(interpreter_state.current_frame_mut(), i as usize),
        InstructionInfo::fstore_0 => fstore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::fstore_1 => fstore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::fstore_2 => fstore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::fstore_3 => fstore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::fsub => unimplemented!(),
        InstructionInfo::getfield(cp) => get_field(interpreter_state.current_frame_mut(), cp, false),
        InstructionInfo::getstatic(cp) => get_static(jvm, interpreter_state, cp),
        InstructionInfo::goto_(target) => goto_(interpreter_state.current_frame_mut(), target),
        InstructionInfo::goto_w(_) => unimplemented!(),
        InstructionInfo::i2b => i2b(interpreter_state.current_frame_mut()),
        InstructionInfo::i2c => i2c(interpreter_state.current_frame_mut()),
        InstructionInfo::i2d => i2d(interpreter_state.current_frame_mut()),
        InstructionInfo::i2f => i2f(interpreter_state.current_frame_mut()),
        InstructionInfo::i2l => i2l(interpreter_state.current_frame_mut()),
        InstructionInfo::i2s => i2s(interpreter_state.current_frame_mut()),
        InstructionInfo::iadd => iadd(interpreter_state.current_frame_mut()),
        InstructionInfo::iaload => iaload(interpreter_state.current_frame_mut()),
        InstructionInfo::iand => iand(interpreter_state.current_frame_mut()),
        InstructionInfo::iastore => iastore(interpreter_state.current_frame_mut()),
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
            // interpreter_state.get_current_frame().print_stack_trace();
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
        InstructionInfo::istore(n) => istore(interpreter_state.current_frame_mut(), n),
        InstructionInfo::istore_0 => istore(interpreter_state.current_frame_mut(), 0),
        InstructionInfo::istore_1 => istore(interpreter_state.current_frame_mut(), 1),
        InstructionInfo::istore_2 => istore(interpreter_state.current_frame_mut(), 2),
        InstructionInfo::istore_3 => istore(interpreter_state.current_frame_mut(), 3),
        InstructionInfo::isub => isub(interpreter_state.current_frame_mut()),
        InstructionInfo::iushr => iushr(interpreter_state.current_frame_mut()),
        InstructionInfo::ixor => ixor(interpreter_state.current_frame_mut()),
        InstructionInfo::jsr(_) => unimplemented!(),
        InstructionInfo::jsr_w(_) => unimplemented!(),
        InstructionInfo::l2d => unimplemented!(),
        InstructionInfo::l2f => l2f(interpreter_state.current_frame_mut()),
        InstructionInfo::l2i => l2i(interpreter_state.current_frame_mut()),
        InstructionInfo::ladd => ladd(interpreter_state.current_frame_mut()),
        InstructionInfo::laload => laload(interpreter_state.current_frame_mut()),
        InstructionInfo::land => land(interpreter_state.current_frame_mut()),
        InstructionInfo::lastore => lastore(interpreter_state.current_frame_mut()),
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
        InstructionInfo::ret(_) => unimplemented!(),
        InstructionInfo::return_ => return_(interpreter_state),
        InstructionInfo::saload => unimplemented!(),
        InstructionInfo::sastore => unimplemented!(),
        InstructionInfo::sipush(val) => sipush(interpreter_state.current_frame_mut(), val),
        InstructionInfo::swap => unimplemented!(),
        InstructionInfo::tableswitch(switch) => tableswitch(switch, interpreter_state.current_frame_mut()),
        InstructionInfo::wide(w) => wide(interpreter_state.current_frame_mut(), w),
        InstructionInfo::EndOfCode => unimplemented!(),
    }
}

fn athrow(_jvm: &JVMState, interpreter_state: &mut InterpreterStateGuard) {
    println!("EXCEPTION:");
    let exception_obj = {
        let value = interpreter_state.pop_current_operand_stack();
        // let value = interpreter_state.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap();
        value.unwrap_object_nonnull()
    };
    dbg!(exception_obj.lookup_field("detailMessage"));
    interpreter_state.print_stack_trace();
    interpreter_state.set_throw(exception_obj.into());
}
