use std::sync::Arc;

use descriptor_parser::MethodDescriptor;
use rust_jvm_common::classfile::{ACC_ABSTRACT, ACC_NATIVE, ACC_STATIC, MethodInfo};
use rust_jvm_common::classnames::ClassName;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::assert_inited_or_initing_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::call_vmentry;
use crate::interpreter::run_function;
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;

// todo this doesn't handle sig poly
pub fn run_invoke_static(jvm: &JVMState, int_state: &mut InterpreterStateGuard, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let view = int_state.current_class_view();
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &view);
    let class_name = class_name_type.unwrap_class_type();
    let target_class = assert_inited_or_initing_class(
        jvm,
        int_state,
        class_name.into(),
    );
    let (target_method_i, final_target_method) = find_target_method(jvm, int_state, expected_method_name, &expected_descriptor, target_class);

    invoke_static_impl(
        jvm,
        int_state,
        expected_descriptor,
        final_target_method.clone(),
        target_method_i,
        &final_target_method.view().method_view_i(target_method_i).method_info(),
    );
}

pub fn invoke_static_impl(
    jvm: &JVMState,
    interpreter_state: &mut InterpreterStateGuard,
    expected_descriptor: MethodDescriptor,
    target_class: Arc<RuntimeClass>,
    target_method_i: usize,
    target_method: &MethodInfo,
) {
    let mut args = vec![];
    let current_frame = interpreter_state.current_frame_mut();
    if target_class.view().method_view_i(target_method_i).is_signature_polymorphic() {
        let method_view = target_class.view().method_view_i(target_method_i);
        let name = method_view.name();
        if name == "linkToStatic" {
            let op_stack = interpreter_state.current_frame().operand_stack();
            // dbg!(interpreter_state.current_frame().operand_stack_types());
            let member_name = op_stack[op_stack.len() - 1].cast_member_name();
            assert_eq!(member_name.clone().java_value().to_type(), ClassName::member_name().into());
            interpreter_state.pop_current_operand_stack();
            let res = call_vmentry(jvm, interpreter_state, member_name);
            // let _member_name = interpreter_state.pop_current_operand_stack();
            interpreter_state.push_current_operand_stack(res);
        } else {
            unimplemented!()
        }
    } else if target_method.access_flags & ACC_NATIVE == 0 {
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
        let next_entry = StackEntry::new_java_frame(jvm, target_class, target_method_i as u16, args);
        let function_call_frame = interpreter_state.push_frame(next_entry);
        run_function(jvm, interpreter_state);
        interpreter_state.pop_frame(function_call_frame);
        if interpreter_state.throw().is_some() || *interpreter_state.terminate() {
            return;
        }
        let function_return = interpreter_state.function_return_mut();
        if *function_return {
            *function_return = false;
            return;
        }
    } else {
        run_native_method(jvm, interpreter_state, target_class, target_method_i);
    }
}
