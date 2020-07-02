use crate::interpreter_util::check_inited_class;

use crate::instructions::invoke::virtual_::setup_virtual_args;
use crate::instructions::invoke::find_target_method;

use verification::verifier::instructions::branches::get_method_descriptor;

use std::sync::Arc;
use crate::instructions::invoke::native::run_native_method;
use classfile_view::view::{HasAccessFlags};
use crate::{JVMState, StackEntry};
use crate::runtime_class::RuntimeClass;
use descriptor_parser::MethodDescriptor;
use crate::interpreter::run_function;
use classfile_view::view::method_view::MethodView;

pub fn invoke_special(jvm: &'static JVMState, current_frame: &mut StackEntry, cp: u16) -> () {
    let loader_arc = current_frame.class_pointer.loader(jvm).clone();
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, current_frame.class_pointer.view());
    let method_class_name = method_class_type.unwrap_class_type();
    let target_class = check_inited_class(
        jvm,
        &method_class_name.into(),
        loader_arc.clone()
    );
    let (target_m_i, final_target_class) = find_target_method(jvm, loader_arc.clone(), method_name.clone(), &parsed_descriptor, target_class);
    let target_m = &final_target_class.view().method_view_i(target_m_i);
    invoke_special_impl(jvm, current_frame, &parsed_descriptor, target_m_i, final_target_class.clone(), target_m);
}

pub fn invoke_special_impl(
    jvm: &'static JVMState,
    current_frame: &mut StackEntry,
    parsed_descriptor: &MethodDescriptor,
    target_m_i: usize,
    final_target_class: Arc<RuntimeClass>,
    target_m: &MethodView,
) -> () {
    if target_m.is_native() {
        run_native_method(jvm, current_frame, final_target_class, target_m_i, false);
    } else {
        let mut args = vec![];
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args(current_frame, &parsed_descriptor, &mut args, max_locals);
        let next_entry = StackEntry {
            class_pointer: final_target_class.clone(),
            method_i: target_m_i as u16,
            local_vars: args,
            operand_stack: vec![],
            pc: 0,
            pc_offset: 0,
        };
        let mut current_thread = jvm.thread_state.get_current_thread();
        current_thread.call_stack.write().unwrap().push(next_entry);
        run_function(jvm,&mut current_thread);
        current_thread.call_stack.write().unwrap().pop();
        let interpreter_state = &current_thread.interpreter_state;
        if interpreter_state.throw.read().unwrap().is_some() || *interpreter_state.terminate.read().unwrap() {
            return;
        }
        let mut function_return = interpreter_state.function_return.write().unwrap();
        if *function_return {
            *function_return = false;
        }
    }
}
