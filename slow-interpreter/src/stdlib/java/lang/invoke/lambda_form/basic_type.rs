use jvmti_jni_bindings::jchar;
use jvmti_jni_bindings::jint;

use crate::{JString, NewAsObjectOrJavaValue};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::class::JClass;

#[derive(Clone)]
pub struct BasicType<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> BasicType<'gc> {
    //noinspection DuplicatedCode
    pub fn get_ordinal_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_ordinal());
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
    pub fn get_ordinal(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.get_ordinal_or_null(jvm).unwrap()
    }
    pub fn get_bt_char_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jchar> {
        todo!()
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_btChar());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.unwrap_char().into()
            } else {
                None
            }
        } else {
            maybe_null.unwrap_char().into()
        }*/
    }
    pub fn get_bt_char(&self, jvm: &'gc JVMState<'gc>) -> jchar {
        self.get_bt_char_or_null(jvm).unwrap()
    }

    //noinspection DuplicatedCode
    pub fn get_bt_class_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
        // let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_btClass());
        todo!()
        /*if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.to_new().cast_class().into()
            } else {
                None
            }
        } else {
            maybe_null.to_new().cast_class().into()
        }*/
    }
    pub fn get_bt_class(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        self.get_bt_class_or_null(jvm).unwrap()
    }
    pub fn get_name_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JString<'gc>> {
        // let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_name());
        todo!()
        /*if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.cast_string().into()
            } else {
                None
            }
        } else {
            maybe_null.cast_string().into()
        }*/
    }
    pub fn get_name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
        self.get_name_or_null(jvm).unwrap()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for BasicType<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
