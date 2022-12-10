use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use crate::{AllocatedHandle, NewAsObjectOrJavaValue, PushableFrame};
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::java_values::{JavaValue};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::security::protection_domain::ProtectionDomain;

pub struct AccessControlContext<'gc> {
    normal_object: AllocatedHandle<'gc>,
}

impl<'gc> AccessControlContext<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, protection_domains: Vec<ProtectionDomain<'gc>>) -> Result<Self, crate::WasException<'gc>> {
        let access_control_context_class = assert_inited_or_initing_class(jvm, CClassName::access_control_context().into());
        let access_control_object = new_object(jvm, int_state, &access_control_context_class, false);
        let pds_jv = JavaValue::new_vec_from_vec(jvm, protection_domains.iter().map(|pd| pd.new_java_value()).collect(), CClassName::protection_domain().into());
        let desc = CMethodDescriptor::void_return(vec![CPDType::array(CClassName::protection_domain().into())]);
        run_constructor(jvm, int_state, access_control_context_class, vec![access_control_object.new_java_value(), pds_jv.new_java_value()], &desc)?;
        Ok(access_control_object.cast_access_control_context())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for AccessControlContext<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object.unwrap_normal_object()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        self.normal_object.unwrap_normal_object_ref()
    }
}
