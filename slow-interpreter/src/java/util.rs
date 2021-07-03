pub mod properties {
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::java::lang::string::JString;
    use crate::java_values::{GcManagedObject, JavaValue, Object};
    use crate::utils::run_static_or_virtual;

    pub struct Properties<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_properties(&self) -> Properties<'gc_life> {
            let res = Properties { normal_object: self.unwrap_object_nonnull() };
            assert_eq!(res.normal_object.unwrap_normal_object().objinfo.class_pointer.view().name(), ClassName::properties().into());
            res
        }
    }

    impl<'gc_life> Properties<'gc_life> {
        pub fn set_property(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, key: JString<'gc_life>, value: JString<'gc_life>) -> Result<(), WasException> {
            let properties_class = assert_inited_or_initing_class(jvm, ClassName::properties().into());
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(key.java_value());
            int_state.push_current_operand_stack(value.java_value());
            run_static_or_virtual(jvm, int_state, &properties_class, "setProperty".to_string(), "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;".to_string())?;
            int_state.pop_current_operand_stack(Some(ClassName::object().into()));
            Ok(())
        }
    }
}

