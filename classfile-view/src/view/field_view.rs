use wtf8::Wtf8Buf;
use classfile_parser::attribute_infos::runtime_annotations_to_bytes;
use rust_jvm_common::classfile::FieldInfo;
use rust_jvm_common::compressed_classfile::compressed_types::CPDType;
use rust_jvm_common::compressed_classfile::CompressedFieldInfo;
use rust_jvm_common::compressed_classfile::field_names::FieldName;


use crate::view::{ClassBackedView, ClassView, HasAccessFlags};
use crate::view::constant_info_view::ConstantInfoView;

pub struct FieldView<'l> {
    view: &'l ClassBackedView,
    i: u16,
}

impl FieldView<'_> {
    fn field_info_compressed(&self) -> &CompressedFieldInfo {
        &self.view.backing_class.fields[self.i as usize]
    }

    fn field_info(&self) -> &FieldInfo {
        &self.view.underlying_class.fields[self.i as usize]
    }
    pub fn field_name(&self) -> FieldName {
        FieldName(self.field_info_compressed().name)
    }
    pub fn field_desc(&self) -> String {
        self.view.underlying_class.constant_pool[self.view.underlying_class.fields[self.i as usize].descriptor_index as usize].extract_string_from_utf8().clone().into_string().expect("should have validated this earlier maybe todo")
    }
    pub fn constant_value_attribute(&self) -> Option<ConstantInfoView> {
        self.view.underlying_class.fields[self.i as usize].constant_value_attribute_i().map(|i| self.view.constant_pool_view(i as usize))
    }

    pub fn signature_attribute(&self) -> Option<Wtf8Buf> {
        self.view.underlying_class.fields[self.i as usize].signature_attribute_i().map(|i| self.view.underlying_class.constant_pool[i as usize].extract_string_from_utf8())
    }
    pub fn from(c: &ClassBackedView, i: usize) -> FieldView {
        FieldView { view: c, i: i as u16 }
    }
    pub fn field_type(&self) -> CPDType {
        self.field_info_compressed().descriptor_type.clone()
        /*PTypeView::from_ptype(&parse_field_descriptor(self.field_desc().as_str()).unwrap().field_type)*/
    }

    pub fn get_annotation_bytes(&self) -> Option<Vec<u8>> {
        self.field_info().runtime_visible_annotations().map(|annotations| runtime_annotations_to_bytes(annotations.annotations.clone()))
    }

    pub fn field_i(&self) -> u16 {
        self.i
    }
}

impl HasAccessFlags for FieldView<'_> {
    fn access_flags(&self) -> u16 {
        self.field_info_compressed().access_flags
    }
}

pub enum FieldIterator<'l> {
    ClassBacked { backing_class: &'l ClassBackedView, i: usize },
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
            FieldIterator::Empty => None,
        }
    }
}