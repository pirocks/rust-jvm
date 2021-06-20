use std::borrow::Borrow;
use std::ops::Rem;
use std::sync::Arc;

use num::Zero;

use classfile_parser::code::{CodeParserContext, parse_instruction};
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::PTypeView;
use classfile_view::vtype::VType;
use jvmti_jni_bindings::JVM_ACC_SYNCHRONIZED;
use rust_jvm_common::classfile::{Code, InstructionInfo};
use rust_jvm_common::classnames::ClassName;
use verification::OperandStack;
use verification::verifier::Frame;

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
use crate::stack_entry::{StackEntryMut, StackEntryRef};
use crate::threading::monitors::Monitor;

#[derive(Debug)]
pub struct WasException;

pub fn run_function(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<(), WasException> {
    let view = interpreter_state.current_class_view(jvm).clone();
    let method_i = interpreter_state.current_method_i(jvm);
    let method = view.method_view_i(method_i);
    let synchronized = method.access_flags() & JVM_ACC_SYNCHRONIZED as u16 > 0;
    let code = method.code_attribute().unwrap();
    let meth_name = method.name();
    let class_name__ = view.type_();

    let method_desc = method.desc_str();
    let current_depth = interpreter_state.call_stack_depth();
    let current_thread_tid = jvm.thread_state.try_get_current_thread().map(|t| t.java_tid).unwrap_or(-1);
    let function_enter_guard = jvm.tracing.trace_function_enter(&class_name__, &meth_name, &method_desc, current_depth, current_thread_tid);
    assert!(!interpreter_state.function_return());
    let current_frame = interpreter_state.current_frame();
    let class_pointer = current_frame.class_pointer(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
    //so figuring out which monitor to use is prob not this funcitions problem, like its already quite busy
    let monitor = monitor_for_function(jvm, interpreter_state, &method, synchronized);


    while !interpreter_state.function_return() && interpreter_state.throw().is_none() {
        let (instruct, instruction_size) = current_instruction(interpreter_state.current_frame(), &code);
        interpreter_state.set_current_pc_offset(instruction_size as i32);
        breakpoint_check(jvm, interpreter_state, method_id);
        if let Ok(()) = safepoint_check(jvm, interpreter_state) {
            run_single_instruction(jvm, interpreter_state, instruct, method_id);
        };
        if interpreter_state.throw().is_some() {
            let throw_class = interpreter_state.throw().as_ref().unwrap().unwrap_normal_object().objinfo.class_pointer.clone();
            for excep_table in &code.exception_table {
                let pc = interpreter_state.current_pc();
                if excep_table.start_pc <= pc && pc < (excep_table.end_pc) {//todo exclusive
                    if excep_table.catch_type == 0 {
                        //todo dup
                        interpreter_state.push_current_operand_stack(JavaValue::Object(todo!()/*interpreter_state.throw()*/));
                        interpreter_state.set_throw(None);
                        interpreter_state.set_current_pc(excep_table.handler_pc);
                        // println!("Caught Exception:{}", &throw_class.view().name().get_referred_name());
                        break;
                    } else {
                        let catch_runtime_name = interpreter_state.current_class_view(jvm).constant_pool_view(excep_table.catch_type as usize).unwrap_class().class_ref_type().unwrap_name();
                        let saved_throw = interpreter_state.throw().clone();
                        interpreter_state.set_throw(None);
                        let catch_class = check_resolved_class(jvm, interpreter_state, catch_runtime_name.into())?;
                        interpreter_state.set_throw(saved_throw);
                        if inherits_from(jvm, interpreter_state, &throw_class, &catch_class)? {
                            interpreter_state.push_current_operand_stack(JavaValue::Object(todo!()/*interpreter_state.throw()*/));
                            interpreter_state.set_throw(None);
                            interpreter_state.set_current_pc(excep_table.handler_pc);
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
    // let res = if interpreter_state.call_stack_depth() >= 2 {
    //     let frame = interpreter_state.previous_frame();
    //     frame.operand_stack().last().unwrap_or(&JavaValue::Top).clone()
    // } else {
    //     JavaValue::Top
    // };
    jvm.tracing.function_exit_guard(function_enter_guard, JavaValue::Top);//todo put actual res in here again
    if interpreter_state.throw().is_some() {
        return Err(WasException);
    }
    Ok(())
}

pub fn safepoint_check(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<(), WasException> {
    let thread = interpreter_state.thread.clone();
    let safe_point = thread.safepoint_state.borrow();
    safe_point.check(jvm, interpreter_state)
}

fn update_pc_for_next_instruction(interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let offset = interpreter_state.current_pc_offset();
    let mut pc = interpreter_state.current_pc();
    if offset > 0 {
        pc += offset as u16;
    } else {
        pc -= (-offset) as u16;
    }
    interpreter_state.set_current_pc(pc);
}

fn breakpoint_check(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, methodid: MethodId) {
    let pc = interpreter_state.current_pc();
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

fn current_instruction(current_frame: StackEntryRef, code: &Code) -> (InstructionInfo, usize) {
    let current = &code.code_raw[current_frame.pc() as usize..];
    let mut context = CodeParserContext { offset: current_frame.pc(), iter: current.iter() };
    let parsedq = parse_instruction(&mut context).expect("but this parsed the first time round");
    (parsedq, (context.offset as i32 - current_frame.pc() as i32) as usize)
}

pub fn monitor_for_function(
    jvm: &'_ JVMState<'gc_life>,
    int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
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
            int_state.current_frame_mut().local_vars(jvm).get(0, PTypeView::object()).unwrap_normal_object().monitor.clone()
        };
        monitor.lock(jvm);
        monitor.into()
    } else {
        None
    }
}

pub static mut TIMES: usize = 0;

fn run_single_instruction(
    jvm: &'_ JVMState<'gc_life>,
    interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
    instruct: InstructionInfo,
    method_id: MethodId,
) {
    unsafe {
        TIMES += 1;
        if TIMES % 100000 == 0 {
            interpreter_state.debug_print_stack_trace(jvm);
        }
    };
    // interpreter_state.verify_frame(jvm);
    // if interpreter_state.call_stack_depth() > 2 {
    //     let current_method_i = interpreter_state.current_frame().method_i();
    //     let current_view = interpreter_state.current_frame().class_pointer().view();
    //
    //     if jvm.vm_live() && current_view.method_view_i(current_method_i as usize).name().as_str() == "hasNext" && current_view.name().unwrap_name() == ClassName::new("java/util/ArrayList$Itr") {
    //         let previous_method_i = interpreter_state.previous_frame().method_i();
    //         let previous_view = interpreter_state.previous_frame().class_pointer().view();
    //         if jvm.vm_live() && previous_view.method_view_i(previous_method_i as usize).name().as_str() == "<init>" && previous_view.name().unwrap_name() == ClassName::new("bed")
    //         {
    //             // dbg!(&instruct);
    //             // dbg!(interpreter_state.current_frame().operand_stack());
    //         }
    //     }
    // }
    // dbg!(&instruct);
    // dbg!(interpreter_state.current_pc());
    match instruct {
        InstructionInfo::aaload => aaload(interpreter_state),
        InstructionInfo::aastore => aastore(jvm, interpreter_state),
        InstructionInfo::aconst_null => aconst_null(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::aload(n) => aload(jvm, interpreter_state.current_frame_mut(), n as u16),
        InstructionInfo::aload_0 => aload(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::aload_1 => aload(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::aload_2 => aload(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::aload_3 => aload(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        InstructionInfo::areturn => areturn(jvm, interpreter_state),
        InstructionInfo::arraylength => arraylength(jvm, interpreter_state),
        InstructionInfo::astore(n) => astore(jvm, interpreter_state.current_frame_mut(), n as u16),
        InstructionInfo::astore_0 => astore(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::astore_1 => astore(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::astore_2 => astore(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::astore_3 => astore(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::athrow => athrow(jvm, interpreter_state),
        InstructionInfo::baload => baload(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::bastore => bastore(jvm, interpreter_state),
        InstructionInfo::bipush(b) => bipush(jvm, interpreter_state.current_frame_mut(), b),
        InstructionInfo::caload => caload(jvm, interpreter_state),
        InstructionInfo::castore => castore(jvm, interpreter_state),
        InstructionInfo::checkcast(cp) => invoke_checkcast(jvm, interpreter_state, cp),
        InstructionInfo::d2f => d2f(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::d2i => d2i(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::d2l => d2l(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dadd => dadd(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::daload => daload(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dastore => dastore(jvm, interpreter_state),
        InstructionInfo::dcmpg => dcmpg(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dcmpl => dcmpl(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dconst_0 => dconst_0(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dconst_1 => dconst_1(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::ddiv => ddiv(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dload(i) => dload(jvm, interpreter_state.current_frame_mut(), i as u16),
        InstructionInfo::dload_0 => dload(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::dload_1 => dload(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::dload_2 => dload(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::dload_3 => dload(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::dmul => dmul(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dneg => dneg(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::drem => drem(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dreturn => dreturn(jvm, interpreter_state),
        InstructionInfo::dstore(i) => dstore(jvm, interpreter_state.current_frame_mut(), i as u16),
        InstructionInfo::dstore_0 => dstore(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::dstore_1 => dstore(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::dstore_2 => dstore(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::dstore_3 => dstore(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::dsub => dsub(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dup => dup(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dup_x1 => dup_x1(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::dup_x2 => dup_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        InstructionInfo::dup2 => dup2(jvm, method_id, interpreter_state.current_frame_mut()),
        InstructionInfo::dup2_x1 => dup2_x1(jvm, method_id, interpreter_state.current_frame_mut()),
        InstructionInfo::dup2_x2 => dup2_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        InstructionInfo::f2d => f2d(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::f2i => f2i(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::f2l => f2l(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fadd => fadd(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::faload => faload(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fastore => fastore(jvm, interpreter_state),
        InstructionInfo::fcmpg => fcmpg(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fcmpl => fcmpl(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_0 => fconst_0(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_1 => fconst_1(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fconst_2 => fconst_2(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fdiv => fdiv(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fload(n) => fload(jvm, interpreter_state.current_frame_mut(), n as u16),
        InstructionInfo::fload_0 => fload(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::fload_1 => fload(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::fload_2 => fload(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::fload_3 => fload(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::fmul => fmul(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::fneg => fneg(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::frem => frem(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::freturn => freturn(jvm, interpreter_state),
        InstructionInfo::fstore(i) => fstore(jvm, interpreter_state.current_frame_mut(), i as u16),
        InstructionInfo::fstore_0 => fstore(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::fstore_1 => fstore(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::fstore_2 => fstore(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::fstore_3 => fstore(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::fsub => fsub(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::getfield(cp) => get_field(jvm, interpreter_state, cp, false),
        InstructionInfo::getstatic(cp) => get_static(jvm, interpreter_state, cp),
        InstructionInfo::goto_(target) => goto_(jvm, interpreter_state.current_frame_mut(), target as i32),
        InstructionInfo::goto_w(target) => goto_(jvm, interpreter_state.current_frame_mut(), target),
        InstructionInfo::i2b => i2b(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::i2c => i2c(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::i2d => i2d(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::i2f => i2f(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::i2l => i2l(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::i2s => i2s(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iadd => iadd(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iaload => iaload(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iand => iand(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iastore => iastore(jvm, interpreter_state),
        InstructionInfo::iconst_m1 => iconst_m1(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_0 => iconst_0(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_1 => iconst_1(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_2 => iconst_2(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_3 => iconst_3(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_4 => iconst_4(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iconst_5 => iconst_5(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::idiv => idiv(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::if_acmpeq(offset) => if_acmpeq(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_acmpne(offset) => if_acmpne(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpeq(offset) => if_icmpeq(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpne(offset) => if_icmpne(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmplt(offset) => if_icmplt(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpge(offset) => if_icmpge(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmpgt(offset) => if_icmpgt(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::if_icmple(offset) => if_icmple(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifeq(offset) => ifeq(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifne(offset) => ifne(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::iflt(offset) => iflt(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifge(offset) => ifge(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifgt(offset) => ifgt(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifle(offset) => ifle(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifnonnull(offset) => ifnonnull(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::ifnull(offset) => ifnull(jvm, interpreter_state.current_frame_mut(), offset),
        InstructionInfo::iinc(iinc) => {
            let mut current_frame = interpreter_state.current_frame_mut();
            let val = current_frame.local_vars(jvm).get(iinc.index, PTypeView::IntType).unwrap_int();
            let res = val + iinc.const_ as i32;
            current_frame.local_vars_mut(jvm).set(iinc.index, JavaValue::Int(res));
        }
        InstructionInfo::iload(n) => iload(jvm, interpreter_state.current_frame_mut(), n as u16),
        InstructionInfo::iload_0 => iload(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::iload_1 => iload(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::iload_2 => iload(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::iload_3 => iload(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::imul => imul(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::ineg => ineg(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::instanceof(cp) => invoke_instanceof(jvm, interpreter_state, cp),
        InstructionInfo::invokedynamic(cp) => {
            invoke_dynamic(jvm, interpreter_state, cp)
        }
        InstructionInfo::invokeinterface(invoke_i) => invoke_interface(jvm, interpreter_state, invoke_i),
        InstructionInfo::invokespecial(cp) => invoke_special(jvm, interpreter_state, cp),
        InstructionInfo::invokestatic(cp) => run_invoke_static(jvm, interpreter_state, cp),
        InstructionInfo::invokevirtual(cp) => invoke_virtual_instruction(jvm, interpreter_state, cp),
        InstructionInfo::ior => ior(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::irem => irem(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::ireturn => ireturn(jvm, interpreter_state),
        InstructionInfo::ishl => ishl(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::ishr => ishr(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::istore(n) => istore(jvm, interpreter_state.current_frame_mut(), n as u16),
        InstructionInfo::istore_0 => istore(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::istore_1 => istore(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::istore_2 => istore(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::istore_3 => istore(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::isub => isub(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::iushr => iushr(interpreter_state),
        InstructionInfo::ixor => ixor(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::jsr(target) => jsr(interpreter_state, target as i32),
        InstructionInfo::jsr_w(target) => jsr(interpreter_state, target),
        InstructionInfo::l2d => l2d(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::l2f => l2f(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::l2i => l2i(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::ladd => ladd(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::laload => laload(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::land => land(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lastore => lastore(jvm, interpreter_state),
        InstructionInfo::lcmp => lcmp(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lconst_0 => lconst(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::lconst_1 => lconst(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::ldc(cp) => ldc_w(jvm, interpreter_state, cp as u16),
        InstructionInfo::ldc_w(cp) => ldc_w(jvm, interpreter_state, cp),
        InstructionInfo::ldc2_w(cp) => ldc2_w(jvm, interpreter_state.current_frame_mut(), cp),
        InstructionInfo::ldiv => ldiv(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lload(i) => lload(jvm, interpreter_state.current_frame_mut(), i as u16),
        InstructionInfo::lload_0 => lload(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::lload_1 => lload(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::lload_2 => lload(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::lload_3 => lload(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::lmul => lmul(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lneg => lneg(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lor => lor(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lrem => lrem(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lreturn => lreturn(jvm, interpreter_state),
        InstructionInfo::lshl => lshl(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lshr => lshr(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lstore(n) => lstore(jvm, interpreter_state.current_frame_mut(), n as u16),
        InstructionInfo::lstore_0 => lstore(jvm, interpreter_state.current_frame_mut(), 0),
        InstructionInfo::lstore_1 => lstore(jvm, interpreter_state.current_frame_mut(), 1),
        InstructionInfo::lstore_2 => lstore(jvm, interpreter_state.current_frame_mut(), 2),
        InstructionInfo::lstore_3 => lstore(jvm, interpreter_state.current_frame_mut(), 3),
        InstructionInfo::lsub => lsub(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lushr => lushr(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::lxor => lxor(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::monitorenter => {
            interpreter_state.current_frame_mut().pop(jvm, PTypeView::object()).unwrap_object_nonnull().monitor_lock(jvm);
        }
        InstructionInfo::monitorexit => {
            interpreter_state.current_frame_mut().pop(jvm, PTypeView::object()).unwrap_object_nonnull().monitor_unlock(jvm);
        }
        InstructionInfo::multianewarray(cp) => multi_a_new_array(jvm, interpreter_state, cp),
        InstructionInfo::new(cp) => new(jvm, interpreter_state, cp),
        InstructionInfo::newarray(a_type) => newarray(jvm, interpreter_state, a_type),
        InstructionInfo::nop => {}
        InstructionInfo::pop => pop(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::pop2 => pop2(jvm, method_id, interpreter_state.current_frame_mut()),
        InstructionInfo::putfield(cp) => putfield(jvm, interpreter_state, cp),
        InstructionInfo::putstatic(cp) => putstatic(jvm, interpreter_state, cp),
        InstructionInfo::ret(local_var_index) => ret(jvm, interpreter_state.current_frame_mut(), local_var_index as u16),
        InstructionInfo::return_ => return_(interpreter_state),
        InstructionInfo::saload => saload(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::sastore => sastore(jvm, interpreter_state),
        InstructionInfo::sipush(val) => sipush(jvm, interpreter_state.current_frame_mut(), val),
        InstructionInfo::swap => swap(jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::tableswitch(switch) => tableswitch(switch, jvm, interpreter_state.current_frame_mut()),
        InstructionInfo::wide(w) => wide(jvm, interpreter_state.current_frame_mut(), w),
        InstructionInfo::EndOfCode => panic!(),
    }
}

fn l2d(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let val = current_frame.pop(jvm, PTypeView::LongType).unwrap_long();
    current_frame.push(jvm, JavaValue::Double(val as f64))
}

fn jsr(interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, target: i32) {
    let next_instruct = (interpreter_state.current_pc() as i32 + interpreter_state.current_pc_offset()) as i64;
    interpreter_state.push_current_operand_stack(JavaValue::Long(next_instruct));
    interpreter_state.set_current_pc_offset(target);
}

fn f2l(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let val = current_frame.pop(jvm, PTypeView::FloatType,
    ).unwrap_float();
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
    current_frame.push(jvm, JavaValue::Long(res))
}

fn dup2_x2(jvm: &'_ JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life>) {
    let current_pc = current_frame.to_ref().pc();
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = &stack_frames[&current_pc];
    let value1_vtype = data[0].clone();
    let value2_vtype = data[1].clone();
    let value1 = current_frame.pop(jvm, PTypeView::LongType);
    let value2 = current_frame.pop(jvm, PTypeView::LongType);
    match value1_vtype {
        VType::LongType | VType::DoubleType => {
            match value2_vtype {
                VType::LongType | VType::DoubleType => {
                    //form 4
                    current_frame.push(jvm, value1.clone());
                    current_frame.push(jvm, value2);
                    current_frame.push(jvm, value1);
                }
                _ => {
                    //form 2
                    let value3 = current_frame.pop(jvm, PTypeView::LongType);
                    // assert!(value3.is_size_1());
                    current_frame.push(jvm, value1.clone());
                    current_frame.push(jvm, value3);
                    current_frame.push(jvm, value2);
                    current_frame.push(jvm, value1);
                }
            }
        }
        _ => {
            // assert!(value2.is_size_1());
            let value2_vtype = data[2].clone();
            let value3 = current_frame.pop(jvm, PTypeView::LongType);
            match value2_vtype {
                VType::LongType | VType::DoubleType => {
                    //form 3
                    current_frame.push(jvm, value2.clone());
                    current_frame.push(jvm, value1.clone());
                    current_frame.push(jvm, value3);
                    current_frame.push(jvm, value2);
                    current_frame.push(jvm, value1);
                }
                _ => {
                    //form 1
                    let value4 = current_frame.pop(jvm, PTypeView::LongType);
                    // assert!(value4.is_size_1());
                    current_frame.push(jvm, value2.clone());
                    current_frame.push(jvm, value1.clone());
                    current_frame.push(jvm, value4);
                    current_frame.push(jvm, value3);
                    current_frame.push(jvm, value2);
                    current_frame.push(jvm, value1);
                }
            }
        }
    }
}

fn frem(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    let res = drem_impl(value2 as f64, value1 as f64) as f32;
    current_frame.push(jvm, JavaValue::Float(res));
}

fn fneg(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let val = current_frame.pop(jvm, PTypeView::FloatType).unwrap_float();
    current_frame.push(jvm, JavaValue::Float(-val))
}

fn drem(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let value2 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_double();//divisor
    let value1 = current_frame.pop(jvm, PTypeView::FloatType).unwrap_double();
    let res = drem_impl(value2, value1);
    current_frame.push(jvm, JavaValue::Double(res))
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

fn dneg(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let val = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    current_frame.push(jvm, JavaValue::Double(-val))
}

fn swap(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let first = current_frame.pop(jvm, PTypeView::LongType);
    let second = current_frame.pop(jvm, PTypeView::LongType);
    current_frame.push(jvm, first);
    current_frame.push(jvm, second);
}

pub fn ret(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, local_var_index: u16) {
    let ret = current_frame.local_vars(jvm).get(local_var_index, PTypeView::LongType).unwrap_long();
    current_frame.set_pc(ret as u16);
    *current_frame.pc_offset_mut() = 0;
}

fn dcmpl(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let val2 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    let val1 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(jvm, JavaValue::Int(-1));
    }
    dcmp_common(jvm, current_frame, val2, val1);
}

fn dcmp_common(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>, val2: f64, val1: f64) {
    let res = if val1 > val2 {
        1
    } else if val1 == val2 {
        0
    } else if val1 < val2 {
        -1
    } else {
        unreachable!()
    };
    current_frame.push(jvm, JavaValue::Int(res));
}

fn dcmpg(jvm: &'_ JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life>) {
    let val2 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    let val1 = current_frame.pop(jvm, PTypeView::DoubleType).unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(jvm, JavaValue::Int(-1));
    }
    dcmp_common(jvm, current_frame, val2, val1)
}

fn athrow(jvm: &'_ JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) {
    let exception_obj = {
        let value = interpreter_state.pop_current_operand_stack(ClassName::throwable().into());
        // let value = interpreter_state.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap();
        value.unwrap_object_nonnull()
    };
    if jvm.debug_print_exceptions {
        println!("EXCEPTION:");
        dbg!(exception_obj.lookup_field("detailMessage"));
        interpreter_state.debug_print_stack_trace(jvm);
    }

    interpreter_state.set_throw(exception_obj.into());
}
