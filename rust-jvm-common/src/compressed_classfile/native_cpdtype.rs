use std::num::NonZeroU8;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use add_only_static_vec::AddOnlyId;

use crate::compressed_classfile::{CompressedClassfileString, CompressedParsedDescriptorType, CPDType, NonArrayCompressedParsedDescriptorType};
use crate::compressed_classfile::names::{CompressedClassName};

const CPDTYPE_MAX_DISCRIMINANT: u8 = 10;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, FromPrimitive)]
#[repr(u8)]
pub enum CompressedParsedDescriptorTypeNativeDiscriminant {
    BooleanType = 0,
    ByteType = 1,
    ShortType = 2,
    CharType = 3,
    IntType = 4,
    LongType = 5,
    FloatType = 6,
    DoubleType = 7,
    VoidType = 8,
    Class = 9,
    Array = CPDTYPE_MAX_DISCRIMINANT,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, FromPrimitive)]
#[repr(u8)]
pub enum NonArrayCompressedParsedDescriptorTypeNativeDiscriminant {
    BooleanType = 0,
    ByteType = 1,
    ShortType = 2,
    CharType = 3,
    IntType = 4,
    LongType = 5,
    FloatType = 6,
    DoubleType = 7,
    VoidType = 8,
    Class = 9,
}

impl NonArrayCompressedParsedDescriptorType {
    pub fn discriminant(&self) -> NonArrayCompressedParsedDescriptorTypeNativeDiscriminant {
        match self {
            NonArrayCompressedParsedDescriptorType::BooleanType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::BooleanType,
            NonArrayCompressedParsedDescriptorType::ByteType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::ByteType,
            NonArrayCompressedParsedDescriptorType::ShortType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::ShortType,
            NonArrayCompressedParsedDescriptorType::CharType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::CharType,
            NonArrayCompressedParsedDescriptorType::IntType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::IntType,
            NonArrayCompressedParsedDescriptorType::LongType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::LongType,
            NonArrayCompressedParsedDescriptorType::FloatType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::FloatType,
            NonArrayCompressedParsedDescriptorType::DoubleType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::DoubleType,
            NonArrayCompressedParsedDescriptorType::VoidType => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::VoidType,
            NonArrayCompressedParsedDescriptorType::Class(_) => NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::Class,
        }
    }
}


impl CompressedParsedDescriptorType {
    pub fn discriminant(&self) -> CompressedParsedDescriptorTypeNativeDiscriminant {
        match self {
            CompressedParsedDescriptorType::BooleanType => CompressedParsedDescriptorTypeNativeDiscriminant::BooleanType,
            CompressedParsedDescriptorType::ByteType => CompressedParsedDescriptorTypeNativeDiscriminant::ByteType,
            CompressedParsedDescriptorType::ShortType => CompressedParsedDescriptorTypeNativeDiscriminant::ShortType,
            CompressedParsedDescriptorType::CharType => CompressedParsedDescriptorTypeNativeDiscriminant::CharType,
            CompressedParsedDescriptorType::IntType => CompressedParsedDescriptorTypeNativeDiscriminant::IntType,
            CompressedParsedDescriptorType::LongType => CompressedParsedDescriptorTypeNativeDiscriminant::LongType,
            CompressedParsedDescriptorType::FloatType => CompressedParsedDescriptorTypeNativeDiscriminant::FloatType,
            CompressedParsedDescriptorType::DoubleType => CompressedParsedDescriptorTypeNativeDiscriminant::DoubleType,
            CompressedParsedDescriptorType::VoidType => CompressedParsedDescriptorTypeNativeDiscriminant::VoidType,
            CompressedParsedDescriptorType::Class(_) => CompressedParsedDescriptorTypeNativeDiscriminant::Class,
            CompressedParsedDescriptorType::Array { .. } => CompressedParsedDescriptorTypeNativeDiscriminant::Array,
        }
    }

    pub fn to_native(&self) -> NativeCPDType {
        let discriminant = self.discriminant() as u8;
        let mut res = (discriminant as u64) << 56;
        match self {
            CompressedParsedDescriptorType::Class(ccn) => {
                res |= ccn.0.id.0 as u64
            }
            CompressedParsedDescriptorType::Array {
                base_type,
                num_nested_arrs
            } => {
                res = CPDTYPE_MAX_DISCRIMINANT as u64 + 1 + base_type.discriminant() as u64;
                res |= (num_nested_arrs.get() as u8 as u64) << 48;
                res |= match base_type {
                    NonArrayCompressedParsedDescriptorType::Class(ccn) => {
                        ccn.0.id.0 as u32 as u64
                    }
                    _ => 0
                }
            }
            _ => {}
        };
        NativeCPDType(res)
    }
}

pub struct NativeCPDType(u64);

impl NativeCPDType {
    pub fn to_cpdtype(&self) -> CPDType {
        let discriminant_raw = (self.0 >> 56) as u8;
        match CompressedParsedDescriptorTypeNativeDiscriminant::from_u8(discriminant_raw) {
            None => {
                let base_type_raw = discriminant_raw - CPDTYPE_MAX_DISCRIMINANT - 1;
                let non_array_base_type = match NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::from_u8(base_type_raw).unwrap() {
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::BooleanType => NonArrayCompressedParsedDescriptorType::BooleanType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::ByteType => NonArrayCompressedParsedDescriptorType::ByteType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::ShortType => NonArrayCompressedParsedDescriptorType::ShortType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::CharType => NonArrayCompressedParsedDescriptorType::CharType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::IntType => NonArrayCompressedParsedDescriptorType::IntType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::LongType => NonArrayCompressedParsedDescriptorType::LongType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::FloatType => NonArrayCompressedParsedDescriptorType::FloatType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::DoubleType => NonArrayCompressedParsedDescriptorType::DoubleType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::VoidType => NonArrayCompressedParsedDescriptorType::VoidType,
                    NonArrayCompressedParsedDescriptorTypeNativeDiscriminant::Class => NonArrayCompressedParsedDescriptorType::Class(CompressedClassName(CompressedClassfileString{ id: AddOnlyId(
                        self.0 as u32
                    ) })),
                };
                CompressedParsedDescriptorType::Array {
                    base_type: non_array_base_type,
                    num_nested_arrs: NonZeroU8::new(((self.0 >> 48) | 0xff) as u8).unwrap(),
                }
            }
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::BooleanType) => CompressedParsedDescriptorType::BooleanType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::ByteType) => CompressedParsedDescriptorType::ByteType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::ShortType) => CompressedParsedDescriptorType::ShortType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::CharType) => CompressedParsedDescriptorType::CharType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::IntType) => CompressedParsedDescriptorType::IntType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::LongType) => CompressedParsedDescriptorType::LongType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::FloatType) => CompressedParsedDescriptorType::FloatType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::DoubleType) => CompressedParsedDescriptorType::DoubleType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::VoidType) => CompressedParsedDescriptorType::VoidType,
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::Class) => CompressedParsedDescriptorType::Class(CompressedClassName(CompressedClassfileString { id: AddOnlyId(self.0 as u32) })),
            Some(CompressedParsedDescriptorTypeNativeDiscriminant::Array) => {
                panic!()
            }
        }
    }
}


#[cfg(test)]
pub mod to_native_from_native_test {}