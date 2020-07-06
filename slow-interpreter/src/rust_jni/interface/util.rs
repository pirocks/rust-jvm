use std::sync::Arc;

use classfile_view::view::ptype_view::ReferenceTypeView;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::check_inited_class;
use crate::java::lang::class::JClass;
use crate::runtime_class::RuntimeClass;

pub fn class_object_to_runtime_class<'l>(obj: &JClass, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> Option<Arc<RuntimeClass>> {
    if obj.as_type().is_primitive() {
        return None;
    }
    //todo needs to be reimplemented when loaded class sett is fixed.
    match obj.as_type().unwrap_ref_type() {
        ReferenceTypeView::Class(class_name) => {
            check_inited_class(jvm, int_state, &class_name.clone().into(), int_state.current_loader(jvm)).into()//todo a better way?
        }
        ReferenceTypeView::Array(_) => {
            None
        }
    }
}