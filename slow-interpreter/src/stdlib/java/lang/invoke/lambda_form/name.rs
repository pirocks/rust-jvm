use itertools::Itertools;

use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::NewJavaValueHandle;
use crate::stdlib::java::lang::invoke::lambda_form::basic_type::BasicType;
use crate::stdlib::java::lang::invoke::lambda_form::named_function::NamedFunction;

#[derive(Clone)]
pub struct Name<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_lambda_form_name(&self) -> Name<'gc> {
        todo!()
    }
}

impl<'gc> Name<'gc> {
    pub fn arguments(&self, jvm: &'gc JVMState<'gc>) -> Vec<NewJavaValueHandle<'gc>> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_arguments()).unwrap_object_nonnull().unwrap_array().array_iterator().collect_vec()
    }

    //noinspection DuplicatedCode
    pub fn get_index_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_index());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.unwrap_int().into()
            } else {
                None
            }
        } else {
            maybe_null.unwrap_int().into()
        }*/
    }
    pub fn get_index(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.get_index_or_null(jvm).unwrap()
    }
    pub fn get_type_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<BasicType<'gc>> {
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_type());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.cast_lambda_form_basic_type().into()
            } else {
                None
            }
        } else {
            maybe_null.cast_lambda_form_basic_type().into()
        }*/
    }
    pub fn get_type(&self, jvm: &'gc JVMState<'gc>) -> BasicType<'gc> {
        self.get_type_or_null(jvm).unwrap()
    }
    pub fn get_function_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<NamedFunction<'gc>> {
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_function());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.cast_lambda_form_named_function().into()
            } else {
                None
            }
        } else {
            maybe_null.cast_lambda_form_named_function().into()
        }*/
    }
    pub fn get_function(&self, jvm: &'gc JVMState<'gc>) -> NamedFunction<'gc> {
        self.get_function_or_null(jvm).unwrap()
    }
}
