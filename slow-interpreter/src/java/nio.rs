pub mod heap_byte_buffer {
    use std::sync::Arc;

    use classfile_view::view::ptype_view::PTypeView;
    use jvmti_jni_bindings::{jbyte, jint};
    use rust_jvm_common::classnames::ClassName;

    use crate::class_loading::assert_inited_or_initing_class;
    use crate::interpreter_state::InterpreterStateGuard;
    use crate::interpreter_util::{push_new_object, run_constructor};
    use crate::java_values::{ArrayObject, JavaValue, Object};
    use crate::jvm_state::JVMState;

    pub struct HeapByteBuffer {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_heap_byte_buffer(&self) -> HeapByteBuffer {
            HeapByteBuffer { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl HeapByteBuffer {
        pub fn new(jvm: &JVMState, int_state: &mut InterpreterStateGuard, buf: Vec<jbyte>, off: jint, len: jint) -> Self {
            let heap_byte_buffer_class = assert_inited_or_initing_class(jvm, int_state, ClassName::Str("java/nio/HeapByteBuffer".to_string()).into());
            push_new_object(jvm, int_state, &heap_byte_buffer_class);
            let thread_object = int_state.pop_current_operand_stack();

            let elems = buf.into_iter().map(|byte| JavaValue::Byte(byte)).collect();
            let array_object = ArrayObject::new_array(jvm, int_state, elems, PTypeView::ByteType, jvm.thread_state.new_monitor("heap bytebuffer array monitor".to_string()));
            let array = JavaValue::Object(Some(Arc::new(Object::Array(array_object))));
            run_constructor(jvm, int_state, heap_byte_buffer_class, vec![thread_object.clone(), array, JavaValue::Int(off), JavaValue::Int(len)],
                            "([BII)V".to_string());
            thread_object.cast_heap_byte_buffer()
        }

        as_object_or_java_value!();
    }
}