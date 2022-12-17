use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::{PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

pub struct Launcher<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> Launcher<'gc> {
    pub fn get_launcher<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Launcher<'gc>, WasException<'gc>> {
        let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
        let desc = CMethodDescriptor::empty_args(CClassName::launcher().into());
        let args = vec![];
        let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getLauncher(), &desc, args)?.unwrap();
        Ok(res.cast_launcher())
    }

    pub fn get_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<ClassLoader<'gc>, WasException<'gc>> {
        let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
        let desc = CMethodDescriptor::empty_args(CClassName::classloader().into());
        let args = vec![self.normal_object.new_java_value()];
        let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getClassLoader(), &desc, args)?.unwrap();
        Ok(res.cast_class_loader())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Launcher<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}

pub mod ext_class_loader;
