use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::{AllocatedHandle, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::stdlib::java::lang::class_loader::ClassLoader;
use crate::utils::run_static_or_virtual;

pub struct Launcher<'gc> {
    normal_object: AllocatedHandle<'gc>,
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_launcher(self) -> Launcher<'gc> {
        Launcher { normal_object: self }
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_launcher(self) -> Launcher<'gc> {
        Launcher { normal_object: self.unwrap_object_nonnull() }
    }
}

impl<'gc> Launcher<'gc> {
    pub fn get_launcher<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Launcher<'gc>, WasException<'gc>> {
        let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
        let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getLauncher(), &CMethodDescriptor::empty_args(CClassName::launcher().into()), vec![])?.unwrap();
        Ok(res.cast_launcher())
    }

    pub fn get_loader<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<ClassLoader<'gc>, WasException<'gc>> {
        let launcher = check_initing_or_inited_class(jvm, int_state, CClassName::launcher().into())?;
        let res = run_static_or_virtual(jvm, int_state, &launcher, MethodName::method_getClassLoader(), &CMethodDescriptor::empty_args(CClassName::classloader().into()), vec![self.normal_object.new_java_value()])?.unwrap();
        Ok(res.cast_class_loader())
    }
}

pub mod ext_class_loader {
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
}
