


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
        pub fn cast_member_name(&self) -> MemberName{
            MemberName { normal_object: self.clone() }
        }
    }
}

pub mod string {
    use crate::java_values::NormalObject;

    pub struct String{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_string(&self) -> String{
            String { normal_object: self.clone() }
        }
    }

    impl String{
        pub fn to_rust_string(&self) -> String{
            unimplemented!()
            // string_obj_to_string()
        }
    }
}

pub mod reflect;