use std::sync::Arc;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::CPRefType;

use crate::JVMState;
use crate::class_loading::assert_loaded_class;
use crate::stdlib::java::lang::class::JClass;

//todo move util stuff like varargs into here

pub fn class_object_to_runtime_class<'gc, 'l>(obj: &JClass<'gc>, jvm: &'gc JVMState<'gc>) -> Option<Arc<RuntimeClass<'gc>>> {
    if obj.as_type(jvm).is_primitive() {
        return None;
    }
    //todo needs to be reimplemented when loaded class set is fixed.
    match obj.as_type(jvm).unwrap_ref_type() {
        CPRefType::Class(class_name) => {
            assert_loaded_class(jvm, class_name.clone().into()).into() //todo a better way?
        }
        CPRefType::Array { .. } => None,
    }
}