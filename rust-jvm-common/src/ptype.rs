use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

use crate::classfile::UninitializedVariableInfo;
use crate::classnames::ClassName;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub enum PType {
    ByteType,
    CharType,
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Ref(ReferenceType),
    ShortType,
    BooleanType,
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,

    //todo hack. so b/c stackmapframes doesn't really know what type to give to UninitializedThis, b/c invoke special could have happened or not
    // I suspect that Uninitialized might work for this, but making my own anyway
    UninitializedThisOrClass(Box<PType>),

}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub enum ReferenceType {
    Class(ClassName),
    Array(Box<PType>),
}

impl Clone for ReferenceType {
    fn clone(&self) -> Self {
        match self {
            ReferenceType::Class(c) => ReferenceType::Class(c.clone()),
            ReferenceType::Array(a) => ReferenceType::Array(a.clone()),
        }
    }
}

impl PType {
    pub fn unwrap_array_type(&self) -> PType {
        match self {
            PType::Ref(r) => {
                match r {
                    ReferenceType::Class(_) => panic!(),
                    ReferenceType::Array(a) => {
                        a.deref().clone()
                    }
                }
            }
            _ => panic!()
        }
    }
    pub fn unwrap_class_type(&self) -> ClassName {
        match self {
            PType::Ref(r) => {
                match r {
                    ReferenceType::Class(c) => c.clone(),
                    ReferenceType::Array(_) => panic!(),
                }
            }
            _ => panic!()
        }
    }

    pub fn unwrap_ref_type(&self) -> ReferenceType {
        match self {
            PType::Ref(ref_) => ref_.clone(),
            _ => panic!()
        }
    }
}

impl Clone for PType {
    fn clone(&self) -> Self {
        match self {
            PType::ByteType => PType::ByteType,
            PType::CharType => PType::CharType,
            PType::DoubleType => PType::DoubleType,
            PType::FloatType => PType::FloatType,
            PType::IntType => PType::IntType,
            PType::LongType => PType::LongType,
            PType::ShortType => PType::ShortType,
            PType::BooleanType => PType::BooleanType,
            PType::VoidType => PType::VoidType,
            PType::TopType => PType::TopType,
            PType::NullType => PType::NullType,
            PType::Uninitialized(uvi) => PType::Uninitialized(uvi.clone()),
            PType::UninitializedThis => PType::UninitializedThis,
            PType::UninitializedThisOrClass(t) => PType::UninitializedThisOrClass(t.clone()),
            PType::Ref(r) => PType::Ref(r.clone())
        }
    }
}

