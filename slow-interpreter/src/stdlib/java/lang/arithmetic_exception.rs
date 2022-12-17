use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::exceptions::WasException;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct ArithmeticException<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> ArithmeticException<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Self, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::arithmetic_exception().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false);
        let desc = CMethodDescriptor::void_return(vec![CPDType::IntType]);
        let args = vec![this.new_java_value()];
        run_constructor(jvm, int_state, class_not_found_class, args, &desc)?;
        Ok(this.cast_arithmetic_exception())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ArithmeticException<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}

