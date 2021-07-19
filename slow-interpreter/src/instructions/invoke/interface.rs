use std::num::NonZeroU8;

use rust_jvm_common::classfile::InvokeInterface;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::descriptors::ActuallyCompressedMD;
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use rust_jvm_common::runtime_type::RuntimeType;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::virtual_::invoke_virtual_method_i;
use crate::java_values::JavaValue;

pub fn invoke_interface<'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, class_name: CClassName, expected_method_name: MethodName, expected_descriptor_original: ActuallyCompressedMD, count: NonZeroU8) {
    // invoke_interface.count;//todo use this?
    let expected_descriptor = jvm.method_descriptor_pool.lookup(expected_descriptor_original);
    let _target_class = check_initing_or_inited_class(jvm, int_state, class_name.into());
    let desc_len = expected_descriptor.arg_types.len();
    assert_eq!(desc_len + 1, count.get() as usize);
    let current_frame = int_state.current_frame();
    let operand_stack_ref = current_frame.operand_stack(jvm);
    let operand_stack_len = operand_stack_ref.len();
    let this_pointer_jv: JavaValue<'gc_life> = operand_stack_ref.get(operand_stack_len - count.get() as u16, RuntimeType::object());
    let this_pointer_o = this_pointer_jv.unwrap_object().unwrap();//todo handle npe
    let this_pointer = this_pointer_o.unwrap_normal_object();
    let target_class = this_pointer.objinfo.class_pointer.clone();
    let (target_method_i, final_target_class) = find_target_method(jvm, int_state, expected_method_name, &expected_descriptor, target_class);

    let _ = invoke_virtual_method_i(jvm, int_state, expected_descriptor_original, final_target_class.clone(), &final_target_class.view().method_view_i(target_method_i));
}
