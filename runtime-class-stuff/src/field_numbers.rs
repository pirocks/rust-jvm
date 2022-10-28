use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

use itertools::Itertools;

use classfile_view::view::{ClassBackedView, ClassView, HasAccessFlags};
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CPDType};
use rust_jvm_common::compressed_classfile::field_names::FieldName;

use crate::{FieldNameAndFieldType, FieldNumberAndFieldType, RuntimeClass};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FieldNameAndClass {
    pub field_name: FieldName,
    pub class_name: CClassName,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FieldNumber(pub u32);

impl FieldNumber {
    fn new(from: u32) -> Self {
        Self(from)
    }

    fn is_static() -> bool {
        false
    }
}

pub struct FieldNumbers {
    numbers: HashMap<FieldNameAndClass, (FieldNumber, CPDType)>,
    canonical_numbers: HashMap<FieldNameAndClass, (FieldNumber, CPDType)>,
}

fn reverse_fields(field_numbers: HashMap<FieldNameAndClass, (FieldNumber, CPDType)>) -> (HashMap<FieldNameAndClass, FieldNumberAndFieldType>, HashMap<FieldNumber, FieldNameAndFieldType>) {
    let reverse: HashMap<FieldNumber, FieldNameAndFieldType> = field_numbers.clone().into_iter()
        .map(|(name, (number, cpdtype))| (number, FieldNameAndFieldType { name, cpdtype }))
        .collect();
    let forward: HashMap<FieldNameAndClass, FieldNumberAndFieldType> = field_numbers.into_iter()
        .map(|(name, (number, cpdtype))| (name, FieldNumberAndFieldType { number, cpdtype }))
        .collect();
    assert_eq!(forward.len(), reverse.len());
    (forward, reverse)
}

impl FieldNumbers {
    pub fn reverse_fields(self) -> (HashMap<FieldNameAndClass, FieldNumberAndFieldType>, HashMap<FieldNumber, FieldNameAndFieldType>) {
        let Self { numbers, canonical_numbers } = self;
        let reverse: HashMap<FieldNumber, FieldNameAndFieldType> = canonical_numbers.into_iter()
            .map(|(name, (number, cpdtype))| (number, FieldNameAndFieldType { name, cpdtype }))
            .collect();
        let forward: HashMap<FieldNameAndClass, FieldNumberAndFieldType> = numbers.into_iter()
            .map(|(name, (number, cpdtype))| (name, FieldNumberAndFieldType { number, cpdtype }))
            .collect();
        (forward, reverse)
    }
}

fn get_field_numbers_impl(class_view: Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> FieldNumbers {
    let mut temp_vec = vec![];
    get_fields(class_view.deref(), parent, FieldNumber::is_static(), &mut temp_vec);
    let mut field_number = 0;
    let mut numbers = HashMap::new();
    let mut canonical_numbers = HashMap::new();
    for (i, (class_name, fields)) in temp_vec.iter().into_iter().enumerate() {
        let class_name = *class_name;
        let subclasses = &temp_vec[i..];
        for (field_name, cpdtype) in fields.into_iter().sorted_by_key(|(name, _ptype)| *name).cloned() {
            let field_name_and_class = FieldNameAndClass { field_name, class_name };
            canonical_numbers.insert(field_name_and_class, (FieldNumber::new(field_number), cpdtype));
            for (subclass, _) in subclasses {
                numbers.insert(FieldNameAndClass { field_name, class_name: *subclass }, (FieldNumber::new(field_number), cpdtype));
            }
            field_number += 1;
        }
    }
    FieldNumbers { numbers, canonical_numbers }
}

pub fn get_field_numbers(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> FieldNumbers {
    get_field_numbers_impl(class_view.clone(), parent)
}


pub(crate) fn get_fields(class_view: &dyn ClassView, parent: &Option<Arc<RuntimeClass>>, static_: bool, res: &mut Vec<(CClassName, Vec<(FieldName, CPDType)>)>) {
    if let Some(parent) = parent.as_ref() {
        get_fields(parent.unwrap_class_class().class_view.deref(), &parent.unwrap_class_class().parent, static_, res);
    };

    let class_name = class_view.name().unwrap_name();
    res.push((class_name, class_view.fields().filter(|field| {
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
        .collect_vec()));
}
