use std::collections::HashMap;
use rust_jvm_common::MethodId;
use crate::ir_to_java_layer::compiler::YetAnotherLayoutImpl;

pub struct LayoutCache{
    inner: HashMap<MethodId, YetAnotherLayoutImpl>
}

impl LayoutCache{
    pub fn add_entry(&mut self, method_id: MethodId, layout: YetAnotherLayoutImpl ){
        todo!()
    }
}

