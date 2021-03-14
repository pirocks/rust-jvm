pub mod protection_domain {
    use std::sync::Arc;

    use crate::java_values::{JavaValue, Object};

    pub struct ProtectionDomain {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_protection_domain(&self) -> ProtectionDomain {
            ProtectionDomain { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl ProtectionDomain {
        as_object_or_java_value!();
    }
}

pub mod access_control_context {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java::security::protection_domain::ProtectionDomain;
    use crate::java_values::{JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct AccessControlContext {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_access_control_context(&self) -> AccessControlContext {
            AccessControlContext { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl AccessControlContext {
        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, protection_domains: Vec<ProtectionDomain>) -> Result<Self, WasException> {
            let access_control_context_class = assert_inited_or_initing_class(jvm, int_state, ClassName::Str("java/security/AccessControlContext".to_string()).into());
            push_new_object(jvm, int_state, &access_control_context_class);
            let access_control_object = int_state.pop_current_operand_stack();
            let pds_jv = JavaValue::new_vec_from_vec(jvm, protection_domains.into_iter().map(|pd| pd.java_value()).collect(), ClassName::new("java/security/ProtectionDomain").into());
            run_constructor(jvm, int_state, access_control_context_class, vec![access_control_object.clone(), pds_jv],
                            "([Ljava/security/ProtectionDomain;)V".to_string())?;
            Ok(access_control_object.cast_access_control_context())
        }

        as_object_or_java_value!();
    }
}