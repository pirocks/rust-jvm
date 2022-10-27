use jvmti_jni_bindings::jint;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{check_initing_or_inited_class, JavaValueCommon, JVMState, NewJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::{new_object, run_constructor};
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::class::JClass;
use crate::stdlib::java::lang::invoke::method_type::MethodType;
use crate::stdlib::java::lang::reflect::constructor::Constructor;
use crate::stdlib::java::lang::reflect::field::Field;
use crate::stdlib::java::lang::reflect::method::Method;
use crate::stdlib::java::lang::string::JString;
use crate::stdlib::java::NewAsObjectOrJavaValue;
use crate::utils::run_static_or_virtual;

#[derive(Clone)]
pub struct MemberName<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> MemberName<'gc> {
    // private Class<?> clazz;
    // private String name;
    // private Object type;
    // private int flags;
    pub fn get_name_func<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<JString<'gc>>, WasException<'gc>> {
        let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
        let args = vec![self.normal_object.new_java_value()];
        let desc = CMethodDescriptor::empty_args(CClassName::string().into());
        let res = run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getName(), &desc, args)?;
        Ok(res.unwrap().cast_string())
    }

    pub fn is_static<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<bool, WasException<'gc>> {
        let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
        let desc = CMethodDescriptor::empty_args(CPDType::BooleanType);
        let args = vec![self.normal_object.new_java_value()];
        let res = run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_isStatic(), &desc, args)?;
        Ok(res.unwrap().as_njv().unwrap_int() != 0)
    }

    pub fn get_name_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JString<'gc>> {
        let str_jvalue = self.normal_object.get_var_top_level(jvm, FieldName::field_name());
        Some(str_jvalue.unwrap_object()?.cast_string())
    }

    pub fn get_name(&self, jvm: &'gc JVMState<'gc>) -> JString<'gc> {
        self.get_name_or_null(jvm).unwrap()
    }

    pub fn set_name(&self, jvm: &'gc JVMState<'gc>, new_val: JString<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_name(), new_val.new_java_value());
    }

    pub fn get_clazz_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
        let possibly_null = self.normal_object.get_var_top_level(jvm, FieldName::field_clazz());
        Some(possibly_null.unwrap_object()?.cast_class())
    }

    pub fn get_clazz(&self, jvm: &'gc JVMState<'gc>) -> JClass<'gc> {
        self.get_clazz_or_null(jvm).unwrap()
    }

    pub fn set_clazz(&self, jvm: &'gc JVMState<'gc>, new_val: JClass<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_clazz(), new_val.new_java_value());
    }

    pub fn set_type(&self, jvm: &'gc JVMState<'gc>, new_val: MethodType<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_type(), new_val.new_java_value());
    }

    pub fn get_type(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_type())
    }

    pub fn set_flags(&self, jvm: &'gc JVMState<'gc>, new_val: jint) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_flags(), NewJavaValue::Int(new_val));
    }

    pub fn get_flags_or_null(&self, jvm: &'gc JVMState<'gc>) -> Option<jint> {
        Some(self.normal_object.get_var_top_level(jvm, FieldName::field_flags()).unwrap_int())
        /*let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_flags());
        if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                maybe_null.unwrap_int().into()
            } else {
                None
            }
        } else {
            maybe_null.unwrap_int().into()
        }*/
    }
    pub fn get_flags(&self, jvm: &'gc JVMState<'gc>) -> jint {
        self.get_flags_or_null(jvm).unwrap()
    }

    pub fn set_resolution(&self, jvm: &'gc JVMState<'gc>, new_val: NewJavaValue<'gc, '_>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_resolution(), new_val);
    }

    pub fn get_resolution(&self, jvm: &'gc JVMState<'gc>) -> NewJavaValueHandle<'gc> {
        self.normal_object.get_var_top_level(jvm, FieldName::field_resolution())
    }

    pub fn clazz(&self, jvm: &'gc JVMState<'gc>) -> Option<JClass<'gc>> {
        Some(self.normal_object.get_var_top_level(jvm, FieldName::field_clazz()).unwrap_object()?.cast_class())
    }

    pub fn get_method_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MethodType<'gc>, WasException<'gc>> {
        /*let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
        int_state.push_current_operand_stack(JavaValue::Object(self.normal_object.clone().into()));
        run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getMethodType(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
        Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())*/
        todo!()
    }

    pub fn get_field_type<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Option<JClass<'gc>>, WasException<'gc>> {
        let member_name_class = assert_inited_or_initing_class(jvm, CClassName::member_name().into());
        let args = vec![self.normal_object.new_java_value()];
        let desc = CMethodDescriptor::empty_args(CClassName::class().into());
        let res = run_static_or_virtual(jvm, int_state, &member_name_class, MethodName::method_getFieldType(), &desc, args)?;
        Ok(res.unwrap().cast_class())
    }

    pub fn new_from_field<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, field: Field<'gc>) -> Result<Self, WasException<'gc>> {
        /*let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
        let res = new_object(jvm, int_state, &member_class).to_jv();
        run_constructor(jvm, int_state, member_class, todo!()/*vec![res.clone(), field.java_value()]*/, &CMethodDescriptor::void_return(vec![CClassName::field().into()]))?;
        Ok(res.cast_member_name())*/
        todo!()
    }

    pub fn new_from_method<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, method: Method<'gc>) -> Result<Self, WasException<'gc>> {
        let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
        let res = new_object(jvm, int_state, &member_class, false);
        let desc = CMethodDescriptor::void_return(vec![CClassName::method().into()]);
        run_constructor(jvm, int_state, member_class, vec![res.new_java_value(), method.new_java_value()], &desc)?;
        Ok(res.cast_member_name())
    }

    pub fn new_from_constructor<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, constructor: Constructor<'gc>) -> Result<Self, WasException<'gc>> {
        let member_class = check_initing_or_inited_class(jvm, int_state, CClassName::member_name().into())?;
        let res = new_object(jvm, int_state, &member_class, false);
        let desc = CMethodDescriptor::void_return(vec![CClassName::constructor().into()]);
        let args = vec![res.new_java_value(), constructor.new_java_value()];
        run_constructor(jvm, int_state, member_class, args, &desc)?;
        Ok(res.cast_member_name())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for MemberName<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
