use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::instructions::invoke::find_target_method;
use crate::interpreter_util::{check_inited_class, run_function};
use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;

use rust_jvm_common::classfile::{ACC_NATIVE, ACC_STATIC, ACC_ABSTRACT, MethodInfo};
use runtime_common::java_values::JavaValue;
use crate::instructions::invoke::native::run_native_method;
use classfile_view::view::ClassView;
use classfile_view::view::descriptor_parser::MethodDescriptor;

pub fn run_invoke_static(state: &mut InterpreterState, current_frame: Rc<StackEntry>, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let classfile = &current_frame.class_pointer.classfile;
    let loader_arc = &current_frame.class_pointer.loader;
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &ClassView::from(classfile.clone()));
    let class_name = class_name_type.unwrap_class_type();
    let target_class = check_inited_class(state, &class_name, current_frame.clone().into(), loader_arc.clone());
    let (target_method_i, final_target_method) = find_target_method(state, loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_static_impl(state, current_frame, expected_descriptor, final_target_method.clone(), target_method_i, &final_target_method.classfile.methods[target_method_i]);
}

pub fn invoke_static_impl(
    state: &mut InterpreterState,
    current_frame: Rc<StackEntry>,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) -> () {
    let mut args = vec![];
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
            last_call_stack: Some(current_frame),
            class_pointer: target_class,
            method_i: target_method_i as u16,
            local_vars: args.clone().into(),
            operand_stack: vec![].into(),
            pc: 0.into(),
            pc_offset: 0.into(),
        };
        run_function(state, Rc::new(next_entry));
        if state.throw.is_some() || state.terminate {
            return;
        }
        if state.function_return {
            state.function_return = false;
            return;
        }
    } else {
        run_native_method(state, current_frame.clone(), target_class.clone(), target_method_i);
    }
}
