use std::num::NonZeroU8;

use itertools::Itertools;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPRefType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use rust_jvm_common::runtime_type::RuntimeType;

use crate::{JavaValueCommon, JVMState, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter::common::invoke::find_target_method;
use crate::interpreter::common::invoke::virtual_::invoke_virtual_method_i;
use crate::interpreter::PostInstructionAction;
use crate::interpreter::real_interpreter_state::RealInterpreterStateGuard;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::null_pointer_exception::NullPointerException;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub fn invoke_interface<'gc, 'l, 'k>(
    jvm: &'gc JVMState<'gc>,
    int_state: &'_ mut RealInterpreterStateGuard<'gc, 'l, 'k>,
    cpreftype: CPRefType,
    expected_method_name: MethodName,
    expected_descriptor: &CMethodDescriptor,
    _count: NonZeroU8
) -> PostInstructionAction<'gc> {
    // invoke_interface.count;//todo use this?
    let _target_class = check_initing_or_inited_class(jvm, int_state.inner(), cpreftype.to_cpdtype());
    // assert_eq!(desc_len + 1, count.get() as usize);
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
    let base_object_class = match args[0].as_njv().unwrap_normal_object() {
        Some(x) => x,
        None => {
            let npe = NullPointerException::new(jvm, int_state.inner()).expect("exception creating exception");
            return PostInstructionAction::Exception { exception: WasException { exception_obj: npe.object().cast_throwable() } }
        },
    }.runtime_class(jvm);

    let (target_method_i, final_target_class) = find_target_method(jvm, expected_method_name, &expected_descriptor, base_object_class);

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