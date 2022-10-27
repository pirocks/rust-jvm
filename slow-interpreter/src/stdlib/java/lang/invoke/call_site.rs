use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::{NewAsObjectOrJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter::common::invoke::virtual_::invoke_virtual;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::invoke::method_handle::MethodHandle;

#[derive(Clone)]
pub struct CallSite<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> CallSite<'gc> {
    pub fn get_target<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MethodHandle<'gc>, WasException<'gc>> {
        let call_site_class = assert_inited_or_initing_class(jvm, CClassName::call_site().into());
        let args = vec![self.new_java_value()];
        let desc = CMethodDescriptor { arg_types: vec![], return_type: CPDType::Class(CClassName::method_handle()) };
        let res = invoke_virtual(jvm, int_state, MethodName::method_getTarget(), &desc, args)?;
        Ok(res.unwrap().cast_method_handle())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for CallSite<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
