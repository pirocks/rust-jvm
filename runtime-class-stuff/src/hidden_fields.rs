use rust_jvm_common::compressed_classfile::{CompressedParsedDescriptorType, CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use strum_macros::EnumIter;


#[allow(non_camel_case_types)]
#[derive(Debug, EnumIter)]
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(non_snake_case)]
#[allow(clippy::upper_case_acronyms)]
pub enum HiddenFields {
    ClassComponentType,
    ClassIsArray,
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

    pub fn class_hidden_fields() -> Vec<HiddenJVMFieldAndFieldType> {
        vec![HiddenJVMFieldAndFieldType {
            name: Self::class_component_type(),
            cpdtype: CompressedParsedDescriptorType::Class(CClassName::class()),
        }, HiddenJVMFieldAndFieldType{
            name: Self::class_is_array(),
            cpdtype: CompressedParsedDescriptorType::BooleanType
        }]
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct HiddenJVMFieldAndFieldType {
    pub name: HiddenJVMField,
    pub cpdtype: CPDType,
}

