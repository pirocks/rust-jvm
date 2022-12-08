use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use crate::{AllocatedHandle, PushableFrame, WasException};
use crate::class_loading::check_initing_or_inited_class;
use crate::interpreter_util::new_object_full;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct ConstantPool<'gc> {
    normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> AllocatedHandle<'gc> {
    pub fn cast_constant_pool(self) -> ConstantPool<'gc> {
        ConstantPool { normal_object: self.unwrap_normal_object() }
    }
}

impl<'gc> ConstantPool<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, class: JClass<'gc>) -> Result<ConstantPool<'gc>, WasException<'gc>> {
        let constant_pool_classfile = check_initing_or_inited_class(jvm, int_state, CClassName::constant_pool().into())?;
        let constant_pool_object = new_object_full(jvm, int_state, &constant_pool_classfile);
        let res = constant_pool_object.cast_constant_pool();
        res.set_constant_pool_oop(jvm, class);
        Ok(res)
    }

    pub fn get_constant_pool_oop(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_constantPoolOop()).cast_class().unwrap()
    }

    pub fn set_constant_pool_oop(&self, jvm: &'gc JVMState<'gc>, jclass: JClass<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_constantPoolOop(), jclass.new_java_value());
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for ConstantPool<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
