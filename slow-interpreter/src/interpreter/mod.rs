use std::os::raw::c_void;
use std::sync::Arc;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use common::special::instance_of_exit_impl_impl_impl;
use rust_jvm_common::{ByteCodeOffset, StackNativeJavaValue};
use rust_jvm_common::classfile::{LineNumber};
use rust_jvm_common::compressed_classfile::code::CompressedExceptionTableElem;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedDescriptorType, CompressedParsedRefType, CPDType};


use rust_jvm_common::runtime_type::RuntimeType;

use crate::{NewAsObjectOrJavaValue, WasException};
use crate::better_java_stack::frames::{HasFrame, PushableFrame};
use crate::better_java_stack::interpreter_frame::JavaInterpreterFrame;
use crate::better_java_stack::StackDepth;
use crate::class_objects::get_or_create_class_object;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::interpreter::single_instruction::run_single_instruction;
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OpaqueFrameIdOrMethodID};
use crate::java_values::{native_to_new_java_value_cpdtype};
use crate::jit::MethodResolverImpl;
use crate::jvm_state::JVMState;
use crate::new_java_values::java_value_common::JavaValueCommon;
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
pub mod common;


pub struct FrameToRunOn {
    pub frame_pointer: JavaStackPosition,
    pub size: usize,
}

//takes exclusive framepush guard so I know I can mut the frame rip safelyish maybe. todo have a better way of doing this
pub fn run_function<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &mut JavaInterpreterFrame<'gc, 'l>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    // let should_trace = unsafe { libc::rand() } < 1000000;
    // if should_trace {
    //     interpreter_state.debug_print_stack_trace(jvm);
    // }
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
        return run_function_interpreted(jvm, interpreter_state);
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
    assert!((interpreter_state.frame_ref().method_id() == Ok(method_id)));

    if !jvm.instruction_tracing_options.partial_tracing() {
        // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.push(None);
    }
    let function_res = jvm.java_vm_state.run_method(jvm, interpreter_state, method_id)?;
    // assert_eq!(jvm.java_vm_state.assertion_state.lock().unwrap().method_ids.pop().unwrap(), method_id);
    // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
    //todo bug what if gc happens here
    if !jvm.instruction_tracing_options.partial_tracing() {
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
            let native_value = StackNativeJavaValue { as_u64: function_res };
            /*unsafe {
                eprintln!("{:X}",native_value.as_u64);
            }*/
            Some(native_to_new_java_value_cpdtype(native_value, *return_type, jvm))
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
        exception: WasException<'gc>
    },
    Next {},
}

static mut INDENT:usize = 10;

pub fn run_function_interpreted<'l, 'gc>(jvm: &'gc JVMState<'gc>, interpreter_state: &mut JavaInterpreterFrame<'gc, 'l>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException<'gc>> {
    // eprintln!("{}",Backtrace::force_capture().to_string());
    let rc = interpreter_state.class_pointer(jvm);
    let method_i = interpreter_state.current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc.clone(), method_i);
    let view = interpreter_state.current_class_view(jvm).clone();
    let method = view.method_view_i(method_i);
    if interpreter_state.should_be_tracing_function_calls(){
        unsafe {
            for _ in 0..INDENT {
                eprint!(" ");
            }
            INDENT = INDENT.saturating_add(1);
        }
        let thread_name_cached = interpreter_state.thread_name_cached();
        let line_number = match method.line_number_table() {
            None => None,
            Some(line_number_table) => {
                match interpreter_state.frame_iter().next().unwrap().try_pc() {
                    None => None,
                    Some(pc) => {
                        line_number_table.lookup_pc(pc)
                    }
                }
            }
        };
        eprintln!("Method entered: \"thread={thread_name_cached}\", {}.{}(), line={} bci=0",
                  rc.cpdtype().java_source_representation(&jvm.string_pool).replace("/","."),
                  method.name().0.to_str(&jvm.string_pool),
                  line_number.unwrap_or(LineNumber(u16::MAX)).0);
    }
    let code = method.code_attribute().unwrap();
    let resolver = MethodResolverImpl { jvm, loader: interpreter_state.current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id, true);
    let function_counter = jvm.function_execution_count.for_function(method_id);
    let mut current_offset = ByteCodeOffset(0);
    let mut real_interpreter_state = RealInterpreterStateGuard::new(jvm, interpreter_state);
    let should_sync = if method.is_synchronized() {
        if method.is_static() {
            //todo
            let class_obj = jvm.classes.read().unwrap().get_class_obj_from_runtime_class(rc.clone());
            let monitor = jvm.monitor_for(class_obj.ptr.as_ptr() as *const c_void);
            monitor.lock(jvm, real_interpreter_state.inner()).unwrap();
            Some(monitor)
        } else {
            let obj = real_interpreter_state.current_frame_mut().local_get(0, RuntimeType::object());
            let monitor = jvm.monitor_for(obj.unwrap_object().unwrap().as_ptr() as *const c_void);
            monitor.lock(jvm, real_interpreter_state.inner()).unwrap();
            Some(monitor)
        }
    } else {
        None
    };
    safepoint_check(jvm, real_interpreter_state.inner())?;
    'outer: loop {
        let current_instruct = code.instructions.get(&current_offset).unwrap();
        assert!(real_interpreter_state.current_stack_depth_from_start <= code.max_stack);
        let stack_depth = StackDepth(real_interpreter_state.current_stack_depth_from_start);
        real_interpreter_state.inner().update_stack_depth(current_offset, stack_depth);
        match run_single_instruction(jvm, &mut real_interpreter_state, &current_instruct.info, &function_counter, &method, code, current_offset) {
            PostInstructionAction::NextOffset { offset_change } => {
                let next_offset = current_offset.0 as i32 + offset_change;
                current_offset.0 = next_offset as u16;
            }
            PostInstructionAction::Return { res } => {
                if real_interpreter_state.inner().should_be_tracing_function_calls(){
                    unsafe {
                        INDENT = INDENT.saturating_sub(1);
                        for _ in 0..INDENT {
                            eprint!(" ");
                        }
                    }
                    let thread_name_cached = real_interpreter_state.inner().thread_name_cached();
                    let line_number = match method.line_number_table() {
                        None => None,
                        Some(line_number_table) => {
                            match real_interpreter_state.inner().frame_iter().next().unwrap().try_pc() {
                                None => None,
                                Some(pc) => {
                                    line_number_table.lookup_pc(pc)
                                }
                            }
                        }
                    };
                    println!("Method exited: return value= , \"thread={thread_name_cached}\", {}.{}(), line={} bci=0",
                             rc.cpdtype().java_source_representation(&jvm.string_pool).replace("/","."),
                             method.name().0.to_str(&jvm.string_pool),
                             line_number.unwrap_or(LineNumber(u16::MAX)).0);
                }
                if let Some(monitor) = should_sync {
                    monitor.unlock(jvm, real_interpreter_state.inner()).unwrap();
                }
                return Ok(res.map(|res|coerce_integer_types_to(res,method.desc().return_type)));
            }
            PostInstructionAction::Exception { exception: WasException { exception_obj } } => {
                // real_interpreter_state.inner().set_current_pc(None);
                assert!(real_interpreter_state.current_stack_depth_from_start <= code.max_stack);
                for CompressedExceptionTableElem {
                    start_pc,
                    end_pc,
                    handler_pc,
                    catch_type
                } in code.exception_table.iter() {
                    let rc = exception_obj.full_object_ref().runtime_class(jvm);
                    // dump_frame(&mut real_interpreter_state,&method,code);
                    if *start_pc <= current_offset && current_offset < *end_pc {
                        let matches_class = match catch_type {
                            None => true,
                            Some(class_name) => {
                                let throw = exception_obj.normal_object.as_allocated_obj().duplicate_discouraged();
                                instance_of_exit_impl_impl_impl(jvm, CompressedParsedRefType::Class(*class_name), rc, &throw) == 1
                            }
                        };
                        if matches_class {
                            current_offset = *handler_pc;
                            let throw_obj = exception_obj.normal_object.duplicate_discouraged().new_java_handle();
                            real_interpreter_state.current_frame_mut().pop_all();
                            real_interpreter_state.current_frame_mut().push(throw_obj.to_interpreter_jv());
                            continue 'outer;
                        }
                    }
                }
                if let Some(monitor) = should_sync {
                    monitor.unlock(jvm, real_interpreter_state.inner()).unwrap();
                }
                return Err(WasException { exception_obj });
            }
            PostInstructionAction::Next { .. } => {
                current_offset.0 += current_instruct.instruction_size;
            }
        }
    }
}

