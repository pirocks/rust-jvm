use jvmti_jni_bindings::jlong;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::{AllocatedHandle, NewAsObjectOrJavaValue, NewJavaValue, pushable_frame_todo};
use crate::better_java_stack::java_stack_guard::JavaStackGuard;
use crate::better_java_stack::opaque_frame::OpaqueFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::new_object;
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::lang::invoke::method_type::MethodType;

#[derive(Clone)]
pub struct MethodTypeForm<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> MethodTypeForm<'gc> {
    pub fn set_arg_to_slot_table(&self, jvm: &'gc JVMState<'gc>, int_arr: NewJavaValue<'gc, '_>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_argToSlotTable(), int_arr);
    }

    pub fn set_slot_to_arg_table(&self, jvm: &'gc JVMState<'gc>, int_arr: NewJavaValue<'gc, '_>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_slotToArgTable(), int_arr);
    }

    pub fn set_arg_counts(&self, jvm: &'gc JVMState<'gc>, counts: jlong) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_argCounts(), NewJavaValue::Long(counts));
    }

    pub fn set_prim_counts(&self, jvm: &'gc JVMState<'gc>, counts: jlong) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_primCounts(), NewJavaValue::Long(counts));
    }

    pub fn set_erased_type(&self, jvm: &'gc JVMState<'gc>, type_: MethodType<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_erasedType(), type_.new_java_value());
    }

    pub fn set_basic_type(&self, jvm: &'gc JVMState<'gc>, type_: MethodType<'gc>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_basicType(), type_.new_java_value());
    }

    pub fn set_method_handles(&self, jvm: &'gc JVMState<'gc>, method_handle: NewJavaValue<'gc, '_>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_methodHandles(), method_handle);
    }

    pub fn set_lambda_forms(&self, jvm: &'gc JVMState<'gc>, lambda_forms: NewJavaValue<'gc, '_>) {
        self.normal_object.set_var_top_level(jvm, FieldName::field_methodHandles(), lambda_forms);
    }

    pub fn new<'l>(
        jvm: &'gc JVMState<'gc>,
        int_state: &mut JavaStackGuard<'gc>,
        arg_to_slot_table: NewJavaValue<'gc, '_>,
        slot_to_arg_table: NewJavaValue<'gc, '_>,
        arg_counts: jlong,
        prim_counts: jlong,
        erased_type: Option<MethodType<'gc>>,
        basic_type: Option<MethodType<'gc>>,
        method_handles: NewJavaValue<'gc, '_>,
        lambda_forms: NewJavaValue<'gc, '_>,
    ) -> MethodTypeForm<'gc> {
        let mut temp: OpaqueFrame<'gc, '_> = todo!();
        let method_type_form = assert_inited_or_initing_class(jvm, CClassName::method_type_form().into());
        let res_handle = AllocatedHandle::NormalObject(new_object(jvm, pushable_frame_todo()/*int_state*/, &method_type_form, false));
        let res = res_handle.cast_method_type_form();
        res.set_arg_to_slot_table(jvm, arg_to_slot_table);
        res.set_slot_to_arg_table(jvm, slot_to_arg_table);
        res.set_arg_counts(jvm, arg_counts);
        res.set_prim_counts(jvm, prim_counts);
        if let Some(x) = erased_type {
            res.set_erased_type(jvm, x);
        }
        if let Some(x) = basic_type {
            res.set_basic_type(jvm, x);
        }
        res.set_method_handles(jvm, method_handles);
        res.set_lambda_forms(jvm, lambda_forms);
        res
    }

    // as_object_or_java_value!();
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for MethodTypeForm<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        todo!()
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        todo!()
    }
}
