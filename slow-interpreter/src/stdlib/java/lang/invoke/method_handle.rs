use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::CMethodDescriptor;
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;
use crate::{JVMState, NewAsObjectOrJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::invoke::lambda_form::LambdaForm;
use crate::stdlib::java::lang::invoke::method_handles::lookup::Lookup;
use crate::stdlib::java::lang::invoke::method_type::MethodType;
use crate::stdlib::java::lang::member_name::MemberName;
use crate::utils::run_static_or_virtual;

#[derive(Clone)]
pub struct MethodHandle<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> MethodHandle<'gc> {
    pub fn lookup<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Lookup<'gc>, WasException<'gc>> {
        let method_handles_class = assert_inited_or_initing_class(jvm, CClassName::method_handles().into());
        run_static_or_virtual(jvm, int_state, &method_handles_class, MethodName::method_lookup(), &CMethodDescriptor::empty_args(CClassName::method_handles_lookup().into()), todo!())?;
        Ok(todo!()/*int_state.pop_current_operand_stack(Some(CClassName::method_handles().into())).cast_lookup()*/)
    }
    pub fn public_lookup<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<Lookup<'gc>, WasException<'gc>> {
        let method_handles_class = assert_inited_or_initing_class(jvm, CClassName::method_handles().into());
        run_static_or_virtual(jvm, int_state, &method_handles_class, MethodName::method_publicLookup(), &CMethodDescriptor::empty_args(CClassName::method_handles_lookup().into()), todo!())?;
        Ok(todo!()/*int_state.pop_current_operand_stack(Some(CClassName::method_handles().into())).cast_lookup()*/)
    }

    pub fn internal_member_name<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>) -> Result<MemberName<'gc>, WasException<'gc>> {
        let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
        let desc = CMethodDescriptor::empty_args(CClassName::member_name().into());
        let args = vec![self.new_java_value()];
        let res = run_static_or_virtual(jvm, int_state, &method_handle_class, MethodName::method_internalMemberName(), &desc, args)?;
        Ok(res.unwrap().cast_member_name())
    }

    pub fn type__(&self, jvm: &'gc JVMState<'gc>) -> MethodType<'gc> {
        let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
        self.normal_object.get_var(jvm, &method_handle_class, FieldName::field_type()).cast_method_type()
    }

    pub fn type_<'l>(&self, _jvm: &'gc JVMState<'gc>, _int_state: &mut impl PushableFrame<'gc>) -> Result<MethodType<'gc>, WasException<'gc>> {
        /*let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
        int_state.push_current_operand_stack(self.clone().java_value());
        run_static_or_virtual(jvm, int_state, &method_handle_class, MethodName::method_type(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
        Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())*/
        todo!()
    }

    pub fn get_form_or_null(&self, jvm: &'gc JVMState<'gc>) -> Result<Option<LambdaForm<'gc>>, WasException<'gc>> {
        let method_handle_class = assert_inited_or_initing_class(jvm, CClassName::method_handle().into());
        let maybe_null = self.normal_object.get_var(jvm, &method_handle_class, FieldName::field_form());
        match maybe_null.unwrap_object() {
            Some(maybe_null) => Ok(Some(maybe_null.cast_lambda_form())),
            None => return Err(WasException { exception_obj: todo!() }),
        }
    }
    pub fn get_form(&self, jvm: &'gc JVMState<'gc>) -> Result<LambdaForm<'gc>, WasException<'gc>> {
        Ok(self.get_form_or_null(jvm)?.unwrap())
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for MethodHandle<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
