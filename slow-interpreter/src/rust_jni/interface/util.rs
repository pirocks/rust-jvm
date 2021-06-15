use std::sync::Arc;

use classfile_view::view::ptype_view::ReferenceTypeView;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_loaded_class;
use crate::java::lang::class::JClass;
use crate::runtime_class::RuntimeClass;

//todo move util stuff like varargs into here


pub fn class_object_to_runtime_class<'l, 'k : 'l, 'gc_life>(obj: &JClass<'gc_life>, jvm: &'gc_life JVMState<'gc_life>, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>) -> Option<Arc<RuntimeClass<'gc_life>>> {
    if obj.as_type(jvm).is_primitive() {
        return None;
    }
    //todo needs to be reimplemented when loaded class set is fixed.
    match obj.as_type(jvm).unwrap_ref_type() {
        ReferenceTypeView::Class(class_name) => {
            assert_loaded_class(jvm, class_name.clone().into()).into()//todo a better way?
        }
        ReferenceTypeView::Array(_) => {
            None
        }
    }
}