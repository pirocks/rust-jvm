use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{check_initing_or_inited_class, JString, JVMState, NewAsObjectOrJavaValue, NewJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;

pub struct BigInteger<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> BigInteger<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, jstring: JString<'gc>, radix: jint) -> Result<Self, WasException<'gc>> {
        let big_integer_class = check_initing_or_inited_class(jvm, int_state, CClassName::big_integer().into())?;
        let object = new_object_full(jvm, int_state, &big_integer_class);
        let args = vec![object.new_java_value(), jstring.new_java_value(), NewJavaValue::Int(radix)];
        let method_descriptor = CMethodDescriptor::void_return(vec![CClassName::string().into(), CPDType::IntType]);
        run_constructor(jvm, int_state, big_integer_class, args, &method_descriptor)?;
        Ok(object.cast_big_integer())
    }

    pub fn signum(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_signum())
    }

    pub fn mag(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_mag())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for BigInteger<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
