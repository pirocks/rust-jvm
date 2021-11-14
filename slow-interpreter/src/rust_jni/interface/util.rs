use std::sync::Arc;

use rust_jvm_common::compressed_classfile::CPRefType;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_loaded_class;
use crate::java::lang::class::JClass;
use crate::runtime_class::RuntimeClass;

//todo move util stuff like varargs into here

pub fn class_object_to_runtime_class(obj: &JClass<'gc_life>, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>) -> Option<Arc<RuntimeClass<'gc_life>>> {
    if obj.as_type(jvm).is_primitive() {
        return None;
    }
    //todo needs to be reimplemented when loaded class set is fixed.
    match obj.as_type(jvm).unwrap_ref_type() {
        CPRefType::Class(class_name) => {
            assert_loaded_class(jvm, class_name.clone().into()).into() //todo a better way?
        }
        CPRefType::Array(_) => None,
    }
}