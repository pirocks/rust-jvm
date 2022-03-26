use std::ops::Deref;
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

use crate::{InterpreterStateGuard, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java::util::properties::Properties;
use crate::java_values::{GcManagedObject};
use crate::runtime_class::static_vars;

pub struct System<'gc> {
    normal_object: GcManagedObject<'gc>,
}

impl<'gc> System<'gc> {
    pub fn props<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Properties<'gc> {
        let system_class = assert_inited_or_initing_class(jvm, CClassName::system().into());
        let temp = static_vars(system_class.deref(),jvm);
        let prop_jv = temp.get(FieldName::field_props());
        prop_jv.unwrap_object_nonnull().cast_properties()
    }

    //as_object_or_java_value!();
}