use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::{InterpreterStateGuard, JavaValueCommon, JVMState, NewAsObjectOrJavaValue, NewJavaValue};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::call_vmentry;
use crate::interpreter::{run_function, WasException};
use crate::jit::MethodResolver;
use crate::new_java_values::NewJavaValueHandle;
use runtime_class_stuff::RuntimeClass;
use crate::stack_entry::StackEntryPush;

// todo this doesn't handle sig poly
pub fn run_invoke_static<'gc, 'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, ref_type: CPRefType, expected_method_name: MethodName, expected_descriptor: &CMethodDescriptor) {
    //todo handle monitor enter and exit
    //handle init cases
    //todo  spec says where check_ is allowed. need to match that
    let target_class = match check_initing_or_inited_class(jvm, int_state, ref_type.to_cpdtype()) {
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
        todo!()
    );
}

pub fn invoke_static_impl<'l, 'gc>(
    jvm: &'gc JVMState<'gc>,
    interpreter_state: &'_ mut InterpreterStateGuard<'gc,'l>,
    expected_descriptor: &CMethodDescriptor,
    target_class: Arc<RuntimeClass<'gc>>,
    target_method_i: u16,
    target_method: &MethodView,
    args: Vec<NewJavaValue<'gc,'_>>
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let target_class_view = target_class.view();
    if target_class_view.method_view_i(target_method_i).is_signature_polymorphic() {
        let method_view = target_class_view.method_view_i(target_method_i);
        let name = method_view.name();
        if name == MethodName::method_linkToStatic() {
            let current_frame = interpreter_state.current_frame();
            let op_stack = current_frame.operand_stack(jvm);
            let member_name = op_stack.get((op_stack.len() - 1) as u16, CClassName::member_name().into()).to_jv().cast_member_name();
            assert_eq!(member_name.clone().java_value().to_type(), CClassName::member_name().into());
            interpreter_state.pop_current_operand_stack(Some(CClassName::object().into())); //todo am I sure this is an object
            let res = call_vmentry(jvm, interpreter_state, member_name)?;
            // let _member_name = interpreter_state.pop_current_operand_stack();
            interpreter_state.push_current_operand_stack(res);
            Ok(todo!())
        } else {
            unimplemented!()
        }
    } else if !target_method.is_native() {
        assert!(target_method.is_static());
        assert!(!target_method.is_abstract());
        let max_locals = target_method.code_attribute().unwrap().max_locals;
        let method_id = jvm.method_table.write().unwrap().get_method_id(target_class.clone(), target_method_i);
        jvm.java_vm_state.add_method_if_needed(jvm, &MethodResolver{ jvm, loader: interpreter_state.current_loader(jvm) }, method_id);
        let next_entry = StackEntryPush::new_java_frame(jvm, target_class, target_method_i as u16, args);
        let mut function_call_frame = interpreter_state.push_frame(next_entry);
        match run_function(jvm, interpreter_state, &mut function_call_frame) {
            Ok(res) => {
                interpreter_state.pop_frame(jvm, function_call_frame, false);
                return Ok(res);
                panic!()
            }
            Err(_) => {
                interpreter_state.pop_frame(jvm, function_call_frame, true);
                return Err(WasException);
            }
        }
    } else {
        match run_native_method(jvm, interpreter_state, target_class, target_method_i, args) {
            Ok(res) => {
                return Ok(res)
            },
            Err(_) => todo!(),
        }
    }
}