use std::sync::Arc;

use classfile_view::view::ptype_view::PTypeView;

use crate::interpreter_state::InterpreterStateGuard;
use crate::jvm_state::JVMState;
use crate::runtime_class::RuntimeClass;

pub fn check_inited_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    todo!()
}

pub fn check_resolved_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    todo!()
}

pub fn assert_inited_or_initing_class(jvm: &JVMState, int_state: &mut InterpreterStateGuard, ptype: PTypeView) -> Arc<RuntimeClass> {
    todo!()
}