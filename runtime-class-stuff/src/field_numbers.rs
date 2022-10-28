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

use crate::{FieldNameAndFieldType, FieldNumberAndFieldType, FieldNumberAndFieldTypeImpl, RuntimeClass};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct FieldNameAndClass {
    pub field_name: FieldName,
    pub class_name: CClassName,
}

pub trait StaticOrNormalFieldNumber : Copy + Clone + Eq + PartialEq + Debug + Hash {
    fn new(from: u32) -> Self;
    fn is_static() -> bool;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FieldNumber(pub u32);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StaticFieldNumber(pub u32);

impl StaticOrNormalFieldNumber for FieldNumber {
    fn new(from: u32) -> Self {
        Self(from)
    }

    fn is_static() -> bool {
        false
    }
}

impl StaticOrNormalFieldNumber for StaticFieldNumber {
    fn new(from: u32) -> Self {
        Self(from)
    }

    fn is_static() -> bool {
        true
    }
}

pub struct FieldNumbers<T: StaticOrNormalFieldNumber> {
    numbers: HashMap<FieldNameAndClass, (T, CPDType)>,
    canonical_numbers: HashMap<FieldNameAndClass, (T, CPDType)>,
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

impl<T: StaticOrNormalFieldNumber> FieldNumbers<T> {
    pub fn reverse_fields(self) -> (HashMap<FieldNameAndClass, FieldNumberAndFieldTypeImpl<T>>, HashMap<T, FieldNameAndFieldType>) {
        let Self { numbers, canonical_numbers } = self;
        let reverse: HashMap<T, FieldNameAndFieldType> = canonical_numbers.into_iter()
            .map(|(name, (number, cpdtype))| (number, FieldNameAndFieldType { name, cpdtype }))
            .collect();
        let forward: HashMap<FieldNameAndClass, FieldNumberAndFieldTypeImpl<T>> = numbers.into_iter()
            .map(|(name, (number, cpdtype))| (name, FieldNumberAndFieldTypeImpl { number, cpdtype }))
            .collect();
        (forward, reverse)
    }
}

fn get_field_numbers_impl<T: StaticOrNormalFieldNumber>(class_view: Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> FieldNumbers<T> {
    let mut temp_vec = vec![];
    get_fields(class_view.deref(), parent, T::is_static(), &mut temp_vec);
    let mut field_number = 0;
    let mut numbers = HashMap::new();
    let mut canonical_numbers = HashMap::new();
    for (i, (class_name, fields)) in temp_vec.iter().into_iter().enumerate() {
        let class_name = *class_name;
        let subclasses = &temp_vec[i..];
        for (field_name, cpdtype) in fields.into_iter().sorted_by_key(|(name, _ptype)| *name).cloned() {
            let field_name_and_class = FieldNameAndClass { field_name, class_name };
            canonical_numbers.insert(field_name_and_class, (T::new(field_number), cpdtype));
            for (subclass, _) in subclasses {
                numbers.insert(FieldNameAndClass { field_name, class_name: *subclass }, (T::new(field_number), cpdtype));
            }
            field_number += 1;
        }
    }
    FieldNumbers { numbers, canonical_numbers }
}

pub fn get_field_numbers_static(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> FieldNumbers<StaticFieldNumber> {
    get_field_numbers_impl(class_view.clone(), parent)
}

pub fn get_field_numbers(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass>>) -> FieldNumbers<FieldNumber> {
    get_field_numbers_impl(class_view.clone(), parent)
}


fn get_fields(class_view: &dyn ClassView, parent: &Option<Arc<RuntimeClass>>, static_: bool, res: &mut Vec<(CClassName, Vec<(FieldName, CPDType)>)>) {
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
