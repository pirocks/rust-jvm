use std::ops::Deref;
use std::sync::Arc;

use classfile_parser::code::{CodeParserContext, parse_instruction};
use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::ACC_SYNCHRONIZED;
use rust_jvm_common::classfile::{Code, InstructionInfo};
use rust_jvm_common::classnames::ClassName;

use crate::JVMState;
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
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::stack_entry::StackEntry;
use crate::threading::JavaThread;
use crate::threading::monitors::Monitor;

pub fn run_function(jvm: &'static JVMState, current_thread: &JavaThread) {
    let current_frame = current_thread.get_current_frame_mut();
    let view = current_frame.class_pointer.view().clone();
    let method = view.method_view_i(current_frame.method_i as usize);
    let synchronized = method.access_flags() & ACC_SYNCHRONIZED as u16 > 0;
    let code = method.code_attribute().unwrap();
    let meth_name = method.name();
    let class_name__ = &current_frame.class_pointer.view().name();

    let method_desc = method.desc_str();
    let current_depth = current_thread.call_stack.read().unwrap().len();
    jvm.tracing.trace_function_enter(&class_name__, &meth_name, &method_desc, current_depth, current_thread.java_tid);
    let interpreter_state = &current_thread.interpreter_state;
    assert!(!*interpreter_state.function_return.read().unwrap());
    let method_id = jvm.method_table.write().unwrap().get_method_id(current_frame.class_pointer.clone(), current_frame.method_i);
    //so figuring out which monitor to use is prob not this funcitions problem, like its already quite busy
    let monitor = monitor_for_function(jvm, current_frame, &method, synchronized, &class_name__);
    while !*interpreter_state.terminate.read().unwrap() && !*interpreter_state.function_return.read().unwrap() && !interpreter_state.throw.read().unwrap().is_some() {
        let read_guard = interpreter_state.suspended.read().unwrap();
        let suspension_lock = read_guard.suspended_lock.clone();
        std::mem::drop(read_guard);
        std::mem::drop(suspension_lock.lock());//so this will block when threads are suspended
        let (instruct, instruction_size) = current_instruction(current_frame, &code, &meth_name);
        current_frame.pc_offset = instruction_size as isize;
        breakpoint_check(jvm, method_id, current_frame.pc as isize);
        run_single_instruction(jvm, &current_thread, instruct);
        if interpreter_state.throw.read().unwrap().is_some() {
            let throw_class = interpreter_state.throw.read().unwrap().as_ref().unwrap().unwrap_normal_object().class_pointer.clone();
            for excep_table in &code.exception_table {
                if excep_table.start_pc as usize <= current_frame.pc && current_frame.pc < (excep_table.end_pc as usize) {//todo exclusive
                    if excep_table.catch_type == 0 {
                        //todo dup
                        current_frame.push(JavaValue::Object(interpreter_state.throw.read().unwrap().deref().clone()));
                        *interpreter_state.throw.write().unwrap() = None;
                        current_frame.pc = excep_table.handler_pc as usize;
                        // println!("Caught Exception:{}", &throw_class.view().name().get_referred_name());
                        break;
                    } else {
                        let catch_runtime_name = current_frame.class_pointer.view().constant_pool_view(excep_table.catch_type as usize).unwrap_class().class_name().unwrap_name();
                        let catch_class = check_inited_class(jvm, &catch_runtime_name.into(), current_frame.class_pointer.loader(jvm).clone());
                        if inherits_from(jvm, &throw_class, &catch_class) {
                            current_frame.push(JavaValue::Object(interpreter_state.throw.read().unwrap().deref().clone()));
                            *interpreter_state.throw.write().unwrap() = None;
                            current_frame.pc = excep_table.handler_pc as usize;
                            // println!("Caught Exception:{}", throw_class.view().name().get_referred_name());
                            break;
                        }
                    }
                }
            }
            if interpreter_state.throw.read().unwrap().is_some() {
                //need to propogate to caller
                break;
            }
        } else {
            //todo need to figure out where return res ends up on next stack
            let offset = current_frame.pc_offset;
            let mut pc = current_frame.pc;
            if offset > 0 {
                pc += offset as usize;
            } else {
                pc -= (-offset) as usize;//todo perhaps i don't have to do this bs if I use u64 instead of usize
            }
            current_frame.pc = pc;
        }
    }
    if synchronized {//todo synchronize better so that natives are synced
        monitor.unwrap().unlock(jvm);
    }
    jvm.tracing.trace_function_exit(
        &class_name__,
        &meth_name,
        &method_desc,
        current_depth,
        current_thread.java_tid,
    )
}

