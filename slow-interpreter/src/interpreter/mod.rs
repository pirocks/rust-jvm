use std::borrow::{Borrow};
use std::sync::Arc;


use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType};
use rust_jvm_common::{ByteCodeOffset, NativeJavaValue};

use crate::class_objects::get_or_create_class_object;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::interpreter::single_instruction::run_single_instruction;
use crate::interpreter_state::{FramePushGuard, InterpreterStateGuard};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition};
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
pub mod wide{
    use rust_jvm_common::classfile::{IInc, Wide, WideAload, WideAstore, WideDload, WideDstore, WideFload, WideFstore, WideIload, WideIstore, WideLload, WideLstore, WideRet};
    use rust_jvm_common::runtime_type::RuntimeType;
    use crate::interpreter::load::{aload, dload, fload, iload, lload};
    use crate::interpreter::PostInstructionAction;
    use crate::interpreter::real_interpreter_state::{InterpreterFrame, InterpreterJavaValue};
    use crate::interpreter::store::{astore, dstore, fstore, istore, lstore};
    use crate::JVMState;

    pub fn wide<'gc, 'j, 'k, 'l>(jvm: &'gc JVMState<'gc>, mut current_frame: InterpreterFrame<'gc, 'l, 'k, 'j>, w: &Wide) -> PostInstructionAction<'gc>{
        match w {
            Wide::Iload(WideIload { index }) => iload(jvm, current_frame, *index),
            Wide::Fload(WideFload { index }) => fload(jvm, current_frame, *index),
            Wide::Aload(WideAload { index }) => aload(current_frame, *index),
            Wide::Lload(WideLload { index }) => lload(jvm, current_frame, *index),
            Wide::Dload(WideDload { index }) => dload(jvm, current_frame, *index),
            Wide::Istore(WideIstore { index }) => istore(jvm, current_frame, *index),
            Wide::Fstore(WideFstore { index }) => fstore(jvm, current_frame, *index),
            Wide::Astore(WideAstore { index }) => astore(jvm,current_frame, *index),
            Wide::Lstore(WideLstore { index }) => lstore(jvm, current_frame, *index),
            Wide::Ret(WideRet { index }) => todo!()/*ret(jvm, current_frame, *index)*/,
            Wide::Dstore(WideDstore { index }) => dstore(jvm, current_frame, *index),
            Wide::IInc(iinc) => {
                let IInc { index, const_ } = iinc;
                let mut val = current_frame.local_get(*index, RuntimeType::IntType).unwrap_int();
                val += *const_ as i32;
                current_frame.local_set(*index, InterpreterJavaValue::Int(val));
                PostInstructionAction::Next {}
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WasException;


pub struct FrameToRunOn {
    pub frame_pointer: JavaStackPosition,
    pub size: usize,
}

//takes exclusive framepush guard so I know I can mut the frame rip safelyish maybe. todo have a better way of doing this
pub fn run_function<'gc, 'l>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>, frame_guard: &mut FramePushGuard) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let rc = interpreter_state.current_frame().class_pointer(jvm);
    let method_i = interpreter_state.current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_i);
    if jvm.config.compiled_mode_active && jvm.function_execution_count.function_instruction_count(method_id) >= 1 {
        let view = interpreter_state.current_class_view(jvm).clone();
        let method = view.method_view_i(method_i);
        let code = method.code_attribute().unwrap();
        let resolver = MethodResolverImpl { jvm, loader: interpreter_state.current_loader(jvm) };
        jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id);
        interpreter_state.current_frame_mut().frame_view.assert_prev_rip(jvm.java_vm_state.ir.get_top_level_return_ir_method_id(), jvm);
        assert!((interpreter_state.current_frame().frame_view.ir_ref.method_id() == Some(method_id)));
        if !jvm.instruction_trace_options.partial_tracing() {
            // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.push(None);
        }
        let function_res = jvm.java_vm_state.run_method(jvm, interpreter_state, method_id);
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
                Some(native_to_new_java_value(native_value, *return_type, jvm))
            }
        })
    } else {
        run_function_interpreted(&jvm, interpreter_state)
    }
}


pub enum PostInstructionAction<'gc> {
    NextOffset {
        offset_change: i32,
    },
    Return {
        res: Option<NewJavaValueHandle<'gc>>
    },
    Exception{
        exception: WasException
    },
    Next{}
}

pub fn run_function_interpreted<'l, 'gc>(jvm: &'gc JVMState<'gc>, interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let rc = interpreter_state.current_frame().class_pointer(jvm);
    let method_i = interpreter_state.current_method_i(jvm);
    let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_i);
    let view = interpreter_state.current_class_view(jvm).clone();
    let method = view.method_view_i(method_i);
    let code = method.code_attribute().unwrap();
    let resolver = MethodResolverImpl { jvm, loader: interpreter_state.current_loader(jvm) };
    jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id);
    interpreter_state.current_frame_mut().frame_view.assert_prev_rip(jvm.java_vm_state.ir.get_top_level_return_ir_method_id(), jvm);
    let function_counter = jvm.function_execution_count.for_function(method_id);
    let mut current_offset = ByteCodeOffset(0);
    let mut real_interpreter_state = RealInterpreterStateGuard::new(jvm, interpreter_state);
    loop {
        let current_instruct = code.instructions.get(&current_offset).unwrap();
        match run_single_instruction(jvm, &mut real_interpreter_state, &current_instruct.info, &function_counter, &method, code, current_offset) {
            PostInstructionAction::NextOffset { offset_change } => {
                let next_offset = current_offset.0 as i32 + offset_change;
                current_offset.0 = next_offset as u16;
            }
            PostInstructionAction::Return { res } => {
                return Ok(res)
            }
            PostInstructionAction::Exception { .. } => {
                todo!()
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