use crate::java::lang::class::JClass;
use crate::{InterpreterStateGuard, JavaValue, JVMState, NewJavaValue, WasException};
use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle, NewJavaValueHandle};

pub trait NewAsObjectOrJavaValue<'gc_life>: Sized {
    fn object(self) -> AllocatedObjectHandle<'gc_life>;


    fn java_value(self) -> JavaValue<'gc_life> {
        todo!()
    }

    fn new_java_value_handle(self) -> NewJavaValueHandle<'gc_life> {
        NewJavaValueHandle::Object(self.object())
    }

    fn get_class(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) -> Result<JClass<'gc_life>, WasException> {
        todo!();/*int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));*/
        /*let desc = rust_jvm_common::compressed_classfile::CMethodDescriptor {
            arg_types: vec![],
            return_type: rust_jvm_common::compressed_classfile::CPDType::Ref(rust_jvm_common::compressed_classfile::CPRefType::Class(rust_jvm_common::compressed_classfile::names::CClassName::class())),
        };
        crate::instructions::invoke::virtual_::invoke_virtual(jvm, int_state, rust_jvm_common::compressed_classfile::names::MethodName::method_getClass(), &desc)?;
        Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::compressed_classfile::names::CClassName::class().into())).to_new().cast_class().expect("object can never not have a class"))*/
    }
}

macro_rules! as_object_or_java_value {
    () => {
        // use crate::java_values::{Object, JavaValue, NormalObject};
        //
        //
        /*pub fn object(self) -> crate::new_java_values::AllocatedObject<'gc_life,'todo> {
            /*self.normal_object*/
            todo!()
        }*/

        pub fn java_value(self) -> JavaValue<'gc_life> {
            /*JavaValue::Object(self.object().into())*/
            todo!()
        }

        pub fn to_string(&self, jvm: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<Option<crate::java::lang::string::JString<'gc_life>>, crate::WasException> {
            todo!();/*int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));*/
            let desc = rust_jvm_common::compressed_classfile::CMethodDescriptor {
                arg_types: vec![],
                return_type: rust_jvm_common::compressed_classfile::CPDType::Ref(rust_jvm_common::compressed_classfile::CPRefType::Class(rust_jvm_common::compressed_classfile::names::CClassName::string())),
            };
            crate::instructions::invoke::virtual_::invoke_virtual(jvm, int_state, rust_jvm_common::compressed_classfile::names::MethodName::method_toString(), &desc, todo!())?;
            Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::compressed_classfile::names::CClassName::string().into())).cast_string())
        }

        pub fn get_class(&self, jvm: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<crate::java::lang::class::JClass<'gc_life>, crate::WasException> {
            todo!();/*int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));*/
            let desc = rust_jvm_common::compressed_classfile::CMethodDescriptor {
                arg_types: vec![],
                return_type: rust_jvm_common::compressed_classfile::CPDType::Ref(rust_jvm_common::compressed_classfile::CPRefType::Class(rust_jvm_common::compressed_classfile::names::CClassName::class())),
            };
            crate::instructions::invoke::virtual_::invoke_virtual(jvm, int_state, rust_jvm_common::compressed_classfile::names::MethodName::method_getClass(), &desc, todo!())?;
            Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::compressed_classfile::names::CClassName::class().into())).to_new().cast_class().expect("object can never not have a class"))
        }

        pub fn hash_code(&self, jvm: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<i32, crate::WasException> {
            todo!();/*int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));*/
            let desc = rust_jvm_common::compressed_classfile::CMethodDescriptor { arg_types: vec![], return_type: rust_jvm_common::compressed_classfile::CPDType::IntType };
            crate::instructions::invoke::virtual_::invoke_virtual(jvm, int_state, rust_jvm_common::compressed_classfile::names::MethodName::method_hashCode(), &desc, todo!())?;
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
