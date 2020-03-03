use runtime_common::runtime_class::RuntimeClass;
use std::sync::Arc;
use crate::rust_jni::native_util::from_object;
use jni_bindings::jclass;
use runtime_common::java_values::NormalObject;
use runtime_common::{InterpreterState, StackEntry};
use std::rc::Rc;
use crate::interpreter_util::check_inited_class;
use classfile_view::view::ptype_view::ReferenceTypeView;

pub struct FieldID {
    pub class: Arc<RuntimeClass>,
    pub field_i: usize,
}


pub unsafe fn runtime_class_from_object(cls: jclass,state: &mut InterpreterState, frame : &Rc<StackEntry>) -> Option<Arc<RuntimeClass>> {
    let object_non_null = from_object(cls).unwrap().clone();
    let object_class = class_object_to_runtime_class(object_non_null.unwrap_normal_object(),state,frame);
    object_class.clone().into()
}

pub fn class_object_to_runtime_class(obj: &NormalObject,state: &mut InterpreterState, frame : &Rc<StackEntry>) -> Arc<RuntimeClass> {
    match obj.class_object_to_ptype().unwrap_ref_type(){
        ReferenceTypeView::Class(class_name) => {
            check_inited_class(state,&class_name,frame.clone().into(),frame.class_pointer.loader.clone())//todo a better way?
        },
        ReferenceTypeView::Array(_) => {
            unimplemented!()
        },
    }

}