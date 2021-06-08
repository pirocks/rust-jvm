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

        pub fn to_string(&self, jvm: &crate::jvm_state::JVMState, int_state: & mut crate::InterpreterStateGuard) -> Result<Option<crate::java::lang::string::JString>,crate::WasException> {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(
             jvm,
             int_state,
             &"toString".to_string(),
             &rust_jvm_common::descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::string()))}
             )?;
            Ok(int_state.current_frame_mut().pop(rust_jvm_common::classnames::ClassName::string().into()).cast_string())
        }

        pub fn get_class<'l>(&self, state: &crate::jvm_state::JVMState, int_state: &'l mut crate::InterpreterStateGuard) -> Result<crate::java::lang::class::JClass,crate::WasException> {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state, int_state,&"getClass".to_string(), &rust_jvm_common::descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::class()))})?;
            Ok(int_state.current_frame_mut().pop(rust_jvm_common::classnames::ClassName::class().into()).cast_class().expect("object can never not have a class"))
        }

        pub fn hash_code<'l>(&self, state: &crate::jvm_state::JVMState, int_state: &'l mut crate::InterpreterStateGuard<'l>) -> Result<i32,crate::WasException> {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state,int_state,&"hashCode".to_string(), &rust_jvm_common::descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::IntType})?;
            Ok(int_state.current_frame_mut().pop(classfile_view::view::ptype_view::PTypeView::IntType).unwrap_int())
        }
    };
}



#[macro_use]
pub mod lang;
#[macro_use]
pub mod util;
#[macro_use]
pub mod nio;
#[macro_use]
pub mod security;