use std::collections::HashMap;
use std::sync::Arc;

use classfile_view::view::{ClassBackedView, ClassView};
use rust_jvm_common::compressed_classfile::{CompressedParsedRefType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};

use crate::{FieldNameAndFieldType, FieldNumber, FieldNumberAndFieldType, get_field_numbers, RuntimeClass};
use crate::hidden_fields::{HiddenJVMField, HiddenJVMFieldAndFieldType};

pub struct ObjectLayout {
    hidden_field_numbers: HashMap<HiddenJVMField, FieldNumberAndFieldType>,
    hidden_field_numbers_reverse: HashMap<FieldNumber, HiddenJVMFieldAndFieldType>,
    pub field_numbers: HashMap<FieldName, FieldNumberAndFieldType>,
    field_numbers_reverse: HashMap<FieldNumber, FieldNameAndFieldType>,
    recursive_num_fields: u32,
}


fn reverse_fields(field_numbers: HashMap<FieldName, (FieldNumber, CPDType)>) -> (HashMap<FieldName, FieldNumberAndFieldType>, HashMap<FieldNumber, FieldNameAndFieldType>) {
    let reverse = field_numbers.clone().into_iter()
        .map(|(name, (number, cpdtype))| (number, FieldNameAndFieldType { name, cpdtype }))
        .collect();
    let forward = field_numbers.into_iter()
        .map(|(name, (number, cpdtype))| (name, FieldNumberAndFieldType { number, cpdtype }))
        .collect();
    (forward, reverse)
}

fn reverse_hidden_fields(hidden_field_numbers_reverse: &HashMap<FieldNumber, HiddenJVMFieldAndFieldType>) -> HashMap<HiddenJVMField, FieldNumberAndFieldType> {
    hidden_field_numbers_reverse.clone().into_iter()
        .map(|(number, HiddenJVMFieldAndFieldType{ name, cpdtype })| (name, FieldNumberAndFieldType { number, cpdtype }))
        .collect()
}



impl ObjectLayout {
    pub fn new<'gc>(class_view: &Arc<ClassBackedView>, parent: &Option<Arc<RuntimeClass<'gc>>>) -> Self {
        let (mut recursive_num_fields, field_numbers) = get_field_numbers(&class_view, &parent);
        let (field_numbers, field_numbers_reverse) = reverse_fields(field_numbers);
        let hidden_fields = if class_view.name() == CompressedParsedRefType::Class(CClassName::class()) {
            HiddenJVMField::class_hidden_fields()
        } else {
            vec![]
        };

        let hidden_field_numbers_reverse: HashMap<FieldNumber, HiddenJVMFieldAndFieldType> = hidden_fields.into_iter().map(|HiddenJVMFieldAndFieldType { name, cpdtype }| {
            let field_number = FieldNumber(recursive_num_fields);
            recursive_num_fields += 1;
            (field_number, HiddenJVMFieldAndFieldType { name, cpdtype })
        }).collect();

        let hidden_field_numbers = reverse_hidden_fields(&hidden_field_numbers_reverse);

        Self {
            hidden_field_numbers,
            hidden_field_numbers_reverse,
            field_numbers,
            field_numbers_reverse,
            recursive_num_fields,
        }
    }

    pub fn recursive_num_fields(&self) -> u32 {
        self.recursive_num_fields
    }
}