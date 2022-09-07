use std::num::NonZeroU8;

use itertools::Itertools;


use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValueCommon, JVMState, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::new_java_values::NewJavaValueHandle;

pub fn invoke_interface<'gc, 'l, 'k>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>, cpreftype: CPRefType, expected_method_name: MethodName, expected_descriptor: &CMethodDescriptor, count: NonZeroU8) -> PostInstructionAction<'gc> {
    // invoke_interface.count;//todo use this?
    let _target_class = check_initing_or_inited_class(jvm, int_state.inner(), cpreftype.to_cpdtype());
    let desc_len = expected_descriptor.arg_types.len();
    // assert_eq!(desc_len + 1, count.get() as usize);
    let current_frame = int_state.current_frame_mut();
    // let operand_stack_ref = current_frame.operand_stack(jvm);
    // let operand_stack_len = operand_stack_ref.len();
    // let this_pointer_jv: NewJavaValueHandle<'gc> = operand_stack_ref.get(operand_stack_len - (desc_len + 1)/*count.get()*/ as u16, RuntimeType::object());
    // let this_pointer_o = this_pointer_jv.as_njv().unwrap_object().unwrap(); //todo handle npe
    let mut args = vec![];
    for _ in 0..(expected_descriptor.arg_types.len() + 1) {//todo dupe
        args.push(NewJavaValueHandle::Top)
    }
    let mut i = 1;
    for ptype in expected_descriptor.arg_types.iter().rev() {
        let popped = int_state.current_frame_mut().pop(ptype.to_runtime_type().unwrap()).to_new_java_handle(jvm);
        args[i] = popped;
        i += 1;
    }
    args[1..i].reverse();
    args[0] = int_state.current_frame_mut().pop(RuntimeType::object()).to_new_java_handle(jvm);
    let base_object_class = args[0].as_njv().unwrap_normal_object().unwrap().runtime_class(jvm);

    let (target_method_i, final_target_class) = find_target_method(jvm, int_state.inner(), expected_method_name, &expected_descriptor, base_object_class);

    match invoke_virtual_method_i(jvm, int_state.inner(), expected_descriptor, final_target_class.clone(), &final_target_class.view().method_view_i(target_method_i), args.iter().map(|njv| njv.as_njv()).collect_vec()) {
        Ok(Some(res)) => {
            int_state.current_frame_mut().push(res.to_interpreter_jv());
        }
        Ok(None) => {
            assert!(expected_descriptor.return_type.is_void());
        }
        Err(WasException { exception_obj }) => {
            return PostInstructionAction::Exception { exception: WasException { exception_obj } };
        }
    }
    PostInstructionAction::Next {}
}