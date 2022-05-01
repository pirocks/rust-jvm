use std::collections::HashMap;
use rust_jvm_common::MethodId;
use stage0::compiler_common::YetAnotherLayoutImpl;

pub struct LayoutCache{
    inner: HashMap<MethodId, YetAnotherLayoutImpl>
}

impl LayoutCache{
    pub fn add_entry(&mut self, method_id: MethodId, layout: YetAnotherLayoutImpl ){
        todo!()
    }
}

