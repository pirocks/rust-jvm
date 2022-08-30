use std::borrow::Borrow;
use std::os::raw::c_void;
use std::sync::Arc;

use another_jit_vm_ir::WasException;
use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::{ByteCodeOffset, NativeJavaValue};
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CompressedParsedRefType};
use rust_jvm_common::compressed_classfile::code::CompressedExceptionTableElem;
use rust_jvm_common::runtime_type::{RuntimeType};

use crate::AllocatedHandle;
use crate::better_java_stack::frames::HasFrame;
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::class_objects::get_or_create_class_object;
use crate::instructions::special::instance_of_exit_impl_impl_impl;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::interpreter::single_instruction::run_single_instruction;
use crate::interpreter_state::InterpreterStateGuard;
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
pub fn run_function<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &mut JavaInterpreterFrame<'gc, 'l>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let rc = interpreter_state.class_pointer(jvm);
    let method_i = interpreter_state.current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_i);
    let view = interpreter_state.current_class_view(jvm).clone();
    let method = view.method_view_i(method_i);
    let code = method.code_attribute().unwrap();
    let resolver = MethodResolverImpl { jvm, loader: interpreter_state.current_loader(jvm) };
    let compile_interpreted = !(jvm.config.compiled_mode_active && jvm.function_execution_count.function_instruction_count(method_id) >= jvm.config.compile_threshold);

    if !compile_interpreted {
        jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, false);
    } else {
        return run_function_interpreted(jvm, todo!()/*interpreter_state*/);
    }

    let ir_method_id = match jvm.java_vm_state.try_lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 }) {
        None => {
            jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, false);
            jvm.java_vm_state.lookup_ir_method_id(OpaqueFrameIdOrMethodID::Method { method_id: method_id as u64 })
        }
        Some(ir_method_id) => ir_method_id
    };
    interpreter_state.frame_mut().set_ir_method_id(ir_method_id);
    interpreter_state.frame_mut().assert_prev_rip(jvm.java_vm_state.ir.get_top_level_return_ir_pointer().as_ptr());
    assert!((interpreter_state.frame_ref().method_id() == Some(method_id)));

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
    //todo
    // if interpreter_state.throw().is_some(){
    //     return Err(WasException{})
    // }
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
    Next {},
}

pub fn run_function_interpreted<'l, 'gc>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    // eprintln!("{}",Backtrace::force_capture().to_string());
    let rc = interpreter_state.current_frame().class_pointer(jvm);
    let method_i = interpreter_state.current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_i);
    let view = interpreter_state.current_class_view(jvm).clone();
    let method = view.method_view_i(method_i);
    let code = method.code_attribute().unwrap();
    let resolver = MethodResolverImpl { jvm, loader: interpreter_state.current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, true);
    let function_counter = jvm.function_execution_count.for_function(method_id);
    let mut current_offset = ByteCodeOffset(0);
    let mut real_interpreter_state = RealInterpreterStateGuard::new(jvm, interpreter_state);
    let should_sync = if method.is_synchronized(){
        if method.is_static(){
            //todo
            let class_obj = jvm.classes.read().unwrap().get_class_obj_from_runtime_class(rc);
            let monitor = jvm.monitor_for(class_obj.ptr.as_ptr() as *const c_void);
            monitor.lock(jvm,real_interpreter_state.inner()).unwrap();
            Some(monitor)
        }else {
            let obj = real_interpreter_state.current_frame_mut().local_get(0,RuntimeType::object());
            let monitor = jvm.monitor_for(obj.unwrap_object().unwrap().as_ptr() as *const c_void);
            monitor.lock(jvm,real_interpreter_state.inner()).unwrap();
            Some(monitor)
        }
    }else {
        None
    };
    'outer: loop {
        let current_instruct = code.instructions.get(&current_offset).unwrap();
        assert!(real_interpreter_state.current_stack_depth_from_start <= code.max_stack);
        // real_interpreter_state.inner().set_current_pc(Some(current_offset));
        // if method.name().0.to_str(&jvm.string_pool) == "a" && view.name().unwrap_name().0.to_str(&jvm.string_pool) == "aqr"{
        //     eprintln!("Interpreted:{}/{}/{}",view.name().unwrap_name().0.to_str(&jvm.string_pool),method.name().0.to_str(&jvm.string_pool), current_instruct.info.better_debug_string(&jvm.string_pool));
        //     // println!("{}", Backtrace::force_capture());
        // }
        assert!(real_interpreter_state.inner().throw().is_none());
        real_interpreter_state.inner().set_current_pc(None);
        match run_single_instruction(jvm, &mut real_interpreter_state, &current_instruct.info, &function_counter, &method, code, current_offset) {
            PostInstructionAction::NextOffset { offset_change } => {
                let next_offset = current_offset.0 as i32 + offset_change;
                current_offset.0 = next_offset as u16;
            }
            PostInstructionAction::Return { res } => {
                assert!(real_interpreter_state.inner().throw().is_none());
                if let Some(monitor) = should_sync{
                    monitor.unlock(jvm,real_interpreter_state.inner()).unwrap();
                }
                return Ok(res);
            }
            PostInstructionAction::Exception { .. } => {
                real_interpreter_state.inner().set_current_pc(None);
                assert!(real_interpreter_state.current_stack_depth_from_start <= code.max_stack);
                for CompressedExceptionTableElem {
                    start_pc,
                    end_pc,
                    handler_pc,
                    catch_type
                } in code.exception_table.iter() {
                    let rc = real_interpreter_state.inner().throw().unwrap().runtime_class(jvm);
                    // dump_frame(&mut real_interpreter_state,&method,code);
                    if *start_pc <= current_offset && current_offset < *end_pc {
                        let matches_class = match catch_type {
                            None => true,
                            Some(class_name) => {
                                let throw = AllocatedHandle::NormalObject(real_interpreter_state.inner().throw().unwrap().duplicate_discouraged());
                                instance_of_exit_impl_impl_impl(jvm, CompressedParsedRefType::Class(*class_name), rc, &throw) == 1
                            }
                        };
                        if matches_class {
                            current_offset = *handler_pc;
                            let throw_obj = real_interpreter_state.inner().throw().unwrap().duplicate_discouraged().new_java_handle();
                            real_interpreter_state.inner().set_throw(None);
                            real_interpreter_state.current_stack_depth_from_start = 0;
                            real_interpreter_state.current_frame_mut().push(throw_obj.to_interpreter_jv());
                            continue 'outer;
                        }
                    }
                }
                if let Some(monitor) = should_sync{
                    monitor.unlock(jvm,real_interpreter_state.inner()).unwrap();
                }
                return Err(WasException {});
            }
            PostInstructionAction::Next { .. } => {
                current_offset.0 += current_instruct.instruction_size;
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