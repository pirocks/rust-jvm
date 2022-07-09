use std::sync::Arc;
use itertools::Itertools;
use another_jit_vm_ir::WasException;

use classfile_view::view::{HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};

use crate::{InterpreterStateGuard, JavaValueCommon, JVMState, NewAsObjectOrJavaValue, NewJavaValue};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::native::run_native_method;
use crate::instructions::invoke::virtual_::{call_vmentry, fixup_args};
use crate::interpreter::{PostInstructionAction, run_function};
use crate::new_java_values::NewJavaValueHandle;
use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::code::CompressedCode;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stack_entry::StackEntryPush;

// todo this doesn't handle sig poly
pub fn run_invoke_static<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l,'k>,
    method: &MethodView,
    code: &CompressedCode,
    ref_type: CPRefType,
    expected_method_name: MethodName,
    expected_descriptor: &CMethodDescriptor,
) -> PostInstructionAction<'gc> {
    //todo handle monitor enter and exit
    //handle init cases
    //todo  spec says where check_ is allowed. need to match that
    let target_class = match check_initing_or_inited_class(jvm, int_state.inner(), ref_type.to_cpdtype()) {
        Ok(x) => x,
        Err(exception) => return PostInstructionAction::Exception { exception },
    };
    let (target_method_i, final_target_method) = find_target_method(jvm, int_state.inner(), expected_method_name, &expected_descriptor, target_class);

    let view = final_target_method.view();
    let target_method = view.method_view_i(target_method_i);

    let mut args = vec![];
    let max_locals = target_method.local_var_slots();
    for _ in 0..max_locals {
        args.push(NewJavaValueHandle::Top);
    }
    let mut i = 0;
    /*
    for ptype in expected_descriptor.arg_types.iter().rev() {
        let popped = int_state.current_frame_mut().pop(ptype.to_runtime_type().unwrap()).to_new_java_handle(jvm);
        match &popped {
            NewJavaValueHandle::Long(_) | NewJavaValueHandle::Double(_) => i += 1,
            _ => {}
        }
        args[i] = popped;
        i += 1;
    }
    args[0..i].reverse();
*/
    // dbg!(expected_method_name.0.to_str(&jvm.string_pool));
    // dbg!(int_state.current_stack_depth_from_start);
    // int_state.inner().debug_print_stack_trace(jvm);
    // if method.classview().name().unwrap_name() == CClassName::string(){
    //     dump_frame(int_state, method, code);
    //     // eprintln!("{}",instruct.better_debug_string(&jvm.string_pool));
    // }
    for ptype in expected_descriptor.arg_types.iter().rev() {
        let popped = int_state.current_frame_mut().pop(ptype.to_runtime_type().unwrap()).to_new_java_handle(jvm);
        args[i] = popped;
        i += 1;
    }

    args[0..i].reverse();
    let res = invoke_static_impl(
        jvm,
        int_state.inner(),
        &expected_descriptor,
        final_target_method.clone(),
        target_method_i,
        &final_target_method.view().method_view_i(target_method_i),
        args.iter().map(|handle|handle.as_njv()).collect_vec(),
    );
    match res {
        Ok(Some(res)) => {
            int_state.current_frame_mut().push(res.to_interpreter_jv());
        }
        Ok(None) => {

        }
        Err(WasException{}) => {
            return PostInstructionAction::Exception { exception: WasException{} }
        }
    }
    PostInstructionAction::Next {}
}

pub fn invoke_static_impl<'l, 'gc>(
    jvm: &'gc JVMState<'gc>,
    interpreter_state: &'_ mut InterpreterStateGuard<'gc, 'l>,
    expected_descriptor: &CMethodDescriptor,
    target_class: Arc<RuntimeClass<'gc>>,
    target_method_i: u16,
    target_method: &MethodView,
    args: Vec<NewJavaValue<'gc, '_>>,
) -> Result<Option<NewJavaValueHandle<'gc>>, WasException> {
    let target_class_view = target_class.view();
    let method_id = jvm.method_table.write().unwrap().get_method_id(target_class.clone(), target_method_i);
    if target_class_view.method_view_i(target_method_i).is_signature_polymorphic() {
        let method_view = target_class_view.method_view_i(target_method_i);
        let name = method_view.name();
        if name == MethodName::method_linkToStatic() {
            let current_frame = interpreter_state.current_frame();
            let op_stack = current_frame.operand_stack(jvm);
            let member_name = op_stack.get((op_stack.len() - 1) as u16, CClassName::member_name().into()).cast_member_name();
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
        let args = fixup_args(args,max_locals);
        let next_entry = StackEntryPush::new_java_frame(jvm, target_class, target_method_i as u16, args);
        let function_call_frame = interpreter_state.push_frame(next_entry);
        match run_function(jvm, interpreter_state) {
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
        return match run_native_method(jvm, interpreter_state, target_class, target_method_i, args) {
            Ok(res) => {
                Ok(res)
            }
            Err(WasException {}) => {
                assert!(interpreter_state.throw().is_some());
                Err(WasException {})
            },
        }
    }
}