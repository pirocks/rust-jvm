use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::exceptions::WasException;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::{AllocatedNormalObjectHandle};
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct NoSuchMethodError<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> NoSuchMethodError<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<NoSuchMethodError<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::no_such_method_error().into())?;
        let this = new_object_full(jvm, int_state, &class_not_found_class);
        run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value()], &CMethodDescriptor::void_return(vec![]))?;
        Ok(this.cast_no_such_method_error())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for NoSuchMethodError<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}

