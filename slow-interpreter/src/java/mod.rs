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

        pub fn to_string(&self, state: &'static crate::JVMState, int_state: & mut crate::InterpreterStateGuard) -> crate::java::lang::string::JString {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(
             state,
             int_state,
             &"toString".to_string(),
             &descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::string()))},
             false
             );
            int_state.current_frame_mut().pop().cast_string()

        }

        pub fn get_class<'l>(&self, state: &'static crate::JVMState, int_state: &'l mut crate::InterpreterStateGuard) -> crate::java::lang::class::JClass {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state, int_state,&"getClass".to_string(), &descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::class()))}, false);
            int_state.current_frame_mut().pop().cast_class()
        }

        pub fn hash_code<'l>(&self, state: &'static crate::JVMState, int_state: &'l mut crate::InterpreterStateGuard<'l>) -> i32 {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state,int_state,&"hashCode".to_string(), &descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::IntType}, false);
            int_state.current_frame_mut().pop().unwrap_int()
        }
    };
}

#[macro_use]
pub mod lang;
pub mod util;
