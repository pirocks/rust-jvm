use std::marker::PhantomData;
use jvmti_jni_bindings::jint;


use crate::{JVMState, StackEntry};
use crate::java_values::{JavaValue};

pub struct Integer<'gc> {
    phantom: PhantomData<&'gc ()>
    // normal_object: GcManagedObject<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_integer(&self) -> Integer<'gc> {
        Integer { /*normal_object: self.unwrap_object_nonnull()*/ phantom: Default::default() }
    }
}

impl<'gc> Integer<'gc> {
    pub fn from(_state: &JVMState, _current_frame: &StackEntry, _i: jint) -> Integer<'gc> {
        unimplemented!()
    }

    pub fn value(&self, jvm: &'gc JVMState<'gc>) -> jint {
        todo!()
        // self.normal_object.unwrap_normal_object().get_var_top_level(jvm, FieldName::field_value()).unwrap_int()
    }

    //as_object_or_java_value!();
}
