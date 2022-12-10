use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::exceptions::WasException;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::{AllocatedHandle, AllocatedNormalObjectHandle};
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct ClassCastException<'gc> {
    normal_object: AllocatedHandle<'gc>,
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_class_cast_exception(self) -> ClassCastException<'gc> {
        ClassCastException { normal_object: self }
    }
}

impl<'gc> ClassCastException<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<ClassCastException<'gc>, WasException<'gc>> {
        let class_cast_class = check_initing_or_inited_class(jvm, int_state, CClassName::class_cast_exception().into())?;
        let this = new_object_full(jvm, int_state, &class_cast_class);
        run_constructor(jvm, int_state, class_cast_class, vec![this.new_java_value()], &CMethodDescriptor::void_return(vec![]))?;
        Ok(this.cast_class_cast_exception())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ClassCastException<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object.unwrap_normal_object()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        self.normal_object.unwrap_normal_object_ref()
    }
}
