macro_rules! as_object_or_java_value {
    () => {
        // use crate::java_values::{Object, JavaValue, NormalObject};
        // use std::sync::Arc;
        //
        pub fn object(self) -> NormalObject{
            self.normal_object
        }

        pub fn java_value(self) -> JavaValue{
            JavaValue::Object(std::sync::Arc::new(crate::java_values::Object::Object(self.object())).into())
        }
    };
}

#[macro_use]
pub mod lang;
