use rust_jvm_common::ptype::{PType, ReferenceType};
use std::rc::Rc;
use std::ops::Deref;
use std::sync::Arc;

use jni_bindings::jstring;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use slow_interpreter::{InterpreterState, StackEntry, array_of_type_class};
use slow_interpreter::java_values::{Object, JavaValue};
use slow_interpreter::utils::string_obj_to_string;
use slow_interpreter::instructions::ldc::load_class_constant_by_type;
use slow_interpreter::rust_jni::native_util::from_object;

pub fn ptype_to_class_object(state: &mut InterpreterState,frame: &Rc<StackEntry>, ptype: &PType) -> Option<Arc<Object>> {
    match ptype {
        PType::Ref(ref_) => {
            match ref_ {
                ReferenceType::Class(cl) => {
                    //todo there' duplication here where unwrap and rewrap.
                    load_class_constant_by_type(state, frame, &PTypeView::Ref(ReferenceTypeView::Class(cl.clone())));
                }
                ReferenceType::Array(sub) => {
                    frame.push(JavaValue::Object(array_of_type_class(
                        state,
                        frame.clone(),
                        sub.deref(),
                    ).into()));
                }
            }
        }
        _ => {
            // dbg!(ptype);
            // frame.print_stack_trace();
            // unimplemented!()
        }
    }
    load_class_constant_by_type(state, frame, &PTypeView::from_ptype(ptype));
    frame.pop().unwrap_object()
}

pub unsafe fn jstring_to_string(js: jstring) -> String{
    let str_obj = from_object(js);
    string_obj_to_string(str_obj)
}
