pub mod invoke;

pub mod member_name {
    use crate::java_values::NormalObject;

    pub struct MemberName{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_member_name(&self) -> MemberName{
            MemberName { normal_object: self.clone() }
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