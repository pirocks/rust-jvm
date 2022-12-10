use crate::{AllocatedHandle, NewAsObjectOrJavaValue, NewJavaValue, NewJavaValueHandle};
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::{AllocatedNormalObjectHandle, AllocatedObject};

pub struct JObject<'gc> {
    pub(crate) normal_object: AllocatedHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_object(&self) -> JObject<'gc> {
        JObject { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
    }
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_object(self) -> JObject<'gc> {
        JObject { normal_object: self }
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_object(self) -> JObject<'gc> {
        JObject { normal_object: self.unwrap_object_nonnull() }
    }
}


impl<'gc> NewAsObjectOrJavaValue<'gc> for JObject<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object.unwrap_normal_object()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        self.normal_object.unwrap_normal_object_ref()
    }

    fn full_object(self) -> AllocatedHandle<'gc> {
        AllocatedHandle::NormalObject(self.object())
    }

    fn full_object_ref(&self) -> AllocatedObject<'gc, '_> {
        self.normal_object.as_allocated_obj()
    }

    fn new_java_value_handle(self) -> NewJavaValueHandle<'gc> {
        NewJavaValueHandle::Object(self.normal_object)
    }

    fn new_java_value(&self) -> NewJavaValue<'gc, '_> {
        self.normal_object.new_java_value()
    }
}
