use std::borrow::Borrow;
use std::sync::Arc;

use itertools::Itertools;

use another_jit_vm_ir::WasException;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::{ByteCodeOffset, MethodId, NativeJavaValue};
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CompressedParsedRefType};
use rust_jvm_common::compressed_classfile::code::{CompressedExceptionTableElem, CompressedInstructionInfo};

use crate::{JavaValueCommon, StackEntryPush};
use crate::class_objects::get_or_create_class_object;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::special::instance_of_exit_impl_impl_impl;
use crate::interpreter::real_interpreter_state::{RealInterpreterStateGuard, RealInterpreterStateSave};
use crate::interpreter::single_instruction::run_single_instruction;
use crate::interpreter_state::{FramePushGuard, InterpreterStateGuard};
use crate::ir_to_java_layer::exit_impls::new_run_native::{setup_native_special_args, setup_static_native_args};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID};
use crate::java_values::native_to_new_java_value;
use crate::jit::MethodResolverImpl;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::threading::safepoints::Monitor2;

pub mod single_instruction;
pub mod real_interpreter_state;
pub mod load;
pub mod consts;
pub mod fields;
pub mod new;
pub mod dup;
pub mod ldc;
pub mod store;
pub mod branch;
pub mod special;
pub mod conversion;
pub mod arithmetic;
pub mod cmp;
pub mod wide;
pub mod switch;
pub mod pop;
pub mod throw;


pub struct FrameToRunOn {
    pub frame_pointer: JavaStackPosition,
    pub size: usize,
}

//takes exclusive framepush guard so I know I can mut the frame rip safelyish maybe. todo have a better way of doing this
pub fn run_function<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let rc = interpreter_state.current_frame().class_pointer(jvm);
    let method_i = interpreter_state.current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_i);
    let view = interpreter_state.current_class_view(jvm).clone();
    let method = view.method_view_i(method_i);
    let code = method.code_attribute().unwrap();
    let resolver = MethodResolverImpl { jvm, loader: interpreter_state.current_loader(jvm) };
    let compile_interpreted = !(jvm.config.compiled_mode_active && jvm.function_execution_count.function_instruction_count(method_id) >= jvm.config.compile_threshold);

    if !compile_interpreted {
        jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, false);
    }

    let ir_method_id = match jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }) {
        None => {
            jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, false);
            jvm.java_vm_state.lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 })
        }
        Some(ir_method_id) => ir_method_id
    };
    interpreter_state.current_frame_mut().frame_view.ir_mut.set_ir_method_id(ir_method_id);
    interpreter_state.current_frame_mut().frame_view.assert_prev_rip(jvm.java_vm_state.ir.get_top_level_return_ir_method_id(), jvm);
    assert!((interpreter_state.current_frame().frame_view.ir_ref.method_id() == Some(method_id)));

    if !jvm.instruction_trace_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.push(None);
    }
    let function_res = jvm.java_vm_state.run_method(jvm, interpreter_state, method_id)?;
    // assert_eq!(jvm.java_vm_state.assertion_state.lock().unwrap().method_ids.pop().unwrap(), method_id);
    // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
    //todo bug what if gc happens here
    if !jvm.instruction_trace_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().current_before = restore_clone;
    }
    let return_type = &method.desc().return_type;
    Ok(match return_type {
        CompressedParsedDescriptorType::VoidType => None,
        return_type => {
            let native_value = NativeJavaValue { as_u64: function_res };
            /*unsafe {
                eprintln!("{:X}",native_value.as_u64);
            }*/
            Some(native_to_new_java_value(native_value, *return_type, jvm))
        }
    })
}


pub enum PostInstructionAction<'gc> {
    NextOffset {
        offset_change: i32,
    },
    Return {
        res: Option<NewJavaValueHandle<'gc>>
    },
    Exception {
        exception: WasException
    },
    Call {
        method_id: MethodId,
        local_vars: Vec<NewJavaValueHandle<'gc>>,
    },
    NativeCall {
        method_id: MethodId,
    },
    Next {},
}

pub enum PostFunctionAction<'gc> {
    Return {
        res: Option<NewJavaValueHandle<'gc>>
    },
    Call {
        method_id: MethodId,
        local_vars: Vec<NewJavaValueHandle<'gc>>,
    },
    Exception {
        exception: WasException
    },
}

pub struct FunctionCallState {
    current_offset: ByteCodeOffset,
    current_method_id: MethodId,
    frame_push_guard: Option<FramePushGuard>,
    instruction_size_for_invoke_skip: Option<u16>,
    real_interpreter_state_save: RealInterpreterStateSave,
}

