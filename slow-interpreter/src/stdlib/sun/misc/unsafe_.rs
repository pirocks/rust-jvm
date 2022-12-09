use std::ops::Deref;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;
use rust_jvm_common::compressed_classfile::method_names::MethodName;


use crate::{JVMState, NewAsObjectOrJavaValue, NewJavaValueHandle, WasException};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::assert_inited_or_initing_class;
use crate::java_values::JavaValue;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::static_vars::static_vars;
use crate::stdlib::java::lang::reflect::field::Field;
use crate::utils::run_static_or_virtual;

pub struct Unsafe<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> JavaValue<'gc> {
    pub fn cast_unsafe(&self) -> Unsafe<'gc> {
        Unsafe { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
    }
}

impl<'gc> Unsafe<'gc> {
    pub fn the_unsafe<'l>(jvm: &'gc JVMState<'gc>, _int_state: &mut impl PushableFrame<'gc>) -> Unsafe<'gc> {
        let unsafe_class = assert_inited_or_initing_class(jvm, CClassName::unsafe_().into());
        let static_vars = static_vars(unsafe_class.deref(), jvm);
        static_vars.get(FieldName::field_theUnsafe(),CPDType::object()).cast_unsafe()
    }

    pub fn object_field_offset<'l>(&self, jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, field: Field<'gc>) -> Result<NewJavaValueHandle<'gc>, WasException<'gc>> {
        let unsafe_class = assert_inited_or_initing_class(jvm, CClassName::unsafe_().into());
        let desc = CMethodDescriptor { arg_types: vec![CClassName::field().into()], return_type: CPDType::LongType };
        let args = vec![self.normal_object.new_java_value(), field.new_java_value()];
        let res = run_static_or_virtual(jvm, int_state, &unsafe_class, MethodName::method_objectFieldOffset(), &desc, args)?;
        Ok(res.unwrap())
    }
}
