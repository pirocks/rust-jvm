use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::{setup_virtual_args, setup_virtual_args2};
use crate::interpreter::{run_function, WasException};
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;

pub fn invoke_special(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, method_class_name: CClassName, method_name: MethodName, parsed_descriptor: &CMethodDescriptor) {
    let target_class = match check_initing_or_inited_class(jvm, int_state, method_class_name.into()) {
        Ok(x) => x,
        Err(WasException {}) => return,
    };
    let (target_m_i, final_target_class) = find_target_method(jvm, int_state, method_name, &parsed_descriptor, target_class);
    let _ = invoke_special_impl(jvm, int_state, &parsed_descriptor, target_m_i, final_target_class.clone(), todo!());
}

pub fn invoke_special_impl(jvm: &'gc_life JVMState<'gc_life>, interpreter_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, parsed_descriptor: &CMethodDescriptor, target_m_i: u16, final_target_class: Arc<RuntimeClass<'gc_life>>, input_args: Vec<JavaValue<'gc_life>>) -> Result<(), WasException> {
    let final_target_view = final_target_class.view();
    let target_m = &final_target_view.method_view_i(target_m_i);
    if final_target_view.method_view_i(target_m_i).is_signature_polymorphic() {
        interpreter_state.debug_print_stack_trace(jvm);
        dbg!(target_m.name());
        unimplemented!()
    } else if target_m.is_native() {
        match run_native_method(jvm, interpreter_state, final_target_class, target_m_i, input_args) {
            Ok(_) => todo!(),
            Err(_) => todo!(),
        }
    } else {
        let mut args = vec![];
        let max_locals = target_m.code_attribute().unwrap().max_locals;
        setup_virtual_args2(interpreter_state, &parsed_descriptor, &mut args, max_locals, input_args);
        assert!(args[0].unwrap_object().is_some());
        let next_entry = StackEntry::new_java_frame(jvm, final_target_class.clone(), target_m_i as u16, args);
        let function_call_frame = interpreter_state.push_frame(next_entry);
        match run_function(jvm, interpreter_state) {
            Ok(()) => {
                if !jvm.config.compiled_mode_active {
                    interpreter_state.pop_frame(jvm, function_call_frame, false);
                }
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