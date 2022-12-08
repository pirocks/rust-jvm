use jvmti_jni_bindings::jbyte;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::{NewJavaValue, NewJavaValueHandle};
use crate::new_java_values::java_value_common::JavaValueCommon;
use crate::new_java_values::owned_casts::OwnedCastAble;

pub struct Byte<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_byte(self) -> Byte<'gc> {
        Byte { normal_object: self.normal_object() }
    }
}

impl<'gc> AllocatedNormalObjectHandle<'gc> {
    pub fn cast_byte(self) -> Byte<'gc> {
        Byte { normal_object: self.normal_object() }
    }
}

impl<'gc> Byte<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jbyte) -> Result<Byte<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::byte().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false);
        let desc = CMethodDescriptor::void_return(vec![CPDType::ByteType]);
        run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value(), NewJavaValue::Byte(param)], &desc)?;
        Ok(this.cast_byte())
    }

    pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jbyte {
        self.normal_object.get_var_top_level(jvm, FieldName::field_value()).unwrap_byte_strict()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Byte<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
