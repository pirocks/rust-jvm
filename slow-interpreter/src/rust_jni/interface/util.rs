use std::sync::Arc;
use crate::rust_jni::native_util::from_object;
use jvmti_jni_bindings::jclass;

use crate::interpreter_util::check_inited_class;
use classfile_view::view::ptype_view::ReferenceTypeView;
use crate::runtime_class::RuntimeClass;
use crate::{JVMState, StackEntry};
use crate::java_values::NormalObject;

pub struct FieldID {
    pub class: Arc<RuntimeClass>,
    pub field_i: usize,
}


pub unsafe fn runtime_class_from_object(cls: jclass, state: & JVMState, frame: &StackEntry) -> Option<Arc<RuntimeClass>> {
    let object_non_null = from_object(cls).unwrap().clone();
    let runtime_class = class_object_to_runtime_class(object_non_null.unwrap_normal_object(), state, frame);
    runtime_class.clone()
}

pub fn class_object_to_runtime_class(obj: &NormalObject, state: & JVMState, frame: &StackEntry) -> Option<Arc<RuntimeClass>> {
    if obj.class_object_to_ptype().is_primitive() {
        return None;
    }
    match obj.class_object_to_ptype().unwrap_ref_type() {
        ReferenceTypeView::Class(class_name) => {
            check_inited_class(state, &class_name,  frame.class_pointer.loader.clone()).into()//todo a better way?
        }
        ReferenceTypeView::Array(_) => {
            None
        }
    }
}