use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use crate::{AllocatedHandle, NewAsObjectOrJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::string::JString;

pub struct ClassNotFoundException<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_class_not_found_exception(self) -> ClassNotFoundException<'gc> {
        ClassNotFoundException { normal_object: self.unwrap_normal_object() }
    }
}

impl<'gc> ClassNotFoundException<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: JString<'gc>) -> Result<ClassNotFoundException<'gc>, WasException<'gc>> {
        let class_not_found_class = check_initing_or_inited_class(jvm, int_state, CClassName::class_not_found_exception().into())?;
        let this = new_object_full(jvm, int_state, &class_not_found_class);
        run_constructor(jvm, int_state, class_not_found_class, vec![this.new_java_value(), class.new_java_value()], &CMethodDescriptor::void_return(vec![CClassName::string().into()]))?;
        Ok(this.cast_class_not_found_exception())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ClassNotFoundException<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
