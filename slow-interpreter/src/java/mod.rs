macro_rules! as_object_or_java_value {
    () => {
        // use crate::java_values::{Object, JavaValue, NormalObject};
        //
        //
        pub fn object(self) -> GcManagedObject<'gc_life>{
            self.normal_object
        }

        pub fn java_value(self) -> JavaValue<'gc_life>{
            JavaValue::Object(self.object().into())
        }

        pub fn to_string(&self, jvm: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<Option<crate::java::lang::string::JString<'gc_life>>,crate::WasException> {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(
             jvm,
             int_state,
             rust_jvm_common::compressed_classfile::names::MethodName::method_toString(),
             &rust_jvm_common::compressed_classfile::CMethodDescriptor {arg_types: vec![], return_type: rust_jvm_common::compressed_classfile::CPDType::Ref(rust_jvm_common::compressed_classfile::CPRefType::Class(rust_jvm_common::compressed_classfile::names::CClassName::string()))}
             )?;
            Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::compressed_classfile::names::CClassName::string().into())).cast_string())
        }

        pub fn get_class(&self, state: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<crate::java::lang::class::JClass<'gc_life>,crate::WasException> {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state, int_state,rust_jvm_common::compressed_classfile::names::MethodName::method_getClass(), &rust_jvm_common::compressed_classfile::CMethodDescriptor {arg_types: vec![], return_type: rust_jvm_common::compressed_classfile::CPDType::Ref(rust_jvm_common::compressed_classfile::CPRefType::Class(rust_jvm_common::compressed_classfile::names::CClassName::class()))})?;
            Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::compressed_classfile::names::CClassName::class().into())).cast_class().expect("object can never not have a class"))
        }

        pub fn hash_code(&self, state: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<i32,crate::WasException> {
            int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));
            crate::instructions::invoke::virtual_::invoke_virtual(state,int_state,rust_jvm_common::compressed_classfile::names::MethodName::method_hashCode(), &rust_jvm_common::compressed_classfile::CMethodDescriptor {arg_types: vec![], return_type: rust_jvm_common::compressed_classfile::CPDType::IntType})?;
            Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::runtime_type::RuntimeType::IntType)).unwrap_int())
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