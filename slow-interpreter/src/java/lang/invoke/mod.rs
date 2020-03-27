pub mod method_type {
    use crate::java_values::{JavaValue, Object};
    use crate::{InterpreterState, StackEntry};
    use std::rc::Rc;
    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use std::sync::Arc;

    pub struct MethodType {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method_type(&self) -> MethodType {
            MethodType { normal_object: self.unwrap_object_nonnull() }
        }

    }

    impl MethodType{
        pub fn to_string(&self, state: &mut InterpreterState, frame: Rc<StackEntry>) -> JString {
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            run_static_or_virtual(
                state,
                &frame,
                &self.normal_object.unwrap_normal_object().class_pointer,
                "toString".to_string(),
                "()Ljava/lang/String;".to_string()
            );
            frame.pop().cast_string()
        }
    }
}