fn breakpoint_check(jvm: &'static JVMState, methodid: usize, pc: isize) {
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
        unimplemented!()
    }
}

fn current_instruction(current_frame: &StackEntry, code: &Code, meth_name: &String) -> (InstructionInfo, usize) {
    let current = &code.code_raw[current_frame.pc..];
    let mut context = CodeParserContext { offset: current_frame.pc, iter: current.iter() };
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
    (parsedq.unwrap().clone(), context.offset - current_frame.pc)
}

pub fn monitor_for_function(
    jvm: &'static JVMState,
    current_frame: &mut StackEntry,
    method: &MethodView,
    synchronized: bool,
    class_name: &ClassName,
) -> Option<Arc<Monitor>> {
    if synchronized {
        let monitor = if method.is_static() {
            let class_object = get_or_create_class_object(
                jvm,
                &class_name.clone().into(),
                current_frame,
                current_frame.class_pointer.loader(jvm).clone(),
            );
            class_object.unwrap_normal_object().monitor.clone()
        } else {
            current_frame.local_vars[0].unwrap_normal_object().monitor.clone()
        };
        monitor.lock(jvm);
        monitor.into()
    } else {
        None
    }
}

fn run_single_instruction(
    jvm: &'static JVMState,
    current_thread: &JavaThread,
    instruct: InstructionInfo,
) {
    let current_frame = current_thread.get_current_frame_mut();
    let interpreter_state = &current_thread.interpreter_state;
    match instruct.clone() {
        InstructionInfo::aaload => aaload(current_frame),
        InstructionInfo::aastore => aastore(current_frame),
        InstructionInfo::aconst_null => aconst_null(current_frame),
        InstructionInfo::aload(n) => aload(current_frame, n as usize),
        InstructionInfo::aload_0 => aload(current_frame, 0),
        InstructionInfo::aload_1 => aload(current_frame, 1),
        InstructionInfo::aload_2 => aload(current_frame, 2),
        InstructionInfo::aload_3 => aload(current_frame, 3),
        InstructionInfo::anewarray(cp) => anewarray(jvm, current_frame, cp),
        InstructionInfo::areturn => areturn(jvm, current_thread, current_frame),
        InstructionInfo::arraylength => arraylength(current_frame),
        InstructionInfo::astore(n) => astore(current_frame, n as usize),
        InstructionInfo::astore_0 => astore(current_frame, 0),
        InstructionInfo::astore_1 => astore(current_frame, 1),
        InstructionInfo::astore_2 => astore(current_frame, 2),
        InstructionInfo::astore_3 => astore(current_frame, 3),
        InstructionInfo::athrow => {
            println!("EXCEPTION:");
            let exception_obj = current_frame.pop().unwrap_object_nonnull();
            dbg!(exception_obj.lookup_field("detailMessage"));
            jvm.thread_state.get_current_thread().print_stack_trace();
            *interpreter_state.throw.write().unwrap() = exception_obj.into();
        }
        InstructionInfo::baload => baload(current_frame),
        InstructionInfo::bastore => bastore(current_frame),
        InstructionInfo::bipush(b) => bipush(current_frame, b),
        InstructionInfo::caload => caload(jvm, current_frame),
        InstructionInfo::castore => castore(current_frame),
        InstructionInfo::checkcast(cp) => invoke_checkcast(jvm, current_frame, cp),
        InstructionInfo::d2f => unimplemented!(),
        InstructionInfo::d2i => d2i(current_frame),
        InstructionInfo::d2l => d2l(current_frame),
        InstructionInfo::dadd => dadd(current_frame),
        InstructionInfo::daload => unimplemented!(),
        InstructionInfo::dastore => unimplemented!(),
        InstructionInfo::dcmpg => unimplemented!(),
        InstructionInfo::dcmpl => unimplemented!(),
        InstructionInfo::dconst_0 => dconst_0(current_frame),
        InstructionInfo::dconst_1 => dconst_1(current_frame),
        InstructionInfo::ddiv => unimplemented!(),
        InstructionInfo::dload(i) => dload(current_frame, i as usize),
        InstructionInfo::dload_0 => dload(current_frame, 0),
        InstructionInfo::dload_1 => dload(current_frame, 1),
        InstructionInfo::dload_2 => dload(current_frame, 2),
        InstructionInfo::dload_3 => dload(current_frame, 3),
        InstructionInfo::dmul => dmul(current_frame),
        InstructionInfo::dneg => unimplemented!(),
        InstructionInfo::drem => unimplemented!(),
        InstructionInfo::dreturn => dreturn(jvm, current_thread, current_frame),
        InstructionInfo::dstore(i) => dstore(current_frame, i as usize),
        InstructionInfo::dstore_0 => dstore(current_frame, 0 as usize),
        InstructionInfo::dstore_1 => dstore(current_frame, 1 as usize),
        InstructionInfo::dstore_2 => dstore(current_frame, 2 as usize),
        InstructionInfo::dstore_3 => dstore(current_frame, 3 as usize),
        InstructionInfo::dsub => dsub(current_frame),
        InstructionInfo::dup => dup(current_frame),
        InstructionInfo::dup_x1 => dup_x1(current_frame),
        InstructionInfo::dup_x2 => dup_x2(current_frame),
        InstructionInfo::dup2 => dup2(current_frame),
        InstructionInfo::dup2_x1 => dup2_x1(current_frame),
        InstructionInfo::dup2_x2 => unimplemented!(),
        InstructionInfo::f2d => f2d(current_frame),
        InstructionInfo::f2i => f2i(current_frame),
        InstructionInfo::f2l => unimplemented!(),
        InstructionInfo::fadd => unimplemented!(),
        InstructionInfo::faload => unimplemented!(),
        InstructionInfo::fastore => unimplemented!(),
        InstructionInfo::fcmpg => fcmpg(current_frame),
        InstructionInfo::fcmpl => fcmpl(current_frame),
        InstructionInfo::fconst_0 => fconst_0(current_frame),
        InstructionInfo::fconst_1 => fconst_1(current_frame),
        InstructionInfo::fconst_2 => unimplemented!(),
        InstructionInfo::fdiv => fdiv(current_frame),
        InstructionInfo::fload(n) => fload(current_frame, n as usize),
        InstructionInfo::fload_0 => fload(current_frame, 0),
        InstructionInfo::fload_1 => fload(current_frame, 1),
        InstructionInfo::fload_2 => fload(current_frame, 2),
        InstructionInfo::fload_3 => fload(current_frame, 3),
        InstructionInfo::fmul => fmul(current_frame),
        InstructionInfo::fneg => unimplemented!(),
        InstructionInfo::frem => unimplemented!(),
        InstructionInfo::freturn => freturn(jvm, current_thread,current_frame),
        InstructionInfo::fstore(i) => fstore(current_frame, i as usize),
        InstructionInfo::fstore_0 => fstore(current_frame, 0),
        InstructionInfo::fstore_1 => fstore(current_frame, 1),
        InstructionInfo::fstore_2 => fstore(current_frame, 2),
        InstructionInfo::fstore_3 => fstore(current_frame, 3),
        InstructionInfo::fsub => unimplemented!(),
        InstructionInfo::getfield(cp) => get_field(current_frame, cp, false),
        InstructionInfo::getstatic(cp) => get_static(jvm, current_frame, cp),
        InstructionInfo::goto_(target) => goto_(current_frame, target),
        InstructionInfo::goto_w(_) => unimplemented!(),
        InstructionInfo::i2b => i2b(current_frame),
        InstructionInfo::i2c => i2c(current_frame),
        InstructionInfo::i2d => i2d(current_frame),
        InstructionInfo::i2f => i2f(current_frame),
        InstructionInfo::i2l => i2l(current_frame),
        InstructionInfo::i2s => i2s(current_frame),
        InstructionInfo::iadd => iadd(current_frame),
        InstructionInfo::iaload => iaload(current_frame),
        InstructionInfo::iand => iand(current_frame),
        InstructionInfo::iastore => iastore(current_frame),
        InstructionInfo::iconst_m1 => iconst_m1(current_frame),
        InstructionInfo::iconst_0 => iconst_0(current_frame),
        InstructionInfo::iconst_1 => iconst_1(current_frame),
        InstructionInfo::iconst_2 => iconst_2(current_frame),
        InstructionInfo::iconst_3 => iconst_3(current_frame),
        InstructionInfo::iconst_4 => iconst_4(current_frame),
        InstructionInfo::iconst_5 => iconst_5(current_frame),
        InstructionInfo::idiv => idiv(current_frame),
        InstructionInfo::if_acmpeq(offset) => if_acmpeq(current_frame, offset),
        InstructionInfo::if_acmpne(offset) => if_acmpne(current_frame, offset),
        InstructionInfo::if_icmpeq(offset) => if_icmpeq(current_frame, offset),
        InstructionInfo::if_icmpne(offset) => if_icmpne(current_frame, offset),
        InstructionInfo::if_icmplt(offset) => if_icmplt(current_frame, offset),
        InstructionInfo::if_icmpge(offset) => if_icmpge(current_frame, offset),
        InstructionInfo::if_icmpgt(offset) => if_icmpgt(current_frame, offset),
        InstructionInfo::if_icmple(offset) => if_icmple(current_frame, offset),
        InstructionInfo::ifeq(offset) => ifeq(current_frame, offset),
        InstructionInfo::ifne(offset) => ifne(current_frame, offset),
        InstructionInfo::iflt(offset) => iflt(current_frame, offset),
        InstructionInfo::ifge(offset) => ifge(current_frame, offset),
        InstructionInfo::ifgt(offset) => ifgt(current_frame, offset),
        InstructionInfo::ifle(offset) => ifle(current_frame, offset),
        InstructionInfo::ifnonnull(offset) => ifnonnull(current_frame, offset),
        InstructionInfo::ifnull(offset) => ifnull(current_frame, offset),
        InstructionInfo::iinc(iinc) => {
            let val = current_frame.local_vars[iinc.index as usize].unwrap_int();
            let res = val + iinc.const_ as i32;
            current_frame.local_vars[iinc.index as usize] = JavaValue::Int(res);
        }
        InstructionInfo::iload(n) => iload(current_frame, n as usize),
        InstructionInfo::iload_0 => iload(current_frame, 0),
        InstructionInfo::iload_1 => iload(current_frame, 1),
        InstructionInfo::iload_2 => iload(current_frame, 2),
        InstructionInfo::iload_3 => iload(current_frame, 3),
        InstructionInfo::imul => imul(current_frame),
        InstructionInfo::ineg => ineg(current_frame),
        InstructionInfo::instanceof(cp) => invoke_instanceof(jvm, current_frame, cp),
        InstructionInfo::invokedynamic(cp) => {
            // current_frame.print_stack_trace();
            invoke_dynamic(jvm, current_frame, cp)
        }
        InstructionInfo::invokeinterface(invoke_i) => invoke_interface(jvm, current_frame, invoke_i),
        InstructionInfo::invokespecial(cp) => invoke_special(jvm, current_frame, cp),
        InstructionInfo::invokestatic(cp) => run_invoke_static(jvm, current_frame, cp),
        InstructionInfo::invokevirtual(cp) => invoke_virtual_instruction(jvm, current_frame, cp, false),
        InstructionInfo::ior => ior(current_frame),
        InstructionInfo::irem => irem(current_frame),
        InstructionInfo::ireturn => ireturn(jvm, current_thread,current_frame),
        InstructionInfo::ishl => ishl(current_frame),
        InstructionInfo::ishr => ishr(current_frame),
        InstructionInfo::istore(n) => istore(current_frame, n),
        InstructionInfo::istore_0 => istore(current_frame, 0),
        InstructionInfo::istore_1 => istore(current_frame, 1),
        InstructionInfo::istore_2 => istore(current_frame, 2),
        InstructionInfo::istore_3 => istore(current_frame, 3),
        InstructionInfo::isub => isub(current_frame),
        InstructionInfo::iushr => iushr(current_frame),
        InstructionInfo::ixor => ixor(current_frame),
        InstructionInfo::jsr(_) => unimplemented!(),
        InstructionInfo::jsr_w(_) => unimplemented!(),
        InstructionInfo::l2d => unimplemented!(),
        InstructionInfo::l2f => l2f(current_frame),
        InstructionInfo::l2i => l2i(current_frame),
        InstructionInfo::ladd => ladd(current_frame),
        InstructionInfo::laload => laload(current_frame),
        InstructionInfo::land => land(current_frame),
        InstructionInfo::lastore => lastore(current_frame),
        InstructionInfo::lcmp => lcmp(current_frame),
        InstructionInfo::lconst_0 => lconst(current_frame, 0),
        InstructionInfo::lconst_1 => lconst(current_frame, 1),
        InstructionInfo::ldc(cp) => ldc_w(jvm, current_frame, cp as u16),
        InstructionInfo::ldc_w(cp) => ldc_w(jvm, current_frame, cp),
        InstructionInfo::ldc2_w(cp) => ldc2_w(current_frame, cp),
        InstructionInfo::ldiv => ldiv(current_frame),
        InstructionInfo::lload(i) => lload(current_frame, i as usize),
        InstructionInfo::lload_0 => lload(current_frame, 0),
        InstructionInfo::lload_1 => lload(current_frame, 1),
        InstructionInfo::lload_2 => lload(current_frame, 2),
        InstructionInfo::lload_3 => lload(current_frame, 3),
        InstructionInfo::lmul => lmul(current_frame),
        InstructionInfo::lneg => lneg(current_frame),
        InstructionInfo::lookupswitch(ls) => invoke_lookupswitch(&ls, current_frame),
        InstructionInfo::lor => lor(current_frame),
        InstructionInfo::lrem => lrem(current_frame),
        InstructionInfo::lreturn => lreturn(jvm, current_thread, current_frame),
        InstructionInfo::lshl => lshl(current_frame),
        InstructionInfo::lshr => lshr(current_frame),
        InstructionInfo::lstore(n) => lstore(current_frame, n as usize),
        InstructionInfo::lstore_0 => lstore(current_frame, 0),
        InstructionInfo::lstore_1 => lstore(current_frame, 1),
        InstructionInfo::lstore_2 => lstore(current_frame, 2),
        InstructionInfo::lstore_3 => lstore(current_frame, 3),
        InstructionInfo::lsub => lsub(current_frame),
        InstructionInfo::lushr => lushr(current_frame),
        InstructionInfo::lxor => lxor(current_frame),
        InstructionInfo::monitorenter => {
            current_frame.pop().unwrap_object_nonnull().monitor_lock(jvm);
        }
        InstructionInfo::monitorexit => {
            current_frame.pop().unwrap_object_nonnull().monitor_unlock(jvm);
        }
        InstructionInfo::multianewarray(cp) => multi_a_new_array(jvm, current_frame, cp),
        InstructionInfo::new(cp) => new(jvm, current_frame, cp as usize),
        InstructionInfo::newarray(a_type) => newarray(jvm, current_frame, a_type),
        InstructionInfo::nop => {}
        InstructionInfo::pop => pop(current_frame),
        InstructionInfo::pop2 => pop2(current_frame),
        InstructionInfo::putfield(cp) => putfield(jvm, current_frame, cp),
        InstructionInfo::putstatic(cp) => putstatic(jvm, current_frame, cp),
        InstructionInfo::ret(_) => unimplemented!(),
        InstructionInfo::return_ => return_(interpreter_state),
        InstructionInfo::saload => unimplemented!(),
        InstructionInfo::sastore => unimplemented!(),
        InstructionInfo::sipush(val) => sipush(current_frame, val),
        InstructionInfo::swap => unimplemented!(),
        InstructionInfo::tableswitch(switch) => tableswitch(switch, current_frame),
        InstructionInfo::wide(w) => wide(current_frame, w),
        InstructionInfo::EndOfCode => unimplemented!(),
    }
}
