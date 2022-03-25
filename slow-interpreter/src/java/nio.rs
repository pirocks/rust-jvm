pub mod heap_byte_buffer {
    use jvmti_jni_bindings::{jbyte, jint};
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{new_object, run_constructor};
    use crate::java_values::{JavaValue};
    use crate::jvm_state::JVMState;
    use crate::new_java_values::{AllocatedObject, AllocatedObjectHandle, NewJavaValueHandle, UnAllocatedObject, UnAllocatedObjectArray};
    use crate::{check_initing_or_inited_class, NewAsObjectOrJavaValue, NewJavaValue};

    pub struct HeapByteBuffer<'gc> {
        normal_object: AllocatedObjectHandle<'gc>,
    }

    impl<'gc> JavaValue<'gc> {
        pub fn cast_heap_byte_buffer(&self) -> HeapByteBuffer<'gc> {
            HeapByteBuffer { normal_object: todo!()/*self.unwrap_object_nonnull()*/ }
        }
    }

    impl<'gc> AllocatedObjectHandle<'gc> {
        pub fn cast_heap_byte_buffer(self) -> HeapByteBuffer<'gc> {
            HeapByteBuffer { normal_object: self }
        }
    }

    impl<'gc> HeapByteBuffer<'gc> {
        pub fn new<'l>(jvm: &'gc JVMState<'gc>, int_state: &'_ mut InterpreterStateGuard<'gc,'l>, buf: Vec<jbyte>, off: jint, len: jint) -> Result<Self, WasException> {
            let heap_byte_buffer_class = assert_inited_or_initing_class(jvm, CClassName::heap_byte_buffer().into());
            let object = new_object(jvm, int_state, &heap_byte_buffer_class);

            let elems = buf.into_iter().map(|byte| NewJavaValue::Byte(byte)).collect();
            let array_object = UnAllocatedObjectArray {
                whole_array_runtime_class: check_initing_or_inited_class(jvm, int_state, CPDType::array(CPDType::ByteType)).unwrap(),
                elems,
            };
            //todo what about check_inited_class for this array type
            let array = NewJavaValueHandle::Object(jvm.allocate_object(UnAllocatedObject::Array(array_object)));
            run_constructor(jvm, int_state, heap_byte_buffer_class, vec![object.new_java_value(), array.as_njv(), NewJavaValue::Int(off), NewJavaValue::Int(len)], &CMethodDescriptor::void_return(vec![CPDType::array(CPDType::ByteType), CPDType::IntType, CPDType::IntType]))?;
            Ok(object.cast_heap_byte_buffer())
        }

        // as_object_or_java_value!();
    }

    impl <'gc> NewAsObjectOrJavaValue<'gc> for HeapByteBuffer<'gc>{
        fn object(self) -> AllocatedObjectHandle<'gc> {
            self.normal_object
        }

        fn object_ref(&self) -> AllocatedObject<'gc, '_> {
            self.normal_object.as_allocated_obj()
        }
    }
}