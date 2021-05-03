use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::loading::LoaderName;
use classfile_view::view::ptype_view::PTypeView;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_loaded_class_force_loader;
use crate::interpreter::WasException;
use crate::java_values::Object;

pub fn get_or_create_class_object(jvm: &JVMState,
                                  type_: PTypeView,
                                  int_state: &mut InterpreterStateGuard,
) -> Result<Arc<Object>, WasException> {
    get_or_create_class_object_force_loader(jvm, type_, int_state, int_state.current_loader())
}

pub fn get_or_create_class_object_force_loader(jvm: &JVMState, type_: PTypeView, int_state: &mut InterpreterStateGuard, loader: LoaderName) -> Result<Arc<Object>, WasException> {
    let arc = check_loaded_class_force_loader(jvm, int_state, &type_, loader)?;
    Ok(jvm.classes.write().unwrap().class_object_pool.get_by_right(&ByAddress(arc.clone())).unwrap().clone().0)
}
