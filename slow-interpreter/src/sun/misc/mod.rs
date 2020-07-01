pub mod unsafe_ {
    use crate::java_values::{Object, JavaValue};
    use std::sync::Arc;
    use crate::{JVMState, StackEntry};

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
        pub fn the_unsafe(jvm: &'static JVMState, frame: &StackEntry) -> Unsafe {
            let unsafe_class = check_inited_class(jvm, &ClassName::unsafe_().into(), frame.class_pointer.loader(jvm).clone());
            let static_vars = unsafe_class.static_vars();
            static_vars.get("theUnsafe").unwrap().clone().cast_unsafe()
        }

        pub fn object_field_offset(&self,jvm: &'static JVMState, frame: &StackEntry, field: Field) -> JavaValue{
            let desc_str =  "(Ljava/lang/reflect/Field;)J";
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            frame.push(field.java_value());
            let rc = self.normal_object.unwrap_normal_object().class_pointer.clone();
            run_static_or_virtual(jvm,&rc,"objectFieldOffset".to_string(),desc_str.to_string());
            frame.pop()
        }

        as_object_or_java_value!();
    }
}