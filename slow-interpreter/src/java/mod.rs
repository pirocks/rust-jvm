use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType, CPRefType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use crate::java::lang::class::JClass;
use crate::{InterpreterStateGuard, JavaValue, JString, JVMState, NewJavaValue, WasException};
use crate::instructions::invoke::virtual_::invoke_virtual;
use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle, NewJavaValueHandle};

pub trait NewAsObjectOrJavaValue<'gc_life>: Sized {
    fn object(self) -> AllocatedObjectHandle<'gc_life>;
    fn object_ref(&self) -> AllocatedObject<'gc_life,'_>;


    fn java_value(self) -> JavaValue<'gc_life> {
        todo!()
    }

    fn new_java_value_handle(self) -> NewJavaValueHandle<'gc_life> {
        NewJavaValueHandle::Object(self.object())
    }

    fn new_java_value(&self) -> NewJavaValue<'gc_life,'_>{
        NewJavaValue::AllocObject(self.object_ref())
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

    fn hash_code(&self, jvm: &'gc_life crate::jvm_state::JVMState<'gc_life>, int_state: &'_ mut crate::InterpreterStateGuard<'gc_life,'l>) -> Result<i32, crate::WasException> {
        let desc = CMethodDescriptor { arg_types: vec![], return_type: CPDType::IntType };
        let res = invoke_virtual(jvm, int_state, MethodName::method_hashCode(), &desc, vec![self.new_java_value()])?;
        Ok(res.unwrap().as_njv().unwrap_int_strict())
    }

    fn to_string(&self, jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life,'l>) -> Result<Option<JString<'gc_life>>, WasException> {
        let desc = CMethodDescriptor {
            arg_types: vec![],
            return_type: CPDType::Ref(CPRefType::Class(CClassName::string())),
        };
        let res = invoke_virtual(jvm, int_state, MethodName::method_toString(), &desc, vec![self.new_java_value()])?.unwrap();
        Ok(res.cast_string())
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
