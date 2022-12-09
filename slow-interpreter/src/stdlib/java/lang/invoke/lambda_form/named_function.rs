use crate::{NewAsObjectOrJavaValue, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::java_values::JavaValue;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::stdlib::java::lang::invoke::method_type::MethodType;
use crate::stdlib::java::lang::member_name::MemberName;

#[derive(Clone)]
pub struct NamedFunction<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_lambda_form_named_function(&self) -> NamedFunction<'gc> {
        todo!()
    }
}

impl<'gc> NamedFunction<'gc> {
    //noinspection DuplicatedCode
    pub fn get_member_or_null(&self, _jvm: &'gc JVMState<'gc>) -> Option<MemberName<'gc>> {
        // let maybe_null = self.normal_object.lookup_field(jvm, FieldName::field_member());
        /*if maybe_null.try_unwrap_object().is_some() {
            if maybe_null.unwrap_object().is_some() {
                todo!()/*maybe_null.cast_member_name().into()*/
            } else {
                None
            }
        } else {
            todo!()/*maybe_null.cast_member_name().into()*/
        }*/
        todo!()
    }
    pub fn get_member(&self, jvm: &'gc JVMState<'gc>) -> MemberName<'gc> {
        self.get_member_or_null(jvm).unwrap()
    }

    pub fn method_type<'l>(&self, _jvm: &'gc JVMState<'gc>, _int_state: &mut impl PushableFrame<'gc>) -> Result<MethodType<'gc>, WasException<'gc>> {
        // java.lang.invoke.LambdaForm.NamedFunction
        /*let named_function_type = assert_inited_or_initing_class(jvm, CClassName::lambda_from_named_function().into());
        int_state.push_current_operand_stack(self.clone().java_value());
        run_static_or_virtual(jvm, int_state, &named_function_type, MethodName::method_methodType(), &CMethodDescriptor::empty_args(CClassName::method_type().into()), todo!())?;
        Ok(int_state.pop_current_operand_stack(Some(CClassName::method_type().into())).cast_method_type())*/
        todo!()
    }
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for NamedFunction<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
