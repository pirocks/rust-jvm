use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;

use crate::classfile::UninitializedVariableInfo;
use crate::classnames::ClassName;

#[derive(Debug, Eq, PartialEq, Hash)]
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

#[derive(Debug, Eq, PartialEq, Hash)]
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
            PType::Ref(r) => match r {
                ReferenceType::Class(_) => panic!(),
                ReferenceType::Array(a) => a.deref().clone(),
            },
            _ => panic!(),
        }
    }
    pub fn unwrap_class_type(&self) -> ClassName {
        match self {
            PType::Ref(r) => match r {
                ReferenceType::Class(c) => c.clone(),
                ReferenceType::Array(_) => panic!(),
            },
            _ => panic!(),
        }
    }

    pub fn unwrap_ref_type(&self) -> ReferenceType {
        match self {
            PType::Ref(ref_) => ref_.clone(),
            _ => panic!(),
        }
    }

    pub fn jvm_representation(&self) -> String{
        let mut res = String::new();
        //todo dup with ptypeview
        match self {
            PType::ByteType => res.push('B'),
            PType::CharType => res.push('C'),
            PType::DoubleType => res.push('D'),
            PType::FloatType => res.push('F'),
            PType::IntType => res.push('I'),
            PType::LongType => res.push('J'),
            PType::Ref(ref_) => match ref_ {
                ReferenceType::Class(c) => {
                    res.push('L');
                    res.push_str(c.get_referred_name());
                    res.push(';')
                }
                ReferenceType::Array(subtype) => {
                    res.push('[');
                    res.push_str(&subtype.deref().jvm_representation())
                }
            },
            PType::ShortType => res.push('S'),
            PType::BooleanType => res.push('Z'),
            PType::VoidType => res.push('V'),
            _ => panic!(),
        }
        res
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
            PType::Ref(r) => PType::Ref(r.clone()),
        }
    }
}
