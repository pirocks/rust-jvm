use std::sync::Arc;
use crate::rust_jni::native_util::from_object;
use jvmti_jni_bindings::jclass;

use crate::interpreter_util::check_inited_class;
use classfile_view::view::ptype_view::ReferenceTypeView;
use crate::runtime_class::RuntimeClass;
use crate::{JVMState, StackEntry};
use crate::java_values::JavaValue;
use crate::java::lang::class::JClass;

pub struct FieldID {
    pub class: Arc<RuntimeClass>,
    pub field_i: usize,
}


pub unsafe fn runtime_class_from_object(cls: jclass) -> Arc<RuntimeClass> {
    let object = from_object(cls);
    JavaValue::Object(object).cast_class().as_runtime_class()
}

pub fn class_object_to_runtime_class(obj: &JClass, jvm: & JVMState, frame: &StackEntry) -> Option<Arc<RuntimeClass>> {
    if obj.as_type().is_primitive() {
        return None;
    }
    //todo needs to be reimplemented when loaded class sett is fixed.
    match obj.as_type().unwrap_ref_type() {
        ReferenceTypeView::Class(class_name) => {
            check_inited_class(jvm, &class_name, frame.class_pointer.loader(jvm).clone()).into()//todo a better way?
        }
        ReferenceTypeView::Array(_) => {
            None
        }
    }
}