use crate::interpreter_util::check_inited_class;

use rust_jvm_common::classfile::InvokeInterface;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::instructions::invoke::virtual_::{setup_virtual_args, invoke_virtual_method_i};
use crate::instructions::invoke::find_target_method;
use crate::{JVMState, StackEntry};

pub fn invoke_interface(jvm: &'static JVMState, current_frame: &mut StackEntry, invoke_interface: InvokeInterface) {
    invoke_interface.count;
    let view = &current_frame.class_pointer.view();
    let loader_arc = &current_frame.class_pointer.loader(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(invoke_interface.index as usize, &view);
    let class_name_ = class_name_type.unwrap_class_type();
    let _target_class = check_inited_class(jvm, &class_name_.into(), loader_arc.clone());
    let mut args = vec![];
    let checkpoint = current_frame.operand_stack.clone();
    setup_virtual_args(current_frame, &expected_descriptor, &mut args, expected_descriptor.parameter_types.len() as u16 + 1);
    let this_pointer_o = args[0].unwrap_object().unwrap();
    let this_pointer = this_pointer_o.unwrap_normal_object();
    current_frame.operand_stack= checkpoint;
    let target_class = this_pointer.class_pointer.clone();
    let (target_method_i, final_target_class) = find_target_method(jvm, loader_arc.clone(), expected_method_name.clone(), &expected_descriptor, target_class);

    invoke_virtual_method_i(jvm, expected_descriptor, final_target_class.clone(), target_method_i, &final_target_class.view().method_view_i(target_method_i), false);
}
