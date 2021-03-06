use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use descriptor_parser::MethodDescriptor;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::setup_virtual_args;
use crate::interpreter::{run_function, WasException};
use crate::runtime_class::RuntimeClass;

pub fn invoke_special(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, int_state.current_frame_mut().class_pointer().view());
    let method_class_name = method_class_type.unwrap_class_type();
    let target_class = check_initing_or_inited_class(
        jvm,
        int_state,
        method_class_name.into(),
    ).unwrap();//todo pass the error up
    let (target_m_i, final_target_class) = find_target_method(jvm, int_state, method_name, &parsed_descriptor, target_class);
    let _ = invoke_special_impl(jvm, int_state, &parsed_descriptor, target_m_i, final_target_class.clone());
}

pub fn invoke_special_impl(
    jvm: &JVMState,
    interpreter_state: &mut InterpreterStateGuard,
    parsed_descriptor: &MethodDescriptor,
    target_m_i: usize,
    final_target_class: Arc<RuntimeClass>,
) -> Result<(), WasException> {
    let target_m = &final_target_class.view().method_view_i(target_m_i);
    if final_target_class.view().method_view_i(target_m_i).is_signature_polymorphic() {
        interpreter_state.debug_print_stack_trace();
        dbg!(target_m.name());
        unimplemented!()
    } else if target_m.is_native() {
        run_native_method(jvm, interpreter_state, final_target_class, target_m_i)
    } else {
        let mut args = vec![];
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args(interpreter_state, &parsed_descriptor, &mut args, max_locals);
        assert!(args[0].unwrap_object().is_some());
        let next_entry = StackEntry::new_java_frame(jvm, final_target_class, target_m_i as u16, args);
        let function_call_frame = interpreter_state.push_frame(next_entry);
        match run_function(jvm, interpreter_state) {
            Ok(()) => {
                interpreter_state.pop_frame(jvm, function_call_frame, false);
                let function_return = interpreter_state.function_return_mut();
                if *function_return {
                    *function_return = false;
                }
                Ok(())
            }
            Err(WasException {}) => {
                interpreter_state.pop_frame(jvm, function_call_frame, true);
                assert!(interpreter_state.throw().is_some());
                Err(WasException)
            }
        }
    }
}
