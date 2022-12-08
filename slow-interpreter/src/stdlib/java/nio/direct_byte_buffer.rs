use jvmti_jni_bindings::{jint, jlong};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};
use crate::better_java_stack::frames::PushableFrame;
use crate::class_loading::check_initing_or_inited_class;
use crate::exceptions::WasException;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::allocated_objects::{AllocatedHandle, AllocatedNormalObjectHandle};
use crate::new_java_values::NewJavaValue;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::stdlib::java::NewAsObjectOrJavaValue;

pub struct DirectByteBuffer<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl <'gc> AllocatedHandle<'gc> {
    pub fn cast_direct_byte_buffer(self) -> DirectByteBuffer<'gc> {
        DirectByteBuffer{
            normal_object: self.normal_object(),
        }
    }
}

impl<'gc> DirectByteBuffer<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, address: jlong, cap: jint) -> Result<Self, WasException<'gc>> {
        let direct_byte_buffer_class = check_initing_or_inited_class(jvm, int_state, CClassName::direct_byte_buffer().into())?;
        let object = new_object_full(jvm, int_state, &direct_byte_buffer_class);


        let full_args = vec![object.new_java_value(), NewJavaValue::Long(address), NewJavaValue::Int(cap)];
        let desc = CMethodDescriptor::void_return(vec![CPDType::LongType, CPDType::IntType]);
        run_constructor(jvm, int_state, direct_byte_buffer_class, full_args, &desc)?;
        Ok(object.cast_direct_byte_buffer())
    }

    // as_object_or_java_value!();
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for DirectByteBuffer<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}

