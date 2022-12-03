use jvmti_jni_bindings::{jbyte, jint};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CMethodDescriptor, CPDType};


use crate::{check_initing_or_inited_class, NewAsObjectOrJavaValue, NewJavaValue, PushableFrame, UnAllocatedObject, WasException};
use crate::class_loading::assert_inited_or_initing_class;
use crate::interpreter_util::{new_object_full, run_constructor};
use crate::jvm_state::JVMState;
use crate::new_java_values::NewJavaValueHandle;
use crate::new_java_values::allocated_objects::AllocatedNormalObjectHandle;
use crate::new_java_values::java_value_common::JavaValueCommon;
use crate::new_java_values::owned_casts::OwnedCastAble;
use crate::new_java_values::unallocated_objects::UnAllocatedObjectArray;

pub struct HeapByteBuffer<'gc> {
    pub(crate) normal_object: AllocatedNormalObjectHandle<'gc>,
}

impl<'gc> HeapByteBuffer<'gc> {
    pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &mut impl PushableFrame<'gc>, buf: Vec<jbyte>, off: jint, len: jint) -> Result<Self, WasException<'gc>> {
        let heap_byte_buffer_class = assert_inited_or_initing_class(jvm, CClassName::heap_byte_buffer().into());
        let object = new_object_full(jvm, int_state, &heap_byte_buffer_class);

        let elems = buf.into_iter().map(|byte| NewJavaValue::Byte(byte)).collect();
        let array_object = UnAllocatedObjectArray {
            whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::ByteType)).unwrap(),
            elems,
        };
        //todo what about check_inited_class for this array type
        let array = NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(array_object)));
        let full_args = vec![object.new_java_value(), array.as_njv(), NewJavaValue::Int(off), NewJavaValue::Int(len)];
        let desc = CMethodDescriptor::void_return(vec![CPDType::array(CPDType::ByteType), CPDType::IntType, CPDType::IntType]);
        run_constructor(jvm, int_state, heap_byte_buffer_class, full_args, &desc)?;
        Ok(object.cast_heap_byte_buffer())
    }

    // as_object_or_java_value!();
}

impl<'gc> NewAsObjectOrJavaValue<'gc> for HeapByteBuffer<'gc> {
    fn object(self) -> AllocatedNormalObjectHandle<'gc> {
        self.normal_object
    }

    fn object_ref(&self) -> &'_ AllocatedNormalObjectHandle<'gc> {
        &self.normal_object
    }
}
