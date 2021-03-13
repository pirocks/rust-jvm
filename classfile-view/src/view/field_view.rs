use rust_jvm_common::classfile::FieldInfo;
use rust_jvm_common::descriptor_parser::parse_field_descriptor;

use crate::view::{ClassBackedView, ClassView, HasAccessFlags};
use crate::view::constant_info_view::ConstantInfoView;
use crate::view::ptype_view::PTypeView;

pub struct FieldView<'l> {
    view: &'l ClassBackedView,
    i: usize,
}

impl FieldView<'_> {
    fn field_info(&self) -> &FieldInfo {
        &self.view.backing_class.fields[self.i]
    }
    pub fn field_name(&self) -> String {
        self.field_info().name(&self.view.backing_class)
    }
    pub fn field_desc(&self) -> String {
        self.view.backing_class.constant_pool[self.field_info().descriptor_index as usize].extract_string_from_utf8()
    }
    pub fn constant_value_attribute(&self) -> Option<ConstantInfoView> {
        self.field_info().constant_value_attribute_i().map(|i| { self.view.constant_pool_view(i as usize) })
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