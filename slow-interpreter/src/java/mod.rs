macro_rules! as_object_or_java_value {
    () => {
        // use crate::java_values::{Object, JavaValue, NormalObject};
        // use std::sync::Arc;
        //
        pub fn object(self) -> std::sync::Arc<crate::java_values::Object>{
            self.normal_object
        }

        pub fn java_value(self) -> JavaValue{
            JavaValue::Object(self.object().into())
        }

        pub fn to_string(&self, state: &mut crate::InterpreterState, frame: std::rc::Rc<crate::StackEntry>) -> crate::java::lang::string::JString {
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::native::mhn_temp::run_static_or_virtual(
                state,
                &frame,
                &self.normal_object.unwrap_normal_object().class_pointer,
                "toString".to_string(),
                "()Ljava/lang/String;".to_string()
            );
            frame.pop().cast_string()
        }

        pub fn get_class(&self, state: &mut crate::InterpreterState, frame: std::rc::Rc<crate::StackEntry>) -> crate::java::lang::class::JClass {
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::native::mhn_temp::run_static_or_virtual(
                state,
                &frame,
                &self.normal_object.unwrap_normal_object().class_pointer,
                "getClass".to_string(),
                "()Ljava/lang/Class;".to_string()
            );
            frame.pop().cast_class()
        }
    };
}

#[macro_use]
pub mod lang;
