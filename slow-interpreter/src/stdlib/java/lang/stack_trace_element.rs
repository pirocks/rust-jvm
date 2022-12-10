use jvmti_jni_bindings::jint;
use rust_jvm_common::classfile::LineNumber;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};


use crate::{AllocatedHandle, NewJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct StackTraceElement<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> StackTraceElement<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, declaring_class: JString<'gc>, method_name: JString<'gc>, file_name: JString<'gc>, line_number: LineNumber) -> Result<StackTraceElement<'gc>, WasException<'gc>> {
        let class_ = check_initing_or_inited_class(jvm, int_state, CClassName::stack_trace_element().into())?;
        let res = AllocatedHandle::NormalObject(new_object(jvm, int_state, &class_, false));
        let full_args = vec![res.new_java_value(), declaring_class.new_java_value(), method_name.new_java_value(), file_name.new_java_value(), NewJavaValue::Int(line_number.0 as jint)];
        let desc = CMethodDescriptor::void_return(vec![CClassName::string().into(), CClassName::string().into(), CClassName::string().into(), CPDType::IntType]);
        run_constructor(jvm, int_state, class_, full_args, &desc)?;
        Ok(res.cast_stack_trace_element())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for StackTraceElement<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
