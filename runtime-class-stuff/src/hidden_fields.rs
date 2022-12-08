use strum_macros::EnumIter;
use rust_jvm_common::compressed_classfile::class_names::CClassName;
use rust_jvm_common::compressed_classfile::compressed_types::{CompressedParsedDescriptorType, CPDType};


#[allow(non_camel_case_types)]
#[derive(Debug, EnumIter)]
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
pub enum HiddenFields {
    ClassComponentType,
    ClassIsArray,
    ClassCPDTypeID,
    WrappedClassCPDTypeID,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct HiddenJVMField(pub u32);

impl HiddenJVMField {
    pub fn from_raw_id(id: HiddenFields) -> Self {
        HiddenJVMField(id as u32)
    }

    pub fn class_component_type() -> Self {
        Self::from_raw_id(HiddenFields::ClassComponentType)
    }

    pub fn class_is_array() -> Self {
        Self::from_raw_id(HiddenFields::ClassIsArray)
    }

    pub fn class_cpdtype_id() -> Self {
        Self::from_raw_id(HiddenFields::ClassCPDTypeID)
    }

    pub fn class_cpdtype_id_of_wrapped_in_array() -> Self {
        Self::from_raw_id(HiddenFields::WrappedClassCPDTypeID)
    }

    pub fn class_hidden_fields() -> Vec<HiddenJVMFieldAndFieldType> {
        vec![
            HiddenJVMFieldAndFieldType {
                name: Self::class_component_type(),
                cpdtype: CompressedParsedDescriptorType::Class(CClassName::class()),
            },
            HiddenJVMFieldAndFieldType {
                name: Self::class_is_array(),
                cpdtype: CompressedParsedDescriptorType::BooleanType,
            },
            HiddenJVMFieldAndFieldType {
                name: Self::class_cpdtype_id(),
                cpdtype: CompressedParsedDescriptorType::IntType,
            },
            HiddenJVMFieldAndFieldType {
                name: Self::class_cpdtype_id_of_wrapped_in_array(),
                cpdtype: CompressedParsedDescriptorType::IntType,
            },
        ]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct HiddenJVMFieldAndFieldType {
    pub name: HiddenJVMField,
    pub cpdtype: CPDType,
}

