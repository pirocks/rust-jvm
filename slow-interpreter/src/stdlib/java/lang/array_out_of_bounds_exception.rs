use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};


use crate::{NewAsObjectOrJavaValue, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;

pub struct ArrayOutOfBoundsException<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> ArrayOutOfBoundsException<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, index: jint) -> Result<ArrayOutOfBoundsException<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::array_out_of_bounds_exception().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false);
        let desc = CMethodDescriptor::void_return(vec![CPDType::IntType]);
        let args = vec![this.new_java_value(), NewJavaValue::Int(index)];
        run_constructor(jvm, int_state, class_not_found_class, args, &desc)?;
        Ok(this.cast_array_out_of_bounds_exception())
    }

    pub fn new_no_index<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<ArrayOutOfBoundsException<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::array_out_of_bounds_exception().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false);
        let desc = CMethodDescriptor::void_return(vec![]);
        run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value()], &desc)?;
        Ok(this.cast_array_out_of_bounds_exception())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ArrayOutOfBoundsException<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
