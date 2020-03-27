pub mod invoke;

pub mod member_name {
    use crate::java_values::NormalObject;
    use crate::java::lang::string::JString;
    use crate::instructions::invoke::native::mhn_temp::run_static_or_virtual;

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
        pub fn get_name() -> JString{
            run_static_or_virtual()
        }
    }
}

pub mod string {
    use crate::java_values::NormalObject;
    use crate::utils::string_obj_to_string;
    use crate::java_values::Object::Object;
    use std::sync::Arc;

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
            string_obj_to_string(Arc::new(Object(self.normal_object.clone())).into())
        }
    }
}

pub mod reflect;