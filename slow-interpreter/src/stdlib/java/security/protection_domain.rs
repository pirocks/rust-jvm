use crate::{NewAsObjectOrJavaValue};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;

pub struct ProtectionDomain<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> ProtectionDomain<'gc> {}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ProtectionDomain<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
