use jvmti_jni_bindings::jboolean;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{NewAsObjectOrJavaValue, NewJavaValueHandle, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::class::JClass;
use crate::utils::run_static_or_virtual;

pub struct Reflection<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_reflection(self) -> Reflection<'gc> {
        Reflection { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
    }
}

impl<'gc> Reflection<'gc> {
    pub fn is_same_class_package<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class1: JClass<'gc>, class2: JClass<'gc>) -> Result<jboolean, WasException<'gc>> {
        let reflection = check_initing_or_inited_class(jvm, int_state, CClassName::reflection().into())?;
        todo!();// int_state.push_current_operand_stack(class1.java_value());
        todo!();// int_state.push_current_operand_stack(class2.java_value()); //I hope these are in the right order, but it shouldn't matter
        let desc = CMethodDescriptor {
            arg_types: vec![CClassName::class().into(), CClassName::class().into()],
            return_type: CPDType::BooleanType,
        };
        run_static_or_virtual(jvm, int_state, &reflection, MethodName::method_isSameClassPackage(), &desc, todo!())?;
        Ok(todo!()/*int_state.pop_current_operand_stack(Some(RuntimeType::IntType)).unwrap_boolean()*/)
    }

}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Reflection<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
