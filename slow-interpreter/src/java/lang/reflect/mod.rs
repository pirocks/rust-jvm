pub mod method {
    use std::sync::Arc;

    use crate::java_values::{JavaValue, Object};
    use crate::java::lang::class::JClass;
    use jvmti_jni_bindings::jint;

    pub struct Method {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_method(&self) -> Method {
            Method { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Method {
        pub fn init() -> Self {
            unimplemented!()
        }

        pub fn get_clazz(&self) -> JClass{
            self.normal_object.lookup_field("clazz").cast_class()
        }

        pub fn get_modifiers(&self) -> jint{
            self.normal_object.lookup_field("modifiers").unwrap_int()
        }

        as_object_or_java_value!();
    }
}

pub mod field {
    use std::sync::Arc;

    use classfile_view::view::ptype_view::PTypeView;
    use jvmti_jni_bindings::jint;
    use rust_jvm_common::classnames::ClassName;

    use crate::{InterpreterStateGuard, JVMState};
    use crate::interpreter_util::{check_inited_class, push_new_object, run_constructor};
    use crate::java::lang::class::JClass;
    use crate::java::lang::string::JString;
    use crate::java_values::{ArrayObject, JavaValue, Object};

    pub struct Field {
        normal_object: Arc<Object>
    }

    impl JavaValue {
        pub fn cast_field(&self) -> Field {
            Field { normal_object: self.unwrap_object_nonnull() }
        }
    }

    impl Field {
        pub fn init<'l>(
            jvm: &JVMState,
            int_state: &mut InterpreterStateGuard,
            clazz: JClass,
            name: JString,
            type_: JClass,
            modifiers: jint,
            slot: jint,
            signature: JString,
            annotations: Vec<JavaValue>,
        ) -> Self {
            let field_classfile = check_inited_class(jvm, int_state, &ClassName::field().into(), int_state.current_loader(jvm));
            push_new_object(jvm, int_state, &field_classfile, None);
            let field_object = int_state.pop_current_operand_stack();


            let modifiers = JavaValue::Int(modifiers);
            let slot = JavaValue::Int(slot);

            //todo impl annotations.
            let annotations = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
                jvm,
                int_state,
                annotations,
                PTypeView::ByteType,
                jvm.thread_state.new_monitor("monitor for annotations array".to_string()),
            )))));

            run_constructor(
                jvm,
                int_state,
                field_classfile.clone(),
                vec![field_object.clone(), clazz.java_value(), name.java_value(), type_.java_value(), modifiers, slot, signature.java_value(), annotations],
                "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V".to_string(),
            );
            field_object.cast_field()
        }

        pub fn name(&self) -> JString {
            self.normal_object.lookup_field("name").cast_string()
        }

        pub fn clazz(&self) -> JClass {
            self.normal_object.lookup_field("clazz").cast_class()
        }

        as_object_or_java_value!();
    }
}