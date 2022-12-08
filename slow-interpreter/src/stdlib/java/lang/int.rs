use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JavaValueCommon, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::{NewJavaValue, NewJavaValueHandle};
use crate::NewAsObjectOrJavaValue;

pub struct Int<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}


impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_int(self) -> Int<'gc> {
        Int { normal_object: self.unwrap_object().unwrap().unwrap_normal_object() }
    }
}

impl<'gc, 'l> Int<'gc> {
    pub fn new<'todo>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut impl PushableFrame<'gc>, param: jint) -> Result<Int<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::int().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false);
        run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value(), NewJavaValue::Int(param)], &CMethodDescriptor::void_return(vec![CPDType::IntType]))?;
        Ok(this.new_java_handle().cast_int())
    }

    pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.normal_object.get_var_top_level(jvm, FieldName::field_value()).unwrap_int_strict()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Int<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
