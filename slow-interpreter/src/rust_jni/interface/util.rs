use std::sync::Arc;

use classfile_view::view::ptype_view::ReferenceTypeView;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::{assert_inited_or_initing_class, assert_loaded_class};
use crate::java::lang::class::JClass;
use crate::runtime_class::RuntimeClass;

pub fn class_object_to_runtime_class(obj: &JClass, jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> Option<Arc<RuntimeClass>> {
    if obj.as_type(jvm).is_primitive() {
        return None;
    }
    //todo needs to be reimplemented when loaded class sett is fixed.
    match obj.as_type(jvm).unwrap_ref_type() {
        ReferenceTypeView::Class(class_name) => {
            assert_loaded_class(jvm, int_state, class_name.clone().into()).into()//todo a better way?
        }
        ReferenceTypeView::Array(_) => {
            None
        }
    }
}