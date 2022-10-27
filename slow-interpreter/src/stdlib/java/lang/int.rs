use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JavaValueCommon, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::NewAsObjectOrJavaValue;

pub struct Int<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_int(&self) -> Int<'gc> {
        Int { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_int(self) -> Int<'gc> {
        Int { normal_object: self.unwrap_object().unwrap().unwrap_normal_object() }
    }
}

impl<'gc, 'l> Int<'gc> {
    pub fn new<'todo>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut impl PushableFrame<'gc>, param: jint) -> Result<Int<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::int().into())?;
        let this = new_object(jvm, int_state, &class_not_found_class, false).to_jv();
        run_constructor(jvm, int_state, class_not_found_class, todo!()/*vec![this.clone(), JavaValue::Int(param)]*/, &CMethodDescriptor::void_return(vec![CPDType::IntType]))?;
        /*Ok(this.cast_int())*/
        todo!()
    }

    pub fn inner_value(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.normal_object.get_var_top_level(jvm, FieldName::field_value()).unwrap_int_strict()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Int<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        todo!()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        todo!()
    }
}
