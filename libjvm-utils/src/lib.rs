use rust_jvm_common::ptype::PType;
use std::rc::Rc;
use std::sync::Arc;

use jni_bindings::jstring;
use classfile_view::view::ptype_view::PTypeView;
use slow_interpreter::{JVMState, StackEntry};
use slow_interpreter::java_values::Object;
use slow_interpreter::utils::string_obj_to_string;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::rust_jni::native_util::from_object;

pub fn ptype_to_class_object(state: & JVMState, frame: &Rc<StackEntry>, ptype: &PType) -> Option<Arc<Object>> {
    // dbg!(ptype);
    load_class_constant_by_type(state, frame, &PTypeView::from_ptype(ptype));
    let res = frame.pop().unwrap_object();
    // dbg!(&res);
    res
}

pub unsafe fn jstring_to_string(js: jstring) -> String{
    let str_obj = from_object(js);
    string_obj_to_string(str_obj)
}
