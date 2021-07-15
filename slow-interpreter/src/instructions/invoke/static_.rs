use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::compressed_classfile::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::names::CClassName;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState, StackEntry};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::call_vmentry;
use crate::interpreter::{run_function, WasException};
use crate::java_values::JavaValue;
use crate::runtime_class::RuntimeClass;

// todo this doesn't handle sig poly
pub fn run_invoke_static(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, cp: u16) {
//todo handle monitor enter and exit
//handle init cases
    let view = int_state.current_class_view(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(cp as usize, &*view);
    let class_name = class_name_type.unwrap_class_type();
    //todo  spec says where check_ is allowed. need to match that
    let target_class = match check_initing_or_inited_class(
        jvm,
        int_state,
        class_name.into(),
    ) {
        Ok(x) => x,
        Err(WasException {}) => return,
    };
    let (target_method_i, final_target_method) = find_target_method(jvm, int_state, expected_method_name, &expected_descriptor, target_class);

    let _ = invoke_static_impl(
        jvm,
        int_state,
        &expected_descriptor,
        final_target_method.clone(),
        target_method_i,
        &final_target_method.view().method_view_i(target_method_i),
    );
}

pub fn invoke_static_impl(
    jvm: &'gc_life JVMState<'gc_life>,
    interpreter_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
    expected_descriptor: &CMethodDescriptor,
    target_class: Arc<RuntimeClass<'gc_life>>,
    target_method_i: u16,
    target_method: &MethodView,
) -> Result<(), WasException> {
    let mut args = vec![];
    let mut current_frame = interpreter_state.current_frame_mut();
    let target_class_view = target_class.view();
    if target_class_view.method_view_i(target_method_i).is_signature_polymorphic() {
        let method_view = target_class_view.method_view_i(target_method_i);
        let name = method_view.name();
        if name == "linkToStatic" {
            let current_frame = interpreter_state.current_frame();
            let op_stack = current_frame.operand_stack(jvm);
            let member_name = op_stack.get((op_stack.len() - 1) as u16, CClassName::member_name().into()).cast_member_name();
            assert_eq!(member_name.clone().java_value().to_type(), CClassName::member_name().into());
            interpreter_state.pop_current_operand_stack(Some(CClassName::object().into()));//todo am I sure this is an object
            let res = call_vmentry(jvm, interpreter_state, member_name)?;
            // let _member_name = interpreter_state.pop_current_operand_stack();
            interpreter_state.push_current_operand_stack(res);
            Ok(())
        } else {
            unimplemented!()
        }
    } else if !target_method.is_native() {
        assert!(target_method.is_static());
        assert!(!target_method.is_abstract());
        let max_locals = target_method.code_attribute().unwrap().max_locals;
        for _ in 0..max_locals {
            args.push(JavaValue::Top);
        }
        let mut i = 0;
        for ptype in expected_descriptor.arg_types.iter().rev() {
            let popped = current_frame.pop(Some(ptype.to_runtime_type().unwrap()));
            match &popped {
                JavaValue::Long(_) | JavaValue::Double(_) => { i += 1 }
                _ => {}
            }
            args[i] = popped;
            i += 1;
        }
        args[0..i].reverse();
        let next_entry = StackEntry::new_java_frame(jvm, target_class, target_method_i as u16, args);
        let function_call_frame = interpreter_state.push_frame(next_entry, jvm);
        match run_function(jvm, interpreter_state) {
            Ok(_) => {
                interpreter_state.pop_frame(jvm, function_call_frame, false);
                if interpreter_state.function_return() {
                    interpreter_state.set_function_return(false);
                    return Ok(());
                }
                panic!()
            }
            Err(_) => {
                interpreter_state.pop_frame(jvm, function_call_frame, true);
                return Err(WasException);
            }
        }
    } else {
        run_native_method(jvm, interpreter_state, target_class, target_method_i)
    }
}
