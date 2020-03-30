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
            crate::instructions::invoke::virtual_::invoke_virtual(
             state,
             frame.clone(),
             &"toString".to_string(),
             &descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::string()))});
            frame.pop().cast_string()
        }

        pub fn get_class(&self, state: &mut crate::InterpreterState, frame: std::rc::Rc<crate::StackEntry>) -> crate::java::lang::class::JClass {
            frame.push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state, frame.clone(),&"getClass".to_string(), &descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::class()))});
            frame.pop().cast_class()
        }
    };
}

#[macro_use]
pub mod lang;
