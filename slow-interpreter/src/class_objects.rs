use by_address::ByAddress;

use classfile_view::view::ptype_view::PTypeView;
use rust_jvm_common::loading::LoaderName;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_loaded_class_force_loader;
use crate::interpreter::WasException;
use crate::java_values::{GcManagedObject, Object};

pub fn get_or_create_class_object(jvm: &'gc_life JVMState<'gc_life>,
                                  type_: PTypeView,
                                  int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>,
) -> Result<GcManagedObject<'gc_life>, WasException> {
    get_or_create_class_object_force_loader(jvm, type_, int_state, int_state.current_loader())
}

pub fn get_or_create_class_object_force_loader(jvm: &'gc_life JVMState<'gc_life>, type_: PTypeView, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, loader: LoaderName) -> Result<GcManagedObject<'gc_life>, WasException> {
    let arc = check_loaded_class_force_loader(jvm, int_state, &type_, loader)?;
    Ok(jvm.classes.write().unwrap().class_object_pool.get_by_right(&ByAddress(arc.clone())).unwrap().clone().0)
}
