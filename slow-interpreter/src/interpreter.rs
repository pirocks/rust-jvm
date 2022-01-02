use std::borrow::{Borrow, BorrowMut};
use std::ffi::c_void;
use std::ops::{Deref, Rem};
use std::sync::Arc;

use itertools::{Either, Itertools};
use num::Zero;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jvalue, JVM_ACC_SYNCHRONIZED};
use rust_jvm_common::compressed_classfile::code::CInstructionInfo;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::RuntimeType;
use rust_jvm_common::vtype::VType;
use verification::OperandStack;
use verification::verifier::Frame;

use crate::class_loading::{check_loaded_class, check_resolved_class};
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
use crate::interpreter_state::{FramePushGuard, InterpreterStateGuard};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OwnedJavaStack};
use crate::java_values::{GcManagedObject, JavaValue, NativeJavaValue};
use crate::jit::MethodResolver;
use crate::jit::state::JITedCodeState;
use crate::jvm_state::JVMState;
use crate::stack_entry::StackEntryMut;
use crate::threading::safepoints::Monitor2;

#[derive(Debug)]
pub struct WasException;

static mut INSTRUCTION_COUNT: u64 = 0;

static mut ITERATION_COUNT: u64 = 0;

pub struct FrameToRunOn {
    pub frame_pointer: JavaStackPosition,
    pub size: usize,
}

//takes exclusive framepush guard so I know I can mut the frame rip safelyish maybe. todo have a better way of doing this
pub fn run_function(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, frame_guard: &mut FramePushGuard) -> Result<Option<JavaValue<'gc_life>>, WasException> {
    if jvm.config.compiled_mode_active {
        let rc = interpreter_state.current_frame().class_pointer(jvm);
        let method_i = interpreter_state.current_method_i(jvm);
        let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_i);
        let view = interpreter_state.current_class_view(jvm).clone();
        let method = view.method_view_i(method_i);
        let code = method.code_attribute().unwrap();
        let resolver = MethodResolver { jvm, loader: LoaderName::BootstrapLoader };
        jvm.java_vm_state.add_method(jvm, &resolver, method_id);
        let frame_size = jvm.java_function_frame_data.read().unwrap().get(&method_id).unwrap().full_frame_size();
        let frame_to_run_on = FrameToRunOn {
            frame_pointer: interpreter_state.current_frame().frame_view.position(),
            size: frame_size,
        };
        let top_level_return_function_id = jvm.java_vm_state.ir.get_top_level_return_ir_method_id();
        interpreter_state.current_frame_mut().frame_view.set_prev_rip(top_level_return_function_id,jvm);
        let function_res = jvm.java_vm_state.run_method(jvm, interpreter_state, method_id, frame_to_run_on);
        //todo bug what if gc happens here
        let return_type = &method.desc().return_type;
        Ok(match return_type {
            CompressedParsedDescriptorType::VoidType => None,
            return_type => {
                Some(NativeJavaValue{as_u64: function_res}.to_java_value(&return_type,jvm))
            }
        })

        /*        let result = jvm.jit_state.with::<_, Result<(), WasException>>(|jit_state| {
                    jit_state.borrow_mut().add_function(code, method_id, resolver); //todo fix method id jankyness
                    // todo!("copy current args over. ");
                    match JITedCodeState::run_method_safe(jit_state, jvm, interpreter_state, method_id) {
                        Ok(res) => {
                            assert!(res.is_none());
                            return Ok(());
                            /*match res {
                                Either::Left(res) => todo!(),
                                Either::Right(VMExitData::InvokeStaticResolveTarget { method_name, descriptor, classname_ref_type, native_start, native_end }) => {
                                    let rc = check_loaded_class(jvm, interpreter_state, CPDType::Ref(classname_ref_type))?;
                                    let view = rc.view();
                                    let method_view = view.lookup_method(method_name, &descriptor).unwrap();
                                    let code = method_view.code_attribute().unwrap();
                                    let invoke_target_method_id = jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_view.method_i());
                                    let guard = jvm.function_frame_type_data.read().unwrap();
                                    let frame_vtype = guard.get(&invoke_target_method_id).unwrap();
                                    let stack_frame_layout = FrameBackedStackframeMemoryLayout::new(code.max_stack as usize, code.max_locals as usize, frame_vtype.clone());//todo use stack frame layouts instead
                                    let sorted_instructions = code.instructions.iter().sorted_by_key(|(offset, _)| *offset).map(|(_, instr)| instr.clone()).collect();
                                    let mut compiled_methods_guard = jvm.compiled_methods.write().unwrap();
                                    compiled_methods_guard.add_method(invoke_target_method_id, sorted_instructions, &stack_frame_layout);
                                    compiled_methods_guard.run_method(invoke_target_method_id, interpreter_state.get_java_stack()).unwrap();
                                    drop(compiled_methods_guard);//tos=do deadlock in exit hadnler


                                    todo!("compile and restore ")
                                }
                                _ => todo!()
                            }*/
                        }
                        Err(_) => todo!(),
                    }
                });
        */        //result
    } else {
        todo!()
        // run_function_interpreted(&jvm, interpreter_state)
    }
}

