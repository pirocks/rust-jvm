use num::one;

use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::classfile::InvokeInterface;
use rust_jvm_common::ptype::PType;
use verification::verifier::instructions::branches::get_method_descriptor;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_initing_or_inited_class;
use crate::instructions::invoke::find_target_method;
use crate::instructions::invoke::virtual_::{invoke_virtual_method_i, setup_virtual_args};
use crate::java_values::JavaValue;
use crate::stack_entry::{OperandStackRef, StackEntryRef};

pub fn invoke_interface(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, invoke_interface: InvokeInterface) {
    // invoke_interface.count;//todo use this?
    let view = &int_state.current_class_view(jvm);
    let (class_name_type, expected_method_name, expected_descriptor) = get_method_descriptor(invoke_interface.index as usize, &**view);
    let class_name_ = class_name_type.unwrap_class_type();
    let _target_class = check_initing_or_inited_class(jvm, int_state, class_name_.into());
    let desc_len = expected_descriptor.parameter_types.len();
    assert_eq!(desc_len + 1, invoke_interface.count as usize);
    let current_frame: StackEntryRef<'gc_life> = int_state.current_frame();
    let operand_stack_ref: OperandStackRef<'gc_life, '_> = current_frame.operand_stack();
    let operand_stack_len = operand_stack_ref.len();
    let this_pointer_jv: JavaValue<'gc_life> = operand_stack_ref.get(operand_stack_len - invoke_interface.count as u16, PTypeView::object());
    let this_pointer_o = this_pointer_jv.unwrap_object().unwrap();//todo handle npe
    let this_pointer = this_pointer_o.unwrap_normal_object();
    let target_class = this_pointer.objinfo.class_pointer.clone();
    let (target_method_i, final_target_class) = find_target_method(jvm, int_state, expected_method_name, &expected_descriptor, target_class);

    let _ = invoke_virtual_method_i(jvm, int_state, expected_descriptor, final_target_class.clone(), &final_target_class.view().method_view_i(target_method_i));
}
