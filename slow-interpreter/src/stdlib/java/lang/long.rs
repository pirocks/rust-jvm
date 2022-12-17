use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JavaValueCommon, NewAsObjectOrJavaValue, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;

pub struct Long<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> Long<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jlong) -> Result<Long<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::long().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false);
        let args = vec![this.new_java_value(), NewJavaValue::Long(param)];
        let desc = CMethodDescriptor::void_return(vec![CPDType::LongType]);
        run_constructor(jvm, int_state, class_not_found_class, args, &desc)?;
        Ok(this.cast_long())
    }

    pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jlong {
        self.normal_object.get_var_top_level(jvm, FieldName::field_value()).as_njv().unwrap_long_strict()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Long<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
