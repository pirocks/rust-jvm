use rust_jvm_common::classfile::InvokeInterface;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::virtual_::{invoke_virtual_method_i, setup_virtual_args};

pub fn invoke_interface(jvm: &JVMState, int_state: &mut InterpreterStateGuard, invoke_interface: InvokeInterface) {
    // invoke_interface.count;//todo use this?
    let view = &int_state.current_class_view();
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(invoke_interface.index as usize, &**view);
    let class_name_ = class_name_type.unwrap_class_type();
    let _target_class = check_initing_or_inited_class(jvm, int_state, class_name_.into());
    let mut args = vec![];
    let checkpoint = int_state.current_frame().operand_stack().clone();
    setup_virtual_args(int_state, &expected_descriptor, &mut args, expected_descriptor.parameter_types.len() as u16 + 10);//todo fix that jankyness with 10
    let this_pointer_o = args[0].unwrap_object().unwrap();//todo handle npe
    let this_pointer = this_pointer_o.unwrap_normal_object();
    *int_state.current_frame_mut().operand_stack_mut() = checkpoint;
    let target_class = this_pointer.class_pointer.clone();
    let (target_method_i, final_target_class) = find_target_method(jvm, int_state, expected_method_name, &expected_descriptor, target_class);

    let _ = invoke_virtual_method_i(jvm, int_state, expected_descriptor, final_target_class.clone(), &final_target_class.view().method_view_i(target_method_i));
}
