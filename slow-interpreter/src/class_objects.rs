use std::sync::Arc;

use by_address::ByAddress;

use classfile_view::loading::LoaderName;
use classfile_view::view::ptype_view::PTypeView;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_loaded_class_force_loader;
use crate::interpreter::WasException;
use crate::java_values::Object;

pub fn get_or_create_class_object<'l, 'k : 'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>,
                                                         type_: PTypeView,
                                                         int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>,
) -> Result<Arc<Object<'gc_life>>, WasException> {
    get_or_create_class_object_force_loader(jvm, type_, int_state, int_state.current_loader())
}

pub fn get_or_create_class_object_force_loader<'l, 'k : 'l, 'gc_life>(jvm: &'gc_life JVMState<'gc_life>, type_: PTypeView, int_state: &'k mut InterpreterStateGuard<'l, 'gc_life>, loader: LoaderName) -> Result<Arc<Object<'gc_life>>, WasException> {
    let arc = check_loaded_class_force_loader(jvm, int_state, &type_, loader)?;
    Ok(jvm.classes.write().unwrap().class_object_pool.get_by_right(&ByAddress(arc.clone())).unwrap().clone().0)
}
