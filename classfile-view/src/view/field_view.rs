use rust_jvm_common::compressed_classfile::{CCString, CompressedFieldInfo};
use rust_jvm_common::descriptor_parser::parse_field_descriptor;

use crate::view::{ClassBackedView, ClassView, HasAccessFlags};
use crate::view::constant_info_view::ConstantInfoView;
use crate::view::ptype_view::PTypeView;

pub struct FieldView<'l> {
    view: &'l ClassBackedView,
    i: usize,
}

impl FieldView<'_> {
    fn field_info(&self) -> &CompressedFieldInfo {
        &self.view.backing_class.fields[self.i]
    }
    pub fn field_name(&self) -> CCString {
        self.field_info().name
    }
    pub fn field_desc(&self) -> String {
        self.view.underlying_class.constant_pool[self.view.underlying_class.fields[self.i].descriptor_index as usize].extract_string_from_utf8()
    }
    pub fn constant_value_attribute(&self) -> Option<ConstantInfoView> {
        self.view.underlying_class.fields[self.i].constant_value_attribute_i().map(|i| { self.view.constant_pool_view(i as usize) })
    }
    pub fn from(c: &ClassBackedView, i: usize) -> FieldView {
        FieldView { view: c, i }
    }
    pub fn field_type(&self) -> PTypeView {
        PTypeView::from_ptype(&parse_field_descriptor(self.field_desc().as_str()).unwrap().field_type)
    }

    pub fn field_i(&self) -> usize {
        self.i
    }
}

impl HasAccessFlags for FieldView<'_> {
    fn access_flags(&self) -> u16 {
        self.field_info().access_flags
    }
}


pub enum FieldIterator<'l> {
    ClassBacked {
        backing_class: &'l ClassBackedView,
        i: usize,
    },
    Empty,
}


impl<'l> Iterator for FieldIterator<'l> {
    type Item = FieldView<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FieldIterator::ClassBacked { i, backing_class } => {
                if *i >= backing_class.num_fields() {
                    return None;
                }
                let res = FieldView::from(backing_class, *i);
                *i += 1;
                Some(res)
            }
            FieldIterator::Empty => {
                None
            }
        }
    }
}