use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::loading::LoaderName;

use crate::{JVMState, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_loaded_class_force_loader;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

pub fn get_or_create_class_object<'gc, 'l>(jvm: &'gc JVMState<'gc>, type_: CPDType, int_state: &mut impl PushableFrame<'gc>) -> Result<AllocatedNormalObjectHandle<'gc>, WasException<'gc>> {
    get_or_create_class_object_force_loader(jvm, type_, int_state, int_state.current_loader(jvm))
}

pub fn get_or_create_class_object_force_loader<'gc, 'l>(jvm: &'gc JVMState<'gc>, type_: CPDType, int_state: &mut impl PushableFrame<'gc>, loader: LoaderName) -> Result<AllocatedNormalObjectHandle<'gc>, WasException<'gc>> {
    let arc = check_loaded_class_force_loader(jvm, int_state, &type_, loader)?;
    let handle = jvm.classes.read().unwrap().get_class_obj_from_runtime_class(arc.clone());
    assert_eq!(handle.runtime_class(jvm).cpdtype(), CClassName::class().into());
    handle.duplicate_discouraged().cast_class().debug_assert(jvm);
    Ok(handle)
}