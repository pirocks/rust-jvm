use runtime_common::java_values::{Object, JavaValue};
use rust_jvm_common::unified_types::{PType, ReferenceType};
use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use slow_interpreter::instructions::ldc::load_class_constant_by_name;
use std::ops::Deref;
use slow_interpreter::array_of_type_class;
use std::sync::Arc;
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::view::ptype_view::ReferenceTypeView;
use jni_bindings::jstring;
use slow_interpreter::rust_jni::native_util::from_object;

pub fn ptype_to_class_object(state: &mut InterpreterState,frame: &Rc<StackEntry>, ptype: &PType) -> Option<Arc<Object>> {
    match ptype {
        PType::IntType => {
            load_class_constant_by_name(state, frame, &ReferenceTypeView::Class(ClassName::Str("java/lang/Integer".to_string())));
        }
        PType::Ref(ref_) => {
            match ref_ {
                ReferenceType::Class(cl) => {
                    load_class_constant_by_name(state, frame, &ReferenceTypeView::Class(cl.clone()));
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
        PType::BooleanType => {
            //todo dup.
            load_class_constant_by_name(state, frame, &ReferenceTypeView::Class(ClassName::Str("java/lang/Boolean".to_string())));
        }
        PType::LongType => {
            //todo dup.
            load_class_constant_by_name(state, frame, &ReferenceTypeView::Class(ClassName::Str("java/lang/Long".to_string())));
        }
        PType::CharType => {
            load_class_constant_by_name(state, frame, &ReferenceTypeView::Class(ClassName::Str("java/lang/Character".to_string())));
        }
        PType::FloatType => {
            //todo there really needs to be a unified function for this
            load_class_constant_by_name(state, frame, &ReferenceTypeView::Class(ClassName::Str("java/lang/Float".to_string())));
        }
        _ => {
            dbg!(ptype);
            frame.print_stack_trace();
            unimplemented!()
        }
    }
    frame.pop().unwrap_object()
}

pub unsafe fn jstring_to_string(js: jstring) -> String{
    let str_obj = from_object(js).unwrap();
    let str_fields = str_obj.unwrap_normal_object().fields.borrow();
    let char_object = str_fields.get("value").unwrap().unwrap_object().unwrap();
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    let mut res = String::new();
    for char_ in borrowed_elems.deref() {
        res.push(char_.unwrap_char());
    }
    res
}