use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::{NewAsObjectOrJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::utils::run_static_or_virtual;

pub struct Throwable<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> Clone for Throwable<'gc> {
    fn clone(&self) -> Self {
        Throwable { normal_object: self.normal_object.duplicate_discouraged() }
    }
}

impl<'gc> Throwable<'gc> {
    pub fn print_stack_trace<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<(), WasException<'gc>> {
        let throwable_class = check_initing_or_inited_class(jvm, int_state, CClassName::throwable().into()).expect("Throwable isn't inited?");
        let args = vec![self.new_java_value()];
        run_static_or_virtual(jvm, int_state, &throwable_class, MethodName::method_printStackTrace(), &CMethodDescriptor::empty_args(CPDType::VoidType), args)?;
        Ok(())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Throwable<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
