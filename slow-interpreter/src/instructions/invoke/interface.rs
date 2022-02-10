use std::num::NonZeroU8;

use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::MethodName;
use rust_jvm_common::runtime_type::RuntimeType;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::java_values::JavaValue;
use crate::new_java_values::NewJavaValueHandle;

pub fn invoke_interface<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, cpreftype: CPRefType, expected_method_name: MethodName, expected_descriptor: &CMethodDescriptor, count: NonZeroU8) {
    // invoke_interface.count;//todo use this?
    let _target_class = check_initing_or_inited_class(jvm, int_state, CPDType::Ref(cpreftype));
    let desc_len = expected_descriptor.arg_types.len();
    // assert_eq!(desc_len + 1, count.get() as usize);
    let current_frame = int_state.current_frame();
    let operand_stack_ref = current_frame.operand_stack(jvm);
    let operand_stack_len = operand_stack_ref.len();
    let this_pointer_jv: NewJavaValueHandle<'gc_life> = operand_stack_ref.get(operand_stack_len - (desc_len + 1)/*count.get()*/ as u16, RuntimeType::object());
    let this_pointer_o = this_pointer_jv.as_njv().unwrap_object().unwrap(); //todo handle npe
    let this_pointer = todo!()/*this_pointer_o.unwrap_normal_object()*/;
    let target_class = todo!()/*this_pointer.objinfo.class_pointer.clone()*/;
    let (target_method_i, final_target_class) = find_target_method(jvm, int_state, expected_method_name, &expected_descriptor, target_class);

    let _ = invoke_virtual_method_i(jvm, int_state, expected_descriptor, final_target_class.clone(), &final_target_class.view().method_view_i(target_method_i), todo!());
}