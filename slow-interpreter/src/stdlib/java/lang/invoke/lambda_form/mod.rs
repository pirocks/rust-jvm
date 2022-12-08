use rust_jvm_common::compressed_classfile::field_names::FieldName;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::invoke::lambda_form::name::Name;
use crate::stdlib::java::lang::member_name::MemberName;

pub mod named_function;
pub mod name;
pub mod basic_type;

#[derive(Clone)]
pub struct LambdaForm<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> LambdaForm<'gc> {
    pub fn names(&self, jvm: &'gc JVMState<'gc>) -> Vec<Name<'gc>> {
        todo!()
        // self.normal_object.get_var_top_level(jvm, FieldName::field_names()).unwrap_object_nonnull().unwrap_array().unwrap_object_array(jvm).iter().map(|name| JavaValue::Object(todo!() /*name.clone()*/).cast_lambda_form_name()).collect()
    }

    //noinspection DuplicatedCode
    pub fn get_vmentry_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<MemberName<'gc>> {
        Some(self.normal_object.get_var_top_level(jvm, FieldName::field_vmentry()).unwrap_object()?.cast_member_name())
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_vmentry());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                todo!()/*maybe_null.cast_member_name().into()*/
            } else {
                None
            }
        } else {
            todo!()/*maybe_null.cast_member_name().into()*/
        }*/
    }
    pub fn get_vmentry(&self, jvm: &'gc JVMState<'gc>) -> MemberName<'gc> {
        self.get_vmentry_or_null(jvm).unwrap()
    }
}
