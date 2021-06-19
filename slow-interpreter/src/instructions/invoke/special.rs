use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::descriptor_parser::MethodDescriptor;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::setup_virtual_args;
use crate::interpreter::{run_function, WasException};
use crate::runtime_class::RuntimeClass;

pub fn invoke_special(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, cp: u16) {
    let (method_class_type, method_name, parsed_descriptor) = get_method_descriptor(cp as usize, &*int_state.current_frame().class_pointer(jvm).view());
    let method_class_name = method_class_type.unwrap_class_type();
    let target_class = match check_initing_or_inited_class(
        jvm,
        int_state,
        method_class_name.into(),
    ) {
        Ok(x) => x,
        Err(WasException {}) => return,
    };
    let (target_m_i, final_target_class) = find_target_method(jvm, int_state, method_name, &parsed_descriptor, target_class);
    let _ = invoke_special_impl(jvm, int_state, &parsed_descriptor, target_m_i, final_target_class.clone());
}

pub fn invoke_special_impl<'gc_life>(
    jvm: &'_ JVMState<'gc_life>,
    interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
    parsed_descriptor: &MethodDescriptor,
    target_m_i: u16,
    final_target_class: Arc<RuntimeClass<'gc_life>>,
) -> Result<(), WasException> {
    let final_target_view = final_target_class.view();
    let target_m = &final_target_view.method_view_i(target_m_i);
    if final_target_view.method_view_i(target_m_i).is_signature_polymorphic() {
        interpreter_state.debug_print_stack_trace(jvm);
        dbg!(target_m.name());
        unimplemented!()
    } else if target_m.is_native() {
        run_native_method(jvm, interpreter_state, final_target_class, target_m_i)
    } else {
        let mut args = vec![];
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args(interpreter_state, &parsed_descriptor, &mut args, max_locals);
        assert!(args[0].unwrap_object().is_some());
        let next_entry = StackEntry::new_java_frame(jvm, final_target_class.clone(), target_m_i as u16, args);
        let arc = final_target_class.view();
        if arc.method_view_i(target_m_i).name() == "<init>" && arc.name() == ClassName::Str("bed".to_string()).into() {
            // dbg!(arc.name());
            // interpreter_state.debug_print_stack_trace();
            // panic!();
        }
        let function_call_frame = interpreter_state.push_frame(next_entry, jvm);
        match run_function(jvm, interpreter_state) {
            Ok(()) => {
                interpreter_state.pop_frame(jvm, function_call_frame, false);
                if interpreter_state.function_return() {
                    interpreter_state.set_function_return(false);
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
