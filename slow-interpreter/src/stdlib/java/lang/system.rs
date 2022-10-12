use std::marker::PhantomData;
use std::ops::Deref;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JVMState, PushableFrame};
use crate::class_loading::assert_inited_or_initing_class;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::runtime_class::static_vars;
use crate::stdlib::java::util::properties::Properties;

pub struct System<'gc> {
    phantom: PhantomData<&'gc ()>,
}

impl<'gc> System<'gc> {
    pub fn props<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Properties<'gc> {
        let system_class = assert_inited_or_initing_class(jvm, CClassName::system().into());
        let temp = static_vars(system_class.deref(), jvm);
        let prop_jv = temp.get(FieldName::field_props());
        prop_jv.unwrap_object_nonnull().cast_properties()
    }
}