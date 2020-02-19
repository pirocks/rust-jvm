use std::sync::Arc;
use rust_jvm_common::classnames::ClassName;

pub struct ClassView{
    backing_class:Arc<ClassFile>
}

impl ClassView{
    fn name(&self)-> ClassName{
        unimplemented!()
    }
}