macro_rules! as_object_or_java_value {
    () => {
        // use crate::java_values::{Object, JavaValue, NormalObject};
        // use std::sync::Arc;
        //
        pub fn object(self) -> GcManagedObject<'gc_life>{
            self.normal_object
        }

        pub fn java_value(self) -> JavaValue<'gc_life>{
            JavaValue::Object(todo!()/*self.object().into()*/)
        }

        pub fn to_string(&self, jvm: &'_ crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'_>) -> Result<Option<crate::java::lang::string::JString<'gc_life>>,crate::WasException> {
            int_state.current_frame_mut().push(jvm, JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            crate::instructions::invoke::virtual_::invoke_virtual(
             jvm,
             int_state,
             &"toString".to_string(),
             &rust_jvm_common::descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::string()))}
             )?;
            Ok(int_state.current_frame_mut().pop(jvm,rust_jvm_common::classnames::ClassName::string().into()).cast_string())
        }

        pub fn get_class(&self, state: &'_ crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'_>) -> Result<crate::java::lang::class::JClass<'gc_life>,crate::WasException> {
            int_state.current_frame_mut().push(state, JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            crate::instructions::invoke::virtual_::invoke_virtual(state, int_state,&"getClass".to_string(), &rust_jvm_common::descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::Ref(rust_jvm_common::ptype::ReferenceType::Class(rust_jvm_common::classnames::ClassName::class()))})?;
            Ok(int_state.current_frame_mut().pop(state,rust_jvm_common::classnames::ClassName::class().into()).cast_class().expect("object can never not have a class"))
        }

        pub fn hash_code(&self, state: &'_ crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'_>) -> Result<i32,crate::WasException> {
            int_state.current_frame_mut().push(state, JavaValue::Object(todo!()/*self.normal_object.clone().into()*/));
            crate::instructions::invoke::virtual_::invoke_virtual(state,int_state,&"hashCode".to_string(), &rust_jvm_common::descriptor_parser::MethodDescriptor {parameter_types: vec![], return_type: rust_jvm_common::ptype::PType::IntType})?;
            Ok(int_state.current_frame_mut().pop(state,classfile_view::view::ptype_view::PTypeView::IntType).unwrap_int())
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