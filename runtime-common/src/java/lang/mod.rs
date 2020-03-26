


pub mod method_type {
    use crate::java_values::NormalObject;

    pub struct MethodType{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_method_type(&self) -> MethodType{
            MethodType { normal_object: self.clone() }
        }
    }
}

pub mod member_name {
    use crate::java_values::NormalObject;

    pub struct MemberName{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_method_type(&self) -> MemberName{
            MemberName { normal_object: self.clone() }
        }
    }
}

pub mod reflect;