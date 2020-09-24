use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use descriptor_parser::MethodDescriptor;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::setup_virtual_args;
use crate::interpreter::run_function;
use crate::interpreter_util::check_inited_class;
use crate::runtime_class::RuntimeClass;

pub fn invoke_special<'l>(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) -> () {
    let loader_arc = int_state.current_frame_mut().class_pointer().loader(jvm).clone();
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, int_state.current_frame_mut().class_pointer().view());
    let method_class_name = method_class_type.unwrap_class_type();
    let target_class = check_inited_class(
        jvm,
        int_state,
        &method_class_name.into(),
        loader_arc.clone(),
    );
    let (target_m_i, final_target_class) = find_target_method(jvm, loader_arc.clone(), method_name.clone(), &parsed_descriptor, target_class);
    let target_m = &final_target_class.view().method_view_i(target_m_i);
    invoke_special_impl(jvm, int_state, &parsed_descriptor, target_m_i, final_target_class.clone(), target_m);
}

pub fn invoke_special_impl<'l>(
    jvm: &JVMState,
    interpreter_state: &mut InterpreterStateGuard,
    parsed_descriptor: &MethodDescriptor,
    target_m_i: usize,
    final_target_class: Arc<RuntimeClass>,
    target_m: &MethodView,
) -> () {
    if target_m.is_native() {
        run_native_method(jvm, interpreter_state, final_target_class, target_m_i);
    } else {
        let mut args = vec![];
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args(interpreter_state.current_frame_mut(), &parsed_descriptor, &mut args, max_locals);
        assert!(args[0].unwrap_object().is_some());
        let next_entry = StackEntry::new_java_frame(final_target_class, target_m_i as u16, args);
        let function_call_frame = interpreter_state.push_frame(next_entry);
        run_function(jvm, interpreter_state);
        interpreter_state.pop_frame(function_call_frame);
        if interpreter_state.throw().is_some() || *interpreter_state.terminate() {
            return;
        }
        let function_return = interpreter_state.function_return_mut();
        if *function_return {
            *function_return = false;
        }
    }
}
