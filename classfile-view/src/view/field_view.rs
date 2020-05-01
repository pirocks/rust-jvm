use rust_jvm_common::classfile::{Classfile, FieldInfo};
use std::sync::Arc;
use crate::view::{HasAccessFlags, ClassView};
use crate::view::constant_info_view::ConstantInfoView;
use crate::view::ptype_view::PTypeView;
use descriptor_parser::parse_field_descriptor;

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
    pub fn constant_value_attribute(&self) -> Option<ConstantInfoView> {
        unimplemented!()
//            self.field_info().constant_value_attribute_i().map(|i| { self.backing_class.constant_pool[i as usize] })
    }
    pub fn from(c: &ClassView, i: usize) -> FieldView {
        FieldView { backing_class: c.backing_class.clone(), i }
    }
    pub fn field_type(&self) -> PTypeView{
        PTypeView::from_ptype(&parse_field_descriptor(self.field_desc().as_str()).unwrap().field_type)
    }
    pub fn fields(&self) -> FieldIterator{
        unimplemented!()
    }
}

impl HasAccessFlags for FieldView {
    fn access_flags(&self) -> u16 {
        self.field_info().access_flags
    }
}


pub struct FieldIterator<'l> {
    //todo create a from and remove pub(crate)
    pub(crate) backing_class: &'l ClassView,
    pub(crate) i: usize,
}


impl Iterator for FieldIterator<'_> {
    type Item = FieldView;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.backing_class.num_fields() {
            return None;
        }
        let res = FieldView::from(self.backing_class, self.i);
        self.i += 1;
        Some(res)
    }
}