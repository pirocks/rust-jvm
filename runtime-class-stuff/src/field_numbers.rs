use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

use itertools::Itertools;

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;

use crate::RuntimeClass;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FieldNameAndClass {
    pub field_name: FieldName,
    pub class_name: CClassName,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FieldNumber(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StaticFieldNumber(pub u32);

pub fn get_field_numbers(class_view: Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> HashMap<FieldNameAndClass, (FieldNumber, CPDType)> {
    let mut temp_vec = vec![];
    get_field_numbers_impl_impl(class_view.deref(), parent, false, &mut temp_vec);
    temp_vec
        .into_iter()
        .enumerate()
        .map(|(index, (name, ptype))| (name, (FieldNumber(index as u32), ptype))).collect::<HashMap<_, _>>()
}

pub fn get_field_numbers_static(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> HashMap<FieldNameAndClass, (StaticFieldNumber, CPDType)> {
    let mut temp_vec = vec![];
    get_field_numbers_impl_impl(class_view.deref(), parent, true, &mut temp_vec);
    temp_vec.into_iter()
        .enumerate()
        .map(|(index, (name, ptype))| (name, (StaticFieldNumber(index as u32), ptype))).collect::<HashMap<_, _>>()
}

fn get_field_numbers_impl_impl(class_view: &dyn ClassView, parent: &Option<Arc<RuntimeClass>>, static_: bool, res: &mut Vec<(FieldNameAndClass, CPDType)>) {
    if let Some(parent) = parent.as_ref() {
        get_field_numbers_impl_impl(parent.unwrap_class_class().class_view.deref(), &parent.unwrap_class_class().parent, static_, res);
    };

    let class_name = class_view.name().unwrap_name();
    res.extend(class_view.fields().filter(|field| {
        let is_static = field.is_static();
        if static_ {
            is_static
        } else {
            !is_static
        }
    })
        .map(|name| {
            let field_name = name.field_name();
            (field_name, name.field_type())
        })
        .sorted_by_key(|(name, _ptype)| *name)
        .map(|(field_name, cpdtype)| (FieldNameAndClass {
            field_name,
            class_name,
        }, cpdtype)));
}
