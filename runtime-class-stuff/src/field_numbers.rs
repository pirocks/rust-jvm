use std::collections::HashMap;
use std::sync::Arc;
use itertools::Itertools;
use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::CompressedParsedDescriptorType;
use rust_jvm_common::compressed_classfile::names::FieldName;
use crate::RuntimeClass;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FieldNumber(pub u32);

pub fn get_field_numbers(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> (u32, HashMap<FieldName, (FieldNumber, CompressedParsedDescriptorType)>) {
    get_field_numbers_impl(class_view, parent, false)
}

pub fn get_field_numbers_static(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> (u32, HashMap<FieldName, (FieldNumber, CompressedParsedDescriptorType)>) {
    get_field_numbers_impl(class_view, parent, true)
}

pub fn get_field_numbers_impl(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>, static_: bool) -> (u32, HashMap<FieldName, (FieldNumber, CompressedParsedDescriptorType)>) {
    let start_field_number = parent.as_ref().map(|parent| parent.unwrap_class_class().num_vars()).unwrap_or(0);
    let field_numbers = class_view.fields().filter(|field| {
        let is_static = field.is_static();
        if static_ {
            is_static
        } else {
            !is_static
        }
    })
        .map(|name| (name.field_name(), name.field_type()))
        .sorted_by_key(|(name, _ptype)| name.0)
        .enumerate()
        .map(|(index, (name, ptype))| (name, (FieldNumber((index + start_field_number) as u32), ptype))).collect::<HashMap<_, _>>();
    ((start_field_number + field_numbers.len()) as u32, field_numbers)
}
