pub mod method {
    use crate::java_values::NormalObject;

    pub struct Method{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_method(&self) -> Method{
            Method { normal_object: self.clone() }
        }
    }
    use crate::java_values::JavaValue;
    use std::sync::Arc;

    impl Method{
        pub fn init() ->  Self{
            unimplemented!()
        }


        as_object_or_java_value!();
    }
}

pub mod field {
    use crate::java_values::{NormalObject, JavaValue, Object, ArrayObject};
    use crate::java::lang::string::JString;
    use crate::java::lang::class::JClass;
    use jni_bindings::{jint, jbyte};
    use crate::{InterpreterState, StackEntry};
    use std::rc::Rc;
    use crate::interpreter_util::{push_new_object, run_constructor, check_inited_class};
    use crate::instructions::ldc::{load_class_constant_by_type, create_string_on_stack};
    use descriptor_parser::parse_field_descriptor;
    use std::sync::Arc;
    use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
    use std::cell::RefCell;
    use rust_jvm_common::classnames::ClassName;

    pub struct Field{
        normal_object : NormalObject
    }

    impl NormalObject{
        pub fn cast_field(&self) -> Field{
            Field { normal_object: self.clone() }
        }
    }

    impl Field{
        pub fn init(state :&mut InterpreterState, frame: &Rc<StackEntry>, clazz:JClass, name: JString, type_ :JClass, modifiers: jint, slot: jint, signature : JString, annotations: Vec<JavaValue>) -> Self {
            let field_classfile = check_inited_class(state, &ClassName::field(), frame.clone().into(), frame.class_pointer.loader.clone());
            push_new_object(state,frame.clone(), &field_classfile);
            let field_object = frame.pop();


            let modifiers = JavaValue::Int(modifiers);
            let slot = JavaValue::Int(slot);

            //todo impl annotations.
            let annotations = JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(annotations), elem_type: PTypeView::ByteType }))));

            run_constructor(
                state,
                frame.clone(),
                field_classfile.clone(),
                vec![field_object.clone(), clazz.java_value(), name.java_value(), type_.java_value(), modifiers, slot, signature.java_value(), annotations],
                "(Ljava/lang/Class;Ljava/lang/String;Ljava/lang/Class;IILjava/lang/String;[B)V".to_string(),
            );
            dbg!(&field_object);
            field_object.unwrap_normal_object().cast_field()
        }

        as_object_or_java_value!();
    }
}