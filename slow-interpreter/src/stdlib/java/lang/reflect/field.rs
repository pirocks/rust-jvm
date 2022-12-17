use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{JVMState, NewAsObjectOrJavaValue, NewJavaValue, PushableFrame, WasException};
use crate::class_loading::{assert_inited_or_initing_class, check_initing_or_inited_class};
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::java_value_common::JavaValueCommon;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::string::JString;

pub struct Field<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_field(&self) -> Field<'gc> {
        Field { normal_object: todo!() }
    }
}

impl<'gc> Field<'gc> {
    pub fn init<'l>(
        jvm: &'gc JVMState<'gc>,
        int_state: &mut impl PushableFrame<'gc>,
        clazz: JClass<'gc>,
        name: JString<'gc>,
        type_: JClass<'gc>,
        modifiers: jint,
        slot: jint,
        signature: Option<JString<'gc>>,
        annotations: NewJavaValueHandle<'gc>,
    ) -> Result<Self, WasException<'gc>> {
        let field_classfile = check_initing_or_inited_class(jvm, int_state, CClassName::field().into())?;
        let field_object = new_object_full(jvm, int_state, &field_classfile);

        let modifiers = NewJavaValue::Int(modifiers);
        let slot = NewJavaValue::Int(slot);


        run_constructor(
            jvm,
            int_state,
            field_classfile,
            vec![field_object.new_java_value(),
                 clazz.new_java_value(),
                 name.new_java_value(),
                 type_.new_java_value(),
                 modifiers,
                 slot,
                 signature.as_ref().map(|signature| signature.new_java_value()).unwrap_or(NewJavaValue::Null),
                 annotations.as_njv()],
            &CMethodDescriptor::void_return(vec![CClassName::class().into(),
                                                 CClassName::string().into(),
                                                 CClassName::class().into(),
                                                 CPDType::IntType,
                                                 CPDType::IntType,
                                                 CClassName::string().into(),
                                                 CPDType::array(CPDType::ByteType)]),
        )?;

        Ok(field_object.cast_field())
    }

    pub fn name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
        let field_rc = assert_inited_or_initing_class(jvm, CClassName::field().into());
        self.normal_object.get_var(jvm, &field_rc, FieldName::field_name()).cast_string_maybe_null().expect("fields must have names")
    }

    pub fn clazz(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        let field_rc = assert_inited_or_initing_class(jvm, CClassName::field().into());
        self.normal_object.get_var(jvm, &field_rc, FieldName::field_clazz()).cast_class().expect("todo")
    }


}

impl<'gc> NewAsObjectOrJavaValue<'gc> for Field<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
