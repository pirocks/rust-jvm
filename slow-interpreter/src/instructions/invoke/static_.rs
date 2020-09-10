use std::sync::Arc;

use descriptor_parser::MethodDescriptor;
use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_NATIVE, ACC_STATIC, MethodInfo};
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::interpreter::run_function;
use crate::interpreter_util::check_inited_class;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;

pub fn run_invoke_static<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let view = int_state.current_class_view();
    let loader_arc = int_state.current_loader(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &view);
    let class_name = class_name_type.unwrap_class_type();
    let target_class = check_inited_class(
        jvm,
        int_state,
        &class_name.into(),
        loader_arc.clone(),
    );
    let (target_method_i, final_target_method) = find_target_method(jvm, loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_static_impl(
        jvm,
        int_state,
        expected_descriptor,
        final_target_method.clone(),
        target_method_i,
        &final_target_method.view().method_view_i(target_method_i).method_info(),
    );
}

pub fn invoke_static_impl<'l>(
    jvm: &'static JVMState,
    interpreter_state: &mut InterpreterStateGuard,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    let mut args = vec![];
    let current_frame = interpreter_state.current_frame_mut();
    if target_method.access_flags & ACC_NATIVE == 0 {
        assert!(target_method.access_flags & ACC_STATIC > 0);
        assert_eq!(target_method.access_flags & ACC_ABSTRACT, 0);
        let max_locals = target_method.code_attribute().unwrap().max_locals;
        for _ in 0..max_locals {
            args.push(JavaValue::Top);
        }
        let mut i = 0;
        for _ in 0..expected_descriptor.parameter_types.len() {
            let popped = current_frame.pop();
            match &popped {
                JavaValue::Long(_) | JavaValue::Double(_) => { i += 1 }
                _ => {}
            }
            args[i] = popped;
            i += 1;
        }
        args[0..i].reverse();
        let next_entry = StackEntry::new_java_frame(target_class, target_method_i as u16, args);
        interpreter_state.push_frame(next_entry);
        run_function(jvm, interpreter_state);
        interpreter_state.pop_frame();
        if interpreter_state.throw().is_some() || *interpreter_state.terminate() {
            return;
        }
        let function_return = interpreter_state.function_return_mut();
        if *function_return {
            *function_return = false;
            return;
        }
    } else {
        run_native_method(jvm, interpreter_state, target_class.clone(), target_method_i, false);
    }
}
