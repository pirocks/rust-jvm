pub mod unsafe_ {
    use crate::java_values::{Object, JavaValue};
    use std::sync::Arc;
    use crate::{JVMState, InterpreterStateGuard};

    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use crate::java::lang::reflect::field::Field;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;

    pub struct Unsafe {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_unsafe(&self) -> Unsafe {
            Unsafe { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Unsafe {
        pub fn the_unsafe<'l>(jvm: &'static JVMState, int_state: & mut InterpreterStateGuard) -> Unsafe {
            let unsafe_class = check_inited_class(jvm, int_state,&ClassName::unsafe_().into(), int_state.current_loader(jvm));
            let static_vars = unsafe_class.static_vars();
            static_vars.get("theUnsafe").unwrap().clone().cast_unsafe()
        }

        pub fn object_field_offset<'l>(&self,jvm: &'static JVMState, int_state: & mut InterpreterStateGuard, field: Field) -> JavaValue{
            let desc_str =  "(Ljava/lang/reflect/Field;)J";
            int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
            int_state.push_current_operand_stack(field.java_value());
            let rc = self.normal_object.unwrap_normal_object().class_pointer.clone();
            run_static_or_virtual(jvm,int_state,&rc,"objectFieldOffset".to_string(),desc_str.to_string());
            int_state.pop_current_operand_stack()
        }

        as_object_or_java_value!();
    }
}