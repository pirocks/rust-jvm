use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::loading::LoaderName;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_loaded_class_force_loader;
use crate::interpreter::WasException;
use crate::new_java_values::AllocatedObject;

pub fn get_or_create_class_object<'gc, 'l>(jvm: &'gc JVMState<'gc>, type_: CPDType, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>) -> Result<AllocatedObject<'gc, 'gc>, WasException> {
    get_or_create_class_object_force_loader(jvm, type_, int_state, int_state.current_loader(jvm))
}

pub fn get_or_create_class_object_force_loader<'gc, 'l>(jvm: &'gc JVMState<'gc>, type_: CPDType, int_state: &'_ mut InterpreterStateGuard<'gc, 'l>, loader: LoaderName) -> Result<AllocatedObject<'gc, 'gc>, WasException> {
    let arc = check_loaded_class_force_loader(jvm, int_state, &type_, loader)?;
    Ok(jvm.classes.read().unwrap().get_class_obj_from_runtime_class(arc.clone()))
}