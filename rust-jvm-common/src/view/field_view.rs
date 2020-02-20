use crate::classfile::{Classfile, FieldInfo};
use std::sync::Arc;
use crate::view::HasAccessFlags;
use crate::view::constant_info_view::ConstantInfoView;

pub struct FieldView {
    backing_class: Arc<Classfile>,
    i: usize,
}

impl FieldView {
    fn field_info(&self) -> &FieldInfo {
        &self.backing_class.fields[self.i]
    }
    pub fn field_name(&self) -> String {
        self.field_info().name(&self.backing_class)
    }
    pub fn field_desc(&self) -> String {
        self.backing_class.constant_pool[self.field_info().descriptor_index as usize].extract_string_from_utf8()
    }
    pub fn constant_value_attribute(&self) -> ConstantInfoView {
        unimplemented!()
//            self.field_info().constant_value_attribute_i().map(|i| { self.backing_class.constant_pool[i as usize] })
    }
}

impl HasAccessFlags for FieldView {
    fn access_flags(&self) -> u16 {
        self.field_info().access_flags
    }
}
