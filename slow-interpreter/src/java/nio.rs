pub mod heap_byte_buffer {
    use jvmti_jni_bindings::{jbyte, jint};
    use rust_jvm_common::compressed_classfile::{CMethodDescriptor, CPDType};
    use rust_jvm_common::compressed_classfile::names::CClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter::WasException;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{ArrayObject, GcManagedObject, JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct HeapByteBuffer<'gc_life> {
        normal_object: GcManagedObject<'gc_life>,
    }

    impl<'gc_life> JavaValue<'gc_life> {
        pub fn cast_heap_byte_buffer(&self) -> HeapByteBuffer<'gc_life> {
            HeapByteBuffer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl<'gc_life> HeapByteBuffer<'gc_life> {
        pub fn new(jvm: &'gc_life JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, 'l>, buf: Vec<jbyte>, off: jint, len: jint) -> Result<Self, WasException> {
            let heap_byte_buffer_class = assert_inited_or_initing_class(jvm, CClassName::heap_byte_buffer().into());
            push_new_object(jvm, int_state, &heap_byte_buffer_class);
            let object = int_state.pop_current_operand_stack(Some(CClassName::object().into()));

            let elems = buf.into_iter().map(|byte| JavaValue::Byte(byte)).collect();
            let array_object = ArrayObject::new_array(jvm, int_state, elems, CPDType::ByteType, jvm.thread_state.new_monitor("heap bytebuffer array monitor".to_string()))?;
            let array = JavaValue::Object(Some(jvm.allocate_object(Object::Array(array_object))));
            run_constructor(jvm, int_state, heap_byte_buffer_class, vec![object.clone(), array, JavaValue::Int(off), JavaValue::Int(len)],
                            &CMethodDescriptor::void_return(vec![CPDType::array(CPDType::ByteType), CPDType::IntType, CPDType::IntType]))?;
            Ok(object.cast_heap_byte_buffer())
        }

        as_object_or_java_value!();
    }
}