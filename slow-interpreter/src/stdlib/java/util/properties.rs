use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::{JVMState, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

pub struct Properties<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> Properties<'gc> {
    pub fn set_property<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, key: JString<'gc>, value: JString<'gc>) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
        let properties_class = assert_inited_or_initing_class(jvm, CClassName::properties().into());
        let args = vec![self.new_java_value(), key.new_java_value(), value.new_java_value()];
        let desc = CMethodDescriptor {
            arg_types: vec![CClassName::string().into(), CClassName::string().into()],
            return_type: CPDType::object(),
        };
        let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_setProperty(), &desc, args)?;
        Ok(res.unwrap())
    }

    pub fn get_property<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, key: JString<'gc>) -> Result<Option<JString<'gc>>, WasException<'gc>> {
        let properties_class = assert_inited_or_initing_class(jvm, CClassName::properties().into());
        let args = vec![self.new_java_value(), key.new_java_value()];
        let desc = CMethodDescriptor {
            arg_types: vec![CClassName::string().into()],
            return_type: CClassName::string().into(),
        };
        let res = run_static_or_virtual(jvm, int_state, &properties_class, MethodName::method_getProperty(), &desc, args)?;
        Ok(res.unwrap().cast_string_maybe_null())
    }

    pub fn table(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        let hashtable_rc = assert_inited_or_initing_class(jvm, CClassName::hashtable().into());
        self.normal_object.get_var(jvm, &hashtable_rc, FieldName::field_table())
    }
}


impl<'gc> NewAsObjectOrJavaValue<'gc> for Properties<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
