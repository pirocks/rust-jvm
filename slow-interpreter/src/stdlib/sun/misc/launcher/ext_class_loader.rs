use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::utils::run_static_or_virtual;
use crate::{NewAsObjectOrJavaValue, WasException};

pub struct ExtClassLoader<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> ExtClassLoader<'gc> {
    pub fn get_ext_class_loader<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<ExtClassLoader<'gc>, WasException<'gc>> {
        let ext_class_loader = check_initing_or_inited_class(jvm, int_state, CClassName::ext_class_loader().into())?;
        run_static_or_virtual(jvm, int_state, &ext_class_loader, MethodName::method_getExtClassLoader(), &CMethodDescriptor::empty_args(CClassName::launcher().into()), todo!())?;
        Ok(todo!()/*int_state.pop_current_operand_stack(Some(CClassName::classloader().into())).cast_ext_class_launcher()*/)
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ExtClassLoader<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