fn run_function_interpreted(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), WasException> {
    let view = interpreter_state.current_class_view(jvm).clone();
    let method_i = interpreter_state.current_method_i(jvm);
    let method = view.method_view_i(method_i);
    let synchronized = method.access_flags() & JVM_ACC_SYNCHRONIZED as u16 > 0;
    let code = method.code_attribute().unwrap();
    let meth_name = method.name();
    let class_name__ = view.type_();

    let method_desc = method.desc_str().to_str(&jvm.string_pool);
    let current_depth = interpreter_state.call_stack_depth();
    let current_thread_tid = jvm.thread_state.try_get_current_thread().map(|t| t.java_tid).unwrap_or(-1);
    let function_enter_guard = jvm.config.tracing.trace_function_enter(&jvm.string_pool, &class_name__, &meth_name, &method_desc, current_depth, current_thread_tid);
    assert!(!interpreter_state.function_return());
    let current_frame = interpreter_state.current_frame();
    let class_pointer = current_frame.class_pointer(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(class_pointer.clone(), method_i);
    //so figuring out which monitor to use is prob not this funcitions problem, like its already quite busy
    let monitor = monitor_for_function(jvm, interpreter_state, &method, synchronized);

    while !interpreter_state.function_return() && interpreter_state.throw().is_none() {
        let current_frame = interpreter_state.current_frame();
        let current = code.instructions.get(todo!() /*&(current_frame.pc(jvm) as u16)*/).unwrap();
        let (instruct, instruction_size) = (current, current.instruction_size as usize);
        interpreter_state.set_current_pc_offset(instruction_size as i32);
        breakpoint_check(jvm, interpreter_state, method_id);
        unsafe {
            INSTRUCTION_COUNT += 1;
            if INSTRUCTION_COUNT % 1 == 0 {
                safepoint_check(jvm, interpreter_state).unwrap();
            }
        }
        if meth_name == MethodName(jvm.string_pool.add_name("developLongDigits", false)) || meth_name == MethodName(jvm.string_pool.add_name("getBinaryToASCIIConverter", false)) || meth_name == MethodName(jvm.string_pool.add_name("dtoa", false)) {
            let mut frame = interpreter_state.current_frame_mut();
            let local_vars_ref = frame.local_vars();
            let num_local_vars = local_vars_ref.len();
            for i in 0..num_local_vars {
                dbg!(i);
                dbg!(local_vars_ref.get(i as u16, RuntimeType::LongType).try_unwrap_long());
            }
            let operand_stack = frame.operand_stack_ref(jvm);
            dbg!(operand_stack.types());
            for elem in operand_stack.types_vals() {
                match elem {
                    JavaValue::Long(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Int(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Short(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Byte(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Boolean(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Char(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Float(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Double(_) => {
                        dbg!(elem);
                    }
                    JavaValue::Object(_) => {}
                    JavaValue::Top => {}
                };
            }

            dbg!(instruct);
        }
        run_single_instruction(jvm, interpreter_state, &instruct.info, method_id);
        if interpreter_state.throw().is_some() {
            let throw_class = interpreter_state.throw().as_ref().unwrap().unwrap_normal_object().objinfo.class_pointer.clone();
            for excep_table in &code.exception_table {
                let pc = interpreter_state.current_pc();
                if excep_table.start_pc <= pc && pc < (excep_table.end_pc) {
                    //todo exclusive
                    match excep_table.catch_type {
                        None => {
                            //todo dup
                            interpreter_state.debug_print_stack_trace(jvm);
                            interpreter_state.push_current_operand_stack(JavaValue::Object(interpreter_state.throw()));
                            interpreter_state.set_throw(None);
                            interpreter_state.set_current_pc(excep_table.handler_pc);
                            // println!("Caught Exception:{}", &throw_class.view().name().get_referred_name());
                            break;
                        }
                        Some(catch_runtime_name) => {
                            let saved_throw = interpreter_state.throw().clone();
                            interpreter_state.set_throw(None);
                            let catch_class = check_resolved_class(jvm, interpreter_state, catch_runtime_name.into())?;
                            interpreter_state.set_throw(saved_throw);
                            if inherits_from(jvm, interpreter_state, &throw_class, &catch_class)? {
                                interpreter_state.push_current_operand_stack(JavaValue::Object(interpreter_state.throw()));
                                interpreter_state.set_throw(None);
                                interpreter_state.set_current_pc(excep_table.handler_pc);
                                // println!("Caught Exception:{}", throw_class.view().name().get_referred_name());
                                break;
                            }
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
    if synchronized {
        //todo synchronize better so that natives are synced
        monitor.unwrap().unlock(jvm, interpreter_state).unwrap();
    }
    // let res = if interpreter_state.call_stack_depth() >= 2 {
    //     let frame = interpreter_state.previous_frame();
    //     frame.operand_stack().last().unwrap_or(&JavaValue::Top).clone()
    // } else {
    //     JavaValue::Top
    // };
    jvm.config.tracing.function_exit_guard(function_enter_guard, JavaValue::Top); //todo put actual res in here again
    if interpreter_state.throw().is_some() {
        return Err(WasException);
    }
    Ok(())
}

pub fn safepoint_check(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), WasException> {
    let thread = interpreter_state.thread.clone();
    let safe_point = thread.safepoint_state.borrow();
    safe_point.check(jvm, interpreter_state)
}

fn update_pc_for_next_instruction(interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let offset = interpreter_state.current_pc_offset();
    let mut pc = interpreter_state.current_pc();
    if offset > 0 {
        pc += offset as u16;
    } else {
        pc -= (-offset) as u16;
    }
    interpreter_state.set_current_pc(pc);
}

fn breakpoint_check(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, methodid: MethodId) {
    let pc = interpreter_state.current_pc();
    let stop = match jvm.jvmti_state() {
        None => false,
        Some(jvmti) => {
            let breakpoints = &jvmti.break_points.read().unwrap();
            let function_breakpoints = breakpoints.get(&methodid);
            function_breakpoints.map(|points| points.contains(&pc)).unwrap_or(false)
        }
    };
    if stop {
        let jdwp = &jvm.jvmti_state().unwrap().built_in_jdwp;
        jdwp.breakpoint(jvm, methodid, pc as i64, interpreter_state);
    }
}

pub fn monitor_for_function(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, method: &MethodView, synchronized: bool) -> Option<Arc<Monitor2>> {
    if synchronized {
        let monitor: Arc<Monitor2> = if method.is_static() {
            let class_object = get_or_create_class_object(jvm, method.classview().type_(), int_state).unwrap();
            todo!() /*class_object.unwrap_normal_object().monitor.clone()*/
        } else {
            /*int_state.current_frame_mut().local_vars().get(0, RuntimeType::object()).unwrap_normal_object().monitor.clone()*/
            todo!()
        };
        monitor.lock(jvm, int_state).unwrap();
        monitor.into()
    } else {
        None
    }
}

pub static mut TIMES: usize = 0;

fn run_single_instruction(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, instruct: &CInstructionInfo, method_id: MethodId) {
    unsafe {
        TIMES += 1;
        if TIMES % 10_000_000 == 0 && jvm.vm_live() && jvm.thread_state.get_main_thread().is_this_thread() {
            interpreter_state.debug_print_stack_trace(jvm);
            //todo this thread suspension stuff is mega sketch
            // drop(interpreter_state.int_state.take());
            // jvm.gc.gc_jvm(jvm);
            // let current_thread = jvm.thread_state.get_current_thread();
            // interpreter_state.int_state = Some(transmute(current_thread.interpreter_state.write().unwrap()));
            // for thread in jvm.thread_state.get_all_threads().values() {
            //     let _ = thread.gc_resume_thread();
            // }
            // dbg!(interpreter_state.current_frame().local_vars(jvm).len());
            // dbg!(interpreter_state.current_frame().operand_stack(jvm).len());
            // dbg!(interpreter_state.current_frame().operand_stack(jvm).types());
            // dbg!(interpreter_state.current_frame().operand_stack(jvm).types_vals().into_iter().map(|jv|jv.to_type()).collect_vec());
            // dbg!(&instruct);
        }
    };
    match instruct {
        CInstructionInfo::aaload => aaload(jvm, interpreter_state),
        CInstructionInfo::aastore => aastore(jvm, interpreter_state),
        CInstructionInfo::aconst_null => aconst_null(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::aload(n) => aload(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::aload_0 => aload(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::aload_1 => aload(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::aload_2 => aload(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::aload_3 => aload(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::anewarray(cp) => anewarray(jvm, interpreter_state, cp),
        CInstructionInfo::areturn => areturn(jvm, interpreter_state),
        CInstructionInfo::arraylength => arraylength(jvm, interpreter_state),
        CInstructionInfo::astore(n) => astore(interpreter_state.current_frame_mut(), *n as u16),
        CInstructionInfo::astore_0 => astore(interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::astore_1 => astore(interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::astore_2 => astore(interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::astore_3 => astore(interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::athrow => athrow(jvm, interpreter_state),
        CInstructionInfo::baload => baload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::bastore => bastore(jvm, interpreter_state),
        CInstructionInfo::bipush(b) => bipush(jvm, interpreter_state.current_frame_mut(), *b),
        CInstructionInfo::caload => caload(jvm, interpreter_state),
        CInstructionInfo::castore => castore(jvm, interpreter_state),
        CInstructionInfo::checkcast(cp) => invoke_checkcast(jvm, interpreter_state, cp),
        CInstructionInfo::d2f => d2f(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::d2i => d2i(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::d2l => d2l(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dadd => dadd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::daload => daload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dastore => dastore(jvm, interpreter_state),
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
        CInstructionInfo::dreturn => dreturn(jvm, interpreter_state),
        CInstructionInfo::dstore(i) => dstore(jvm, interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::dstore_0 => dstore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::dstore_1 => dstore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::dstore_2 => dstore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::dstore_3 => dstore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::dsub => dsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup => dup(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup_x1 => dup_x1(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup_x2 => dup_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup2 => dup2(jvm, method_id, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup2_x1 => dup2_x1(jvm, method_id, interpreter_state.current_frame_mut()),
        CInstructionInfo::dup2_x2 => dup2_x2(jvm, method_id, interpreter_state.current_frame_mut()),
        CInstructionInfo::f2d => f2d(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::f2i => f2i(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::f2l => f2l(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fadd => fadd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::faload => faload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::fastore => fastore(jvm, interpreter_state),
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
        CInstructionInfo::freturn => freturn(jvm, interpreter_state),
        CInstructionInfo::fstore(i) => fstore(jvm, interpreter_state.current_frame_mut(), *i as u16),
        CInstructionInfo::fstore_0 => fstore(jvm, interpreter_state.current_frame_mut(), 0),
        CInstructionInfo::fstore_1 => fstore(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::fstore_2 => fstore(jvm, interpreter_state.current_frame_mut(), 2),
        CInstructionInfo::fstore_3 => fstore(jvm, interpreter_state.current_frame_mut(), 3),
        CInstructionInfo::fsub => fsub(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::getfield { desc, target_class, name } => get_field(jvm, interpreter_state, *target_class, *name, desc, false),
        CInstructionInfo::getstatic { name, target_class, desc } => get_static(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::goto_(target) => goto_(jvm, interpreter_state.current_frame_mut(), *target as i32),
        CInstructionInfo::goto_w(target) => goto_(jvm, interpreter_state.current_frame_mut(), *target),
        CInstructionInfo::i2b => i2b(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2c => i2c(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2d => i2d(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2f => i2f(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2l => i2l(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::i2s => i2s(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iadd => iadd(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iaload => iaload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iand => iand(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::iastore => iastore(jvm, interpreter_state),
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
        CInstructionInfo::invokeinterface { classname_ref_type, descriptor, method_name, count } => invoke_interface(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor, *count),
        CInstructionInfo::invokespecial { method_name, descriptor, classname_ref_type } => invoke_special(jvm, interpreter_state, classname_ref_type.unwrap_object_name(), *method_name, descriptor),
        CInstructionInfo::invokestatic { method_name, descriptor, classname_ref_type } => run_invoke_static(jvm, interpreter_state, classname_ref_type.clone(), *method_name, descriptor),
        CInstructionInfo::invokevirtual { method_name, descriptor, classname_ref_type: _ } => invoke_virtual_instruction(jvm, interpreter_state, *method_name, descriptor),
        CInstructionInfo::ior => ior(jvm, interpreter_state.current_frame_mut()),
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
        CInstructionInfo::lconst_1 => lconst(jvm, interpreter_state.current_frame_mut(), 1),
        CInstructionInfo::ldc(cldc2w) => ldc_w(jvm, interpreter_state, &cldc2w.as_ref()),
        CInstructionInfo::ldc_w(cldcw) => ldc_w(jvm, interpreter_state, &Either::Left(cldcw)),
        CInstructionInfo::ldc2_w(cldc2w) => ldc2_w(jvm, interpreter_state.current_frame_mut(), cldc2w),
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
        CInstructionInfo::lreturn => lreturn(jvm, interpreter_state),
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
            let mut stack_entry_mut: StackEntryMut<'gc_life, '_> = interpreter_state.current_frame_mut();
            let popped: JavaValue<'gc_life> = stack_entry_mut.pop(Some(RuntimeType::object()));
            let gc_managed_object: GcManagedObject<'gc_life> = popped.unwrap_object_nonnull();
            gc_managed_object.monitor_lock(jvm, interpreter_state);
        }
        CInstructionInfo::monitorexit => {
            interpreter_state.current_frame_mut().pop(Some(RuntimeType::object())).unwrap_object_nonnull().monitor_unlock(jvm, interpreter_state);
        }
        CInstructionInfo::multianewarray { type_, dimensions } => multi_a_new_array(jvm, interpreter_state, dimensions.get(), type_),
        CInstructionInfo::new(cn) => new(jvm, interpreter_state, *cn),
        CInstructionInfo::newarray(a_type) => newarray(jvm, interpreter_state, *a_type),
        CInstructionInfo::nop => {}
        CInstructionInfo::pop => pop(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::pop2 => pop2(jvm, method_id, interpreter_state.current_frame_mut()),
        CInstructionInfo::putfield { name, desc, target_class } => putfield(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::putstatic { name, desc, target_class } => putstatic(jvm, interpreter_state, *target_class, *name, desc),
        CInstructionInfo::ret(local_var_index) => ret(jvm, interpreter_state.current_frame_mut(), *local_var_index as u16),
        CInstructionInfo::return_ => return_(interpreter_state),
        CInstructionInfo::saload => saload(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::sastore => sastore(jvm, interpreter_state),
        CInstructionInfo::sipush(val) => sipush(jvm, interpreter_state.current_frame_mut(), *val),
        CInstructionInfo::swap => swap(jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::tableswitch(switch) => tableswitch(switch.deref(), jvm, interpreter_state.current_frame_mut()),
        CInstructionInfo::wide(w) => wide(jvm, interpreter_state.current_frame_mut(), w),
        CInstructionInfo::EndOfCode => panic!(),
    }
}

fn l2d(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val = current_frame.pop(Some(RuntimeType::LongType)).unwrap_long();
    current_frame.push(JavaValue::Double(val as f64))
}

fn jsr(interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, target: i32) {
    let next_instruct = (interpreter_state.current_pc() as i32 + interpreter_state.current_pc_offset()) as i64;
    interpreter_state.push_current_operand_stack(JavaValue::Long(next_instruct));
    interpreter_state.set_current_pc_offset(target);
}

fn f2l(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
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

fn dup2_x2(jvm: &'gc_life JVMState<'gc_life>, method_id: MethodId, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let current_pc = current_frame.to_ref().pc(jvm);
    let stack_frames = &jvm.function_frame_type_data.read().unwrap()[&method_id];
    let Frame { stack_map: OperandStack { data }, .. } = todo!(); //&stack_frames[&current_pc];
    let value1_vtype = data[0].clone();
    let value2_vtype = data[1].clone();
    let value1 = current_frame.pop(Some(RuntimeType::LongType));
    let value2 = current_frame.pop(Some(RuntimeType::LongType));
    match value1_vtype {
        VType::LongType | VType::DoubleType => {
            match value2_vtype {
                VType::LongType | VType::DoubleType => {
                    //form 4
                    current_frame.push(value1.clone());
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
                _ => {
                    //form 2
                    let value3 = current_frame.pop(Some(RuntimeType::LongType));
                    // assert!(value3.is_size_1());
                    current_frame.push(value1.clone());
                    current_frame.push(value3);
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
            }
        }
        _ => {
            // assert!(value2.is_size_1());
            let value2_vtype = data[2].clone();
            let value3 = current_frame.pop(Some(RuntimeType::LongType));
            match value2_vtype {
                VType::LongType | VType::DoubleType => {
                    //form 3
                    current_frame.push(value2.clone());
                    current_frame.push(value1.clone());
                    current_frame.push(value3);
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
                _ => {
                    //form 1
                    let value4 = current_frame.pop(Some(RuntimeType::LongType));
                    // assert!(value4.is_size_1());
                    current_frame.push(value2.clone());
                    current_frame.push(value1.clone());
                    current_frame.push(value4);
                    current_frame.push(value3);
                    current_frame.push(value2);
                    current_frame.push(value1);
                }
            }
        }
    }
}

fn frem(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let value1 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    let res = drem_impl(value2 as f64, value1 as f64) as f32;
    current_frame.push(JavaValue::Float(res));
}

fn fneg(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_float();
    current_frame.push(JavaValue::Float(-val))
}

fn drem(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let value2 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_double(); //divisor
    let value1 = current_frame.pop(Some(RuntimeType::FloatType)).unwrap_double();
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

fn dneg(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    current_frame.push(JavaValue::Double(-val))
}

fn swap(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let first = current_frame.pop(Some(RuntimeType::LongType));
    let second = current_frame.pop(Some(RuntimeType::LongType));
    current_frame.push(first);
    current_frame.push(second);
}

pub fn ret(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, local_var_index: u16) {
    let ret = current_frame.local_vars().get(local_var_index, RuntimeType::LongType).unwrap_long();
    current_frame.set_pc(ret as u16);
    *current_frame.pc_offset_mut() = 0;
}

fn dcmpl(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val2 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let val1 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(JavaValue::Int(-1));
    }
    dcmp_common(jvm, current_frame, val2, val1);
}

fn dcmp_common(_jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>, val2: f64, val1: f64) {
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

fn dcmpg(jvm: &'gc_life JVMState<'gc_life>, mut current_frame: StackEntryMut<'gc_life, 'l>) {
    let val2 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    let val1 = current_frame.pop(Some(RuntimeType::DoubleType)).unwrap_double();
    if val2.is_nan() || val1.is_nan() {
        current_frame.push(JavaValue::Int(-1));
    }
    dcmp_common(jvm, current_frame, val2, val1)
}

fn athrow(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) {
    let exception_obj = {
        let value = interpreter_state.pop_current_operand_stack(Some(CClassName::throwable().into()));
        // let value = interpreter_state.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap();
        value.unwrap_object_nonnull()
    };
    // if jvm.debug_print_exceptions {
    println!("EXCEPTION:");
    interpreter_state.debug_print_stack_trace(jvm);
    /*dbg!(exception_obj.lookup_field(jvm, FieldName::field_detailMessage()));*/
    // }

    interpreter_state.set_throw(exception_obj.into());
}