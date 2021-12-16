use by_address::ByAddress;

use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::loading::LoaderName;

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::check_loaded_class_force_loader;
use crate::interpreter::WasException;
use crate::java_values::GcManagedObject;

pub fn get_or_create_class_object(jvm: &'gc_life JVMState<'gc_life>, type_: CPDType, int_state: &'_ mut InterpreterStateGuard<'gc_life>) -> Result<GcManagedObject<'gc_life>, WasException> {
    get_or_create_class_object_force_loader(jvm, type_, int_state, int_state.current_loader(jvm))
}

pub fn get_or_create_class_object_force_loader(jvm: &'gc_life JVMState<'gc_life>, type_: CPDType, int_state: &'_ mut InterpreterStateGuard<'gc_life>, loader: LoaderName) -> Result<GcManagedObject<'gc_life>, WasException> {
    let arc = check_loaded_class_force_loader(jvm, int_state, &type_, loader)?;
    Ok(jvm.classes.read().unwrap().get_class_obj_from_runtime_class(arc.clone()))
}