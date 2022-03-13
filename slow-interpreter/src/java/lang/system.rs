use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java::util::properties::Properties;
use crate::java_values::{GcManagedObject, JavaValue};

pub struct System<'gc_life> {
    normal_object: GcManagedObject<'gc_life>,
}

impl<'gc_life> System<'gc_life> {
    pub fn props<'l>(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) -> Properties<'gc_life> {
        let system_class = assert_inited_or_initing_class(jvm, CClassName::system().into());
        let temp = system_class.static_vars(jvm);
        let prop_jv = temp.get(FieldName::field_props());
        prop_jv.unwrap_object_nonnull().cast_properties()
    }

    as_object_or_java_value!();
}