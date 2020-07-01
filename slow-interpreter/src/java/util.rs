pub mod properties {
    use std::sync::Arc;
    use crate::java_values::{Object, JavaValue};
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use crate::stack_entry::StackEntry;
    use crate::JVMState;
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
        pub fn set_property(&self, jvm: &'static JVMState, frame: &mut StackEntry, key: JString, value: JString) {
            let properties_class = check_inited_class(jvm, &ClassName::properties().into(), frame.class_pointer.loader(jvm).clone());
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            frame.push(key.java_value());
            frame.push(value.java_value());
            run_static_or_virtual(jvm, &properties_class, "setProperty".to_string(), "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;".to_string());
            frame.pop();
        }
    }
}

