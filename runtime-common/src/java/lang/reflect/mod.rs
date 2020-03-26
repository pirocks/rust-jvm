pub mod method {
    use crate::java_values::NormalObject;

    pub struct Method{
        normal_object: NormalObject
    }

    impl NormalObject{
        pub fn cast_method_type(&self) -> Method{
            Method { normal_object: self.clone() }
        }
    }

    impl Method{
        pub fn init() ->  Self{
            unimplemented!()
        }
        pub fn object(self) -> NormalObject {
            self.normal_object
        }
    }
}