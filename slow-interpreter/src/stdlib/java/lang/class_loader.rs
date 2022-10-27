use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use rust_jvm_common::loading::LoaderName;

use crate::{AllocatedHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::NewJavaValueHandle;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

pub struct ClassLoader<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl Clone for ClassLoader<'_> {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_class_loader(&self) -> ClassLoader<'gc> {
        ClassLoader { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn cast_class_loader(self) -> ClassLoader<'gc> {
        self.unwrap_object_nonnull().cast_class_loader()
    }
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_class_loader(self) -> ClassLoader<'gc> {
        ClassLoader { normal_object: self.unwrap_normal_object() }
    }
}

impl<'gc> AllocatedNormalObjectHandle<'gc> {
    pub fn cast_class_loader(self) -> ClassLoader<'gc> {
        ClassLoader { normal_object: self }
    }
}

impl<'gc> ClassLoader<'gc> {
    pub fn to_jvm_loader(&self, jvm: &'gc JVMState<'gc>) -> LoaderName {
        let mut classes_guard = jvm.classes.write().unwrap();
        let gc_lifefied_obj = self.normal_object.duplicate_discouraged();
        classes_guard.lookup_or_add_classloader(gc_lifefied_obj)
    }

    pub fn load_class<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, name: JString<'gc>) -> Result<JClass<'gc>, WasException<'gc>> {
        let class_loader = assert_inited_or_initing_class(jvm, CClassName::classloader().into());
        let res = run_static_or_virtual(
            jvm,
            int_state,
            &class_loader,
            MethodName::method_loadClass(),
            &CMethodDescriptor { arg_types: vec![CClassName::string().into()], return_type: CClassName::class().into() },
            vec![self.new_java_value(), name.new_java_value()],
        )?.unwrap();
        Ok(res.cast_class().unwrap())
    }

    /*as_object_or_java_value!();*/
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ClassLoader<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
