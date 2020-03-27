pub mod invoke;

pub mod member_name {
    use crate::java_values::{NormalObject, JavaValue};
    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;
    use crate::{InterpreterState, StackEntry};
    use std::rc::Rc;
    use crate::interpreter_util::check_inited_class;
    use rust_jvm_common::classnames::ClassName;
    use std::sync::Arc;
    use crate::java_values::Object::Object;
    use crate::java::lang::class::JClass;
    use crate::java::lang::invoke::method_type::MethodType;

    pub struct MemberName{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_member_name(&self) -> MemberName{
            MemberName { normal_object: self.clone() }
        }
    }

    impl MemberName{
        // private Class<?> clazz;
        // private String name;
        // private Object type;
        // private int flags;
        pub fn get_name(&self,state: &mut InterpreterState, frame: Rc<StackEntry>) -> JString{
            let member_name_class = check_inited_class(state,&ClassName::member_name(),frame.clone().into(),frame.class_pointer.loader.clone());
            frame.push(JavaValue::Object(Arc::new(Object(self.normal_object.clone())).into()));
            run_static_or_virtual(state,&frame,&member_name_class,"getName".to_string(),"()Ljava/lang/String;".to_string());
            frame.pop().unwrap_normal_object().cast_string()
        }

        pub fn clazz(&self) -> JClass{
            self.normal_object.fields.borrow().get("clazz").unwrap().unwrap_normal_object().cast_class()
        }

        pub fn get_method_type(&self,state: &mut InterpreterState, frame: Rc<StackEntry>) -> MethodType {
            let member_name_class = check_inited_class(state,&ClassName::member_name(),frame.clone().into(),frame.class_pointer.loader.clone());
            frame.push(JavaValue::Object(Arc::new(Object(self.normal_object.clone())).into()));
            run_static_or_virtual(state,&frame,&member_name_class,"getMethodType".to_string(),"()Ljava/lang/invoke/MethodType;".to_string());
            frame.pop().unwrap_normal_object().cast_method_type()
        }

        pub fn get_field_type(&self, state: &mut InterpreterState, frame: Rc<StackEntry>) -> JClass{
            let member_name_class = check_inited_class(state,&ClassName::member_name(),frame.clone().into(),frame.class_pointer.loader.clone());
            frame.push(JavaValue::Object(Arc::new(Object(self.normal_object.clone())).into()));
            run_static_or_virtual(state,&frame,&member_name_class,"getFieldType".to_string(),"()Ljava/lang/Class;".to_string());
            frame.pop().unwrap_normal_object().cast_class()
        }

    }
}

#[macro_use]
pub mod class{
    use crate::java_values::{NormalObject, JavaValue};
    use classfile_view::view::ptype_view::PTypeView;
    use crate::java_values::Object::Object;

    pub struct JClass {
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_class(&self) -> JClass {
            JClass { normal_object: self.clone() }
        }
    }

    impl JClass{

        pub fn as_type(&self) -> PTypeView{
            self.normal_object.class_object_ptype.borrow().as_ref().unwrap().clone()
        }

        as_object_or_java_value!();
    }
}

pub mod string {
    use crate::java_values::NormalObject;
    use crate::utils::string_obj_to_string;
    use crate::java_values::Object;
    use std::sync::Arc;
    use crate::java_values::JavaValue;
    use crate::instructions::ldc::create_string_on_stack;
    use crate::{InterpreterState, StackEntry};
    use std::rc::Rc;

    pub struct JString {
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_string(&self) -> JString {
            JString { normal_object: self.clone() }
        }
    }

    impl JString {
        pub fn to_rust_string(&self) -> String {
            string_obj_to_string(Arc::new(Object::Object(self.normal_object.clone())).into())
        }

        pub fn from(state: &mut InterpreterState, current_frame: &Rc<StackEntry>,rust_str: String) -> JString{
            create_string_on_stack(state,current_frame,rust_str);
            current_frame.pop().unwrap_normal_object().cast_string()
        }

        as_object_or_java_value!();
    }
}

pub mod reflect;