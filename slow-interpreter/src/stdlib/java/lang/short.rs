use jvmti_jni_bindings::jshort;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{NewAsObjectOrJavaValue, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::java_values::{GcManagedObject, JavaValue};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;

pub struct Short<'gc> {
    normal_object: GcManagedObject<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_short(&self) -> Short<'gc> {
        Short { normal_object: self.unwrap_object_nonnull() }
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_short(&self) -> Short<'gc> {
        Short { normal_object: todo!() }
    }
}

impl<'gc> Short<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, param: jshort) -> Result<Short<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::short().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
        run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), NewJavaValue::Short(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::ShortType]))?;
        Ok(this.cast_short())
    }

    pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jshort {
        self.normal_object.lookup_field(jvm, FieldName::field_value()).unwrap_short()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Short<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        todo!()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        todo!()
    }
}
