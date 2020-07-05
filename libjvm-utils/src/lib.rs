use rust_jvm_common::ptype::PType;

use std::sync::Arc;

use jvmti_jni_bindings::jstring;
use classfile_view::view::ptype_view::PTypeView;
use slow_interpreter::{JVMState, InterpreterStateGuard};
use slow_interpreter::java_values::Object;
use slow_interpreter::utils::string_obj_to_string;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::rust_jni::native_util::from_object;
use slow_interpreter::stack_entry::StackEntry;

pub fn ptype_to_class_object(state: &'static JVMState, int_state: &mut InterpreterStateGuard, ptype: &PType) -> Option<Arc<Object>> {
    // dbg!(ptype);
    load_class_constant_by_type(state, int_state, &PTypeView::from_ptype(ptype));
    let res = int_state.pop_current_operand_stack().unwrap_object();
    // dbg!(&res);
    res
}

pub unsafe fn jstring_to_string(js: jstring) -> String{
    let str_obj = from_object(js);
    string_obj_to_string(str_obj)
}
