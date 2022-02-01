pub mod protection_domain {
    use crate::java_values::{GcManagedObject, JavaValue};

    pub struct ProtectionDomain<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_protection_domain(&self) -> ProtectionDomain<'gc_life> {
            ProtectionDomain { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> ProtectionDomain<'gc_life> {
        as_object_or_java_value!();
    }
}

pub mod access_control_context {
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java::security::protection_domain::ProtectionDomain;
    use crate::java_values::{GcManagedObject, JavaValue};
    use crate::jvm_state::JVMState;

    pub struct AccessControlContext<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_access_control_context(&self) -> AccessControlContext<'gc_life> {
            AccessControlContext { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> AccessControlContext<'gc_life> {
        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>, protection_domains: Vec<ProtectionDomain<'gc_life>>) -> Result<Self, WasException> {
            let access_control_context_class = assert_inited_or_initing_class(jvm, CClassName::access_control_context().into());
            let access_control_object = new_object(jvm, int_state, &access_control_context_class);
            let pds_jv = JavaValue::new_vec_from_vec(jvm, protection_domains.into_iter().map(|pd| pd.java_value()).collect(), CClassName::protection_domain().into());
            run_constructor(jvm, int_state, access_control_context_class, vec![access_control_object.clone(), pds_jv], &CMethodDescriptor::void_return(vec![CPDType::array(CClassName::protection_domain().into())]))?;
            Ok(access_control_object.cast_access_control_context())
        }

        as_object_or_java_value!();
    }
}