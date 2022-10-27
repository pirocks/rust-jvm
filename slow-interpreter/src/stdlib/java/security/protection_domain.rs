use crate::{AllocatedHandle, NewAsObjectOrJavaValue};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;

pub struct ProtectionDomain<'gc> {
    normal_object: AllocatedHandle<'gc>,
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_protection_domain(self) -> ProtectionDomain<'gc> {
        ProtectionDomain { normal_object: self.unwrap_object_nonnull() }
    }
}

impl<'gc> ProtectionDomain<'gc> {}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ProtectionDomain<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object.normal_object()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        todo!()
    }
}