fn coerce_integer_types_to<'gc>(handle: NewJavaValueHandle<'gc>, cpdtype: CPDType) -> NewJavaValueHandle<'gc>{
    match cpdtype {
        CPDType::BooleanType => {
            NewJavaValueHandle::Boolean(handle.unwrap_int() as u8)
        }
        CPDType::ByteType => {
            NewJavaValueHandle::Byte(handle.unwrap_int() as i8)
        }
        CPDType::ShortType => {
            NewJavaValueHandle::Short(handle.unwrap_int() as i16)
        }
        CPDType::CharType => {
            NewJavaValueHandle::Char(handle.unwrap_int() as u16)
        }
        CPDType::IntType => {
            handle
        }
        CPDType::LongType => {
            handle
        }
        CPDType::FloatType => {
            handle
        }
        CPDType::DoubleType => {
            handle
        }
        CPDType::VoidType => {
            todo!()
        }
        CPDType::Class(_) |
        CPDType::Array { .. } => {
            handle
        }
    }
}


pub fn safepoint_check<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &mut impl HasFrame<'gc>) -> Result<(), WasException<'gc>> {
    let thread = interpreter_state.java_thread().clone();
    thread.safepoint_state.check(jvm, interpreter_state)?;
    Ok(())
}


// fn breakpoint_check<'l, 'gc>(jvm: &'gc JVMState<'gc>, interpreter_state: &mut impl PushableFrame<'gc>, methodid: MethodId) {
//     let pc = interpreter_state.current_pc();
//     let stop = match jvm.jvmti_state() {
//         None => false,
//         Some(jvmti_interface) => {
//             let breakpoints = &jvmti_interface.break_points.read().unwrap();
//             let function_breakpoints = breakpoints.get(&methodid);
//             function_breakpoints.map(|points| points.contains(&pc)).unwrap_or(false)
//         }
//     };
//     if stop {
//         let jdwp = &jvm.jvmti_state().unwrap().built_in_jdwp;
//         jdwp.breakpoint(jvm, methodid, pc.0 as i64, interpreter_state);
//     }
// }

pub fn monitor_for_function<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method: &MethodView, synchronized: bool) -> Option<Arc<Monitor2>> {
    if synchronized {
        let monitor: Arc<Monitor2> = if method.is_static() {
            let class_object = get_or_create_class_object(jvm, method.classview().type_(), int_state).unwrap();
            jvm.monitor_for(class_object.ptr.as_ptr())
        } else {
            let ptr = int_state.local_get_handle(0, RuntimeType::object()).unwrap_object_nonnull().ptr().as_ptr();
            jvm.monitor_for(ptr)
        };
        monitor.lock(jvm, int_state).unwrap();
        monitor.into()
    } else {
        None
    }
}

pub static mut TIMES: usize = 0;