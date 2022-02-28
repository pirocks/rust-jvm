use std::borrow::{Borrow, BorrowMut};
use std::ffi::c_void;
use std::ops::{Deref, Rem};
use std::sync::Arc;

use itertools::{Either, Itertools};
use num::Zero;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use jvmti_jni_bindings::{jvalue, JVM_ACC_SYNCHRONIZED};
use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::code::CInstructionInfo;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::loading::LoaderName;
use rust_jvm_common::MethodId;
use rust_jvm_common::runtime_type::RuntimeType;
use rust_jvm_common::vtype::VType;
use verification::OperandStack;
use verification::verifier::Frame;

use crate::class_loading::{check_loaded_class, check_resolved_class};
use crate::class_objects::get_or_create_class_object;
use crate::instructions::fields::*;
use crate::instructions::invoke::dynamic::invoke_dynamic;
use crate::instructions::invoke::interface::invoke_interface;
use crate::instructions::invoke::special::invoke_special;
use crate::instructions::invoke::static_::run_invoke_static;
use crate::instructions::invoke::virtual_::invoke_virtual_instruction;
use crate::instructions::ldc::*;
use crate::instructions::new::*;
use crate::instructions::special::*;
use crate::interpreter_state::{FramePushGuard, InterpreterStateGuard};
use crate::ir_to_java_layer::java_stack::{JavaStackPosition, OwnedJavaStack};
use crate::java_values::{GcManagedObject, JavaValue, NativeJavaValue};
use crate::jit::MethodResolver;
use crate::jit::state::JITedCodeState;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
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
pub fn run_function(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, frame_guard: &mut FramePushGuard) -> Result<Option<NewJavaValueHandle<'gc_life>>, WasException> {
    if jvm.config.compiled_mode_active {
        let rc = interpreter_state.current_frame().class_pointer(jvm);
        let method_i = interpreter_state.current_method_i(jvm);
        let method_id = jvm.method_table.write().unwrap().get_method_id(rc, method_i);
        let view = interpreter_state.current_class_view(jvm).clone();
        let method = view.method_view_i(method_i);
        let code = method.code_attribute().unwrap();
        let resolver = MethodResolver { jvm, loader: LoaderName::BootstrapLoader };
        jvm.java_vm_state.add_method_if_needed(jvm, &resolver, method_id);
        interpreter_state.current_frame_mut().frame_view.assert_prev_rip(jvm.java_vm_state.ir.get_top_level_return_ir_method_id(), jvm);
        assert!((interpreter_state.current_frame().frame_view.ir_ref.method_id() == Some(method_id)));
        let restore_clone = jvm.java_vm_state.assertion_state.lock().unwrap().current_before.clone();
        if !jvm.instruction_trace_options.partial_tracing(){
            jvm.java_vm_state.assertion_state.lock().unwrap().current_before.push(None);
        }
        let function_res = jvm.java_vm_state.run_method(jvm, interpreter_state, method_id);
        // assert_eq!(jvm.java_vm_state.assertion_state.lock().unwrap().method_ids.pop().unwrap(), method_id);
        // jvm.java_vm_state.assertion_state.lock().unwrap().current_before.pop().unwrap();
        //todo bug what if gc happens here
        if !jvm.instruction_trace_options.partial_tracing(){
            jvm.java_vm_state.assertion_state.lock().unwrap().current_before = restore_clone;
        }
        let return_type = &method.desc().return_type;
        Ok(match return_type {
            CompressedParsedDescriptorType::VoidType => None,
            return_type => {
                Some(NativeJavaValue { as_u64: function_res }.to_new_java_value(&return_type, jvm))
            }
        })
    } else {
        todo!()
        // run_function_interpreted(&jvm, interpreter_state)
    }
}


pub fn safepoint_check(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Result<(), WasException> {
    let thread = interpreter_state.thread().clone();
    let safe_point = thread.safepoint_state.borrow();
    safe_point.check(jvm, interpreter_state)
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
        jdwp.breakpoint(jvm, methodid, pc.0 as i64, interpreter_state);
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