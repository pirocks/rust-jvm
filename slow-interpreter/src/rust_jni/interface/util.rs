use runtime_common::runtime_class::RuntimeClass;
use std::sync::Arc;
use crate::rust_jni::native_util::from_object;
use jni_bindings::jclass;

pub struct FieldID {
    pub class: Arc<RuntimeClass>,
    pub field_i: usize,
}


pub unsafe fn runtime_class_from_object(cls: jclass) -> Option<Arc<RuntimeClass>> {
    let object_non_null = from_object(cls).unwrap().clone();
    let object_class = object_non_null.unwrap_normal_object().object_class_object_pointer.borrow();
    object_class.clone()
}

