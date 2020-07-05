pub mod properties {
    use std::sync::Arc;
    use crate::java_values::{Object, JavaValue};
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use crate::{JVMState, InterpreterStateGuard};
    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;

    pub struct Properties {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_properties(&self) -> Properties {
            let res = Properties { normal_object: self.unwrap_object_nonnull() };
            assert_eq!(res.normal_object.unwrap_normal_object().class_pointer.view().name(), ClassName::properties());
            res
        }
    }

    impl Properties {
        pub fn set_property<'l>(&self, jvm: &'static JVMState, int_state: & mut InterpreterStateGuard, key: JString, value: JString) {
            let properties_class = check_inited_class(jvm, int_state,&ClassName::properties().into(), int_state.current_loader(jvm).clone());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(key.java_value());
            int_state.push_current_operand_stack(value.java_value());
            run_static_or_virtual(jvm, int_state,&properties_class, "setProperty".to_string(), "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;".to_string());
            int_state.pop_current_operand_stack();
        }
    }
}

