pub mod properties {
    use std::sync::Arc;

    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::string::JString;
    use crate::java_values::{JavaValue, Object};
    use crate::utils::run_static_or_virtual;

    pub struct Properties {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_properties(&self) -> Properties {
            let res = Properties { normal_object: self.unwrap_object_nonnull() };
            assert_eq!(res.normal_object.unwrap_normal_object().objinfo.class_pointer.view().name(), ClassName::properties().into());
            res
        }
    }

    impl Properties {
        pub fn set_property(&self, jvm: &JVMState, int_state: &mut InterpreterStateGuard, key: JString, value: JString) -> Result<(), WasException> {
            let properties_class = assert_inited_or_initing_class(jvm, ClassName::properties().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(key.java_value());
            int_state.push_current_operand_stack(value.java_value());
            run_static_or_virtual(jvm, int_state, &properties_class, "setProperty".to_string(), "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;".to_string())?;
            int_state.pop_current_operand_stack();
            Ok(())
        }
    }
}