impl FunctionCallState {
    pub fn real_interpreter_state<'gc, 'l, 'k>(&self, jvm: &'gc JVMState<'gc>, interpreter_state: &'k mut InterpreterStateGuard<'gc, 'l>) -> RealInterpreterStateGuard<'gc, 'l, 'k> {
        RealInterpreterStateGuard::from_save(jvm, interpreter_state, self.real_interpreter_state_save.clone())
    }
}

pub fn run_function_interpreted<'l, 'gc>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>, frame_push_guard: Option<FramePushGuard>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    // eprintln!("{}",Backtrace::force_capture().to_string());
    let current_method_id = interpreter_state.current_frame().method_id();
    let mut real_interpreter_state = RealInterpreterStateGuard::new(jvm, interpreter_state);
    let mut function_stack = vec![FunctionCallState {
        current_offset: ByteCodeOffset(0),
        current_method_id,
        frame_push_guard,
        instruction_size_for_invoke_skip: None,
        real_interpreter_state_save: real_interpreter_state.save(),
    }];
    'outer: loop {
        match run_current_function_interpreted(jvm, real_interpreter_state.inner(), function_stack.last_mut().unwrap()) {
            PostFunctionAction::Return { res } => {
                let inner = real_interpreter_state.inner();
                let current_function = function_stack.pop().unwrap();
                if let Some(frame_push_guard) = current_function.frame_push_guard {
                    inner.pop_frame(jvm, frame_push_guard, false);
                }
                if function_stack.is_empty() {
                    return Ok(res);
                }
                function_stack.last_mut().unwrap().current_offset.0 += function_stack.last_mut().unwrap().instruction_size_for_invoke_skip.unwrap();
                if let Some(res) = res {
                    function_stack.last_mut().unwrap().real_interpreter_state(jvm, inner).current_frame_mut().push(res.to_interpreter_jv());
                }
            }
            PostFunctionAction::Call { method_id, local_vars } => {
                let inner = real_interpreter_state.inner();
                let frame_push_guard = inner.push_frame(StackEntryPush::Java {
                    method_id,
                    local_vars: local_vars.iter().map(|njvh| njvh.as_njv()).collect_vec(),
                    operand_stack: vec![],
                });
                function_stack.push(FunctionCallState {
                    current_offset: ByteCodeOffset(0),
                    current_method_id: method_id,
                    frame_push_guard: Some(frame_push_guard),
                    instruction_size_for_invoke_skip: None,
                    real_interpreter_state_save: RealInterpreterStateGuard::new(jvm, inner).save(),
                });
            }
            PostFunctionAction::Exception { exception } => {
                let inner = real_interpreter_state.inner();
                loop {
                    let rc = inner.current_frame().class_pointer(jvm);
                    let method_i = inner.current_frame().method_i(jvm);
                    let view = rc.view();
                    let method_view = view.method_view_i(method_i);
                    let code = method_view.code_attribute().unwrap();
                    for CompressedExceptionTableElem {
                        start_pc,
                        end_pc,
                        handler_pc,
                        catch_type
                    } in code.exception_table.iter() {
                        let rc = inner.throw().unwrap().runtime_class(jvm);
                        // dump_frame(&mut real_interpreter_state,&method,code);
                        let current_function = function_stack.last_mut().unwrap();
                        if *start_pc <= current_function.current_offset && current_function.current_offset < *end_pc {
                            let matches_class = match catch_type {
                                None => true,
                                Some(class_name) => {
                                    instance_of_exit_impl_impl_impl(jvm, CompressedParsedRefType::Class(*class_name), rc) == 1
                                }
                            };
                            if matches_class {
                                current_function.current_offset = *handler_pc;
                                let throw_obj = inner.throw().unwrap().duplicate_discouraged().new_java_handle();
                                inner.set_throw(None);
                                real_interpreter_state.current_stack_depth_from_start = 0;
                                real_interpreter_state.current_frame_mut().push(throw_obj.to_interpreter_jv());
                                continue 'outer;
                            }
                        }
                    }
                    let this_frame = function_stack.pop().unwrap();
                    if let Some(frame_push_guard) = this_frame.frame_push_guard {
                        inner.pop_frame(jvm, frame_push_guard, true);
                    }
                    if function_stack.is_empty() {
                        return Err(WasException {});
                    }
                }
            }
        }
    }
}

