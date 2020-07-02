
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::instructions::invoke::find_target_method;
use crate::interpreter_util::check_inited_class;
use std::sync::Arc;
use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC, ACC_ABSTRACT, MethodInfo};
use crate::instructions::invoke::native::run_native_method;
use crate::java_values::JavaValue;
use crate::{JVMState, StackEntry};
use crate::runtime_class::RuntimeClass;
use descriptor_parser::MethodDescriptor;
use crate::interpreter::run_function;

pub fn run_invoke_static(jvm: &'static JVMState, current_frame: &StackEntry, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let view = &current_frame.class_pointer.view();
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &view);
    let class_name = class_name_type.unwrap_class_type();
    let target_class = check_inited_class(
        jvm,
        &class_name.into(),
        loader_arc.clone()
    );
    let (target_method_i, final_target_method) = find_target_method(jvm, loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_static_impl(
        jvm,
        expected_descriptor,
        final_target_method.clone(),
        target_method_i,
        &final_target_method.view().method_view_i(target_method_i).method_info()
    );
}

pub fn invoke_static_impl(
    jvm: &'static JVMState,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    let mut args = vec![];
    let current_thread = jvm.thread_state.get_current_thread();
    let mut frames_guard = current_thread.get_frames_mut();
    let current_frame = frames_guard.last_mut().unwrap();
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
        let next_entry = StackEntry {
            class_pointer: target_class,
            method_i: target_method_i as u16,
            local_vars: args.clone(),
            operand_stack: vec![],
            pc: 0,
            pc_offset: 0,
        };
        current_thread.call_stack.write().unwrap().push(next_entry);
        run_function(jvm,&current_thread);
        current_thread.call_stack.write().unwrap().pop();
        let interpreter_state = &current_thread.interpreter_state;
        if interpreter_state.throw.read().unwrap().is_some() || *interpreter_state.terminate.read().unwrap() {
            return;
        }
        let mut function_return = interpreter_state.function_return.write().unwrap();
        if *function_return {
            *function_return = false;
            return;
        }
    } else {
        run_native_method(jvm, current_frame, target_class.clone(), target_method_i, false);
    }
}
