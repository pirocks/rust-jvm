use wtf8::Wtf8Buf;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;


use crate::{NewAsObjectOrJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::string::JString;

pub struct NullPointerException<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_null_pointer_exception(self) -> NullPointerException<'gc> {
        NullPointerException { normal_object: self.unwrap_object_nonnull().unwrap_normal_object() }
    }
}

impl<'gc> NullPointerException<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<NullPointerException<'gc>, WasException<'gc>> {
        let npe_class = check_initing_or_inited_class(jvm, int_state, CClassName::null_pointer_exception().into())?;
        let this = new_object(jvm, int_state, &npe_class, false);
        let message = JString::from_rust(jvm, int_state, Wtf8Buf::from_string("This jvm doesn't believe in helpful null pointer messages so you get this instead".to_string()))?;
        let desc = CMethodDescriptor::void_return(vec![CClassName::string().into()]);
        run_constructor(jvm, int_state, npe_class, vec![this.new_java_value(), message.new_java_value()], &desc)?;
        Ok(this.new_java_handle().cast_null_pointer_exception())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for NullPointerException<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