fn run_current_function_interpreted<'gc>(
    jvm: &'gc JVMState<'gc>,
    interpreter_state: &mut InterpreterStateGuard<'gc, '_>,
    current_function: &mut FunctionCallState,
) -> PostFunctionAction<'gc> {
    let method_id = current_function.current_method_id;
    let function_counter = jvm.function_execution_count.for_function(method_id);
    let mut real_interpreter_state = current_function.real_interpreter_state(jvm, interpreter_state);
    let FunctionCallState { current_offset, current_method_id, frame_push_guard: _, instruction_size_for_invoke_skip, real_interpreter_state_save } = current_function;
    let rc = real_interpreter_state.inner().current_frame().class_pointer(jvm);
    let method_i = real_interpreter_state.inner().current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_i);
    assert_eq!(method_id, *current_method_id);
    let view = real_interpreter_state.inner().current_class_view(jvm).clone();
    let method = view.method_view_i(method_i);
    eprintln!("Interpreted:{}/{}", view.name().unwrap_name().0.to_str(&jvm.string_pool), method.name().0.to_str(&jvm.string_pool));
    let code = match method.code_attribute() {
        Some(code) => code,
        None => {
            panic!()
        }
    };
    let resolver = MethodResolverImpl { jvm, loader: real_interpreter_state.inner().current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, true);
    loop {
        let current_instruct = &code.instructions.get(&current_offset).unwrap();
        assert!(real_interpreter_state.current_stack_depth_from_start <= code.max_stack);
        current_function.instruction_size_for_invoke_skip = Some(current_instruct.instruction_size);
        match run_single_instruction(jvm, &mut real_interpreter_state, &current_instruct.info, &function_counter, &method, code, *current_offset) {
            PostInstructionAction::NextOffset { offset_change } => {
                let next_offset = current_offset.0 as i32 + offset_change;
                current_offset.0 = next_offset as u16;
            }
            PostInstructionAction::Return { res } => {
                return PostFunctionAction::Return { res };
            }
            PostInstructionAction::Exception { .. } => {
                return PostFunctionAction::Exception { exception: WasException {} };
            }
            PostInstructionAction::Next { .. } => {
                current_offset.0 += current_instruct.instruction_size;
            }
            PostInstructionAction::Call { method_id, local_vars } => {
                assert!(!jvm.is_native_by_method_id(method_id));
                return PostFunctionAction::Call { method_id, local_vars };
            }
            PostInstructionAction::NativeCall { method_id } => {
                let (rc, method_i) = jvm.method_table.read().unwrap().try_lookup(method_id).unwrap();
                let view = rc.view();
                let method_view = view.method_view_i(method_i);
                let args = if method.is_static() {
                    setup_static_native_args(jvm, method_id, &method, real_interpreter_state.inner().current_frame())
                } else {
                    setup_native_special_args(jvm, method_id, &method, real_interpreter_state.inner().current_frame())
                };
                match run_native_method(jvm, real_interpreter_state.inner(), rc.clone(), method_i, args.iter().map(|njvh| njvh.as_njv()).collect_vec()) {
                    Ok(res) => {
                        if let Some(res) = res{
                            real_interpreter_state.current_frame_mut().push(res.to_interpreter_jv());
                        }
                        current_offset.0 += current_instruct.instruction_size;
                    }
                    Err(WasException {}) => {
                        return PostFunctionAction::Exception { exception: WasException{} }
                    }
                }
            }
        }
    }
}


pub fn safepoint_check<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<(), WasException> {
    let thread = interpreter_state.thread().clone();
    let safe_point = thread.safepoint_state.borrow();
    safe_point.check(jvm, interpreter_state)
}


// fn breakpoint_check<'l, 'gc>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>, methodid: MethodId) {
//     let pc = interpreter_state.current_pc();
//     let stop = match jvm.jvmti_state() {
//         None => false,
//         Some(jvmti) => {
//             let breakpoints = &jvmti.break_points.read().unwrap();
//             let function_breakpoints = breakpoints.get(&methodid);
//             function_breakpoints.map(|points| points.contains(&pc)).unwrap_or(false)
//         }
//     };
//     if stop {
//         let jdwp = &jvm.jvmti_state().unwrap().built_in_jdwp;
//         jdwp.breakpoint(jvm, methodid, pc.0 as i64, interpreter_state);
//     }
// }

pub fn monitor_for_function<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, method: &MethodView, synchronized: bool) -> Option<Arc<Monitor2>> {
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