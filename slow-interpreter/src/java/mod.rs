use another_jit_vm_ir::WasException;
use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, MethodName};
use crate::java::lang::class::JClass;
use crate::{AllocatedHandle, InterpreterStateGuard, JavaValue, JString, JVMState, NewJavaValue};
use crate::instructions::invoke::virtual_::invoke_virtual;
use crate::new_java_values::{NewJavaValueHandle};
use crate::new_java_values::allocated_objects::{AllocatedNormalObjectHandle, AllocatedObject};
use crate::new_java_values::java_value_common::JavaValueCommon;

pub trait NewAsObjectOrJavaValue<'gc>: Sized {
    fn object(self) -> AllocatedNormalObjectHandle<'gc>;
    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc>;


    fn full_object(self) -> AllocatedHandle<'gc>{
        AllocatedHandle::NormalObject(self.object())
    }

    fn full_object_ref(&self) -> AllocatedObject<'gc,'_>{
        AllocatedObject::NormalObject(self.object_ref())
    }

    fn new_java_value_handle(self) -> NewJavaValueHandle<'gc> {
        NewJavaValueHandle::Object(AllocatedHandle::NormalObject(self.object()))
    }

    fn new_java_value(&self) -> NewJavaValue<'gc,'_>{
        NewJavaValue::AllocObject(self.full_object_ref())
    }

    fn java_value(self) -> JavaValue<'gc> {
        todo!()
    }

    fn get_class<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Result<JClass<'gc>, WasException> {
        todo!();/*int_state.current_frame_mut().push(JavaValue::Object(self.normal_object.clone().into()));*/
        /*let desc = rust_jvm_common::compressed_classfile::CMethodDescriptor {
            arg_types: vec![],
            return_type: rust_jvm_common::compressed_classfile::CPDType::Ref(rust_jvm_common::compressed_classfile::CPRefType::Class(rust_jvm_common::compressed_classfile::names::CClassName::class())),
        };
        crate::instructions::invoke::virtual_::invoke_virtual(jvm, int_state, rust_jvm_common::compressed_classfile::names::MethodName::method_getClass(), &desc)?;
        Ok(int_state.current_frame_mut().pop(Some(rust_jvm_common::compressed_classfile::names::CClassName::class().into())).to_new().cast_class().expect("object can never not have a class"))*/
    }

    fn hash_code<'l>(&self, jvm: &'gc crate::jvm_state::JVMState<'gc>, int_state: &'_ mut crate::InterpreterStateGuard<'gc,'l>) -> Result<i32, WasException> {
        let desc = CMethodDescriptor { arg_types: vec![], return_type: CPDType::IntType };
        let res = invoke_virtual(jvm, int_state, MethodName::method_hashCode(), &desc, vec![self.new_java_value()])?;
        Ok(res.unwrap().unwrap_int_strict())
    }

    fn to_string<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>) -> Result<Option<JString<'gc>>, WasException> {
        let desc = CMethodDescriptor {
            arg_types: vec![],
            return_type: CClassName::string().into(),
        };
        let res = invoke_virtual(jvm, int_state, MethodName::method_toString(), &desc, vec![self.new_java_value()])?.unwrap();
        Ok(res.cast_string())
    }
}

#[macro_use]
pub mod lang;
#[macro_use]
pub mod util;
#[macro_use]
pub mod nio;
#[macro_use]
pub mod security;
