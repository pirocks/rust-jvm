use rust_jvm_common::unified_types::{PType, ReferenceType};
use std::ops::Deref;
use loading_common::{ClassWithLoader, LoaderArc};
use rust_jvm_common::classfile::UninitializedVariableInfo;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum VType {
    //VType for VerificationType
    // todo perhaps this should reside in the verifier
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Class(ClassWithLoader),
    ArrayReferenceType(PType),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,
    //todo hack. so b/c stackmapframes doesn't really know what type to give to UnitialziedThis, b/c invoke special could have happened or not
    // I suspect that Uninitialized might work for this, but making my own anyway
    UninitializedThisOrClass(Box<VType>),
    //below here used internally in isAssignable

    TwoWord,
    OneWord,
    Reference,
    UninitializedEmpty,
}


pub fn to_verification_type(p : &PType, loader: &LoaderArc) -> VType {
    match p {
        PType::ByteType => VType::IntType,
        PType::CharType => VType::IntType,
        PType::DoubleType => VType::DoubleType,
        PType::FloatType => VType::FloatType,
        PType::IntType => VType::IntType,
        PType::LongType => VType::LongType,
        PType::ShortType => VType::IntType,
        PType::BooleanType => VType::IntType,
        PType::VoidType => VType::VoidType,
        PType::TopType => VType::TopType,
        PType::NullType => VType::NullType,
        PType::Uninitialized(uvi) => VType::Uninitialized(uvi.clone()),
        PType::UninitializedThis => VType::UninitializedThis,
        PType::UninitializedThisOrClass(c) => VType::UninitializedThisOrClass(Box::new(c.to_verification_type(loader))),
        PType::Ref(r) => {
            match r {
                ReferenceType::Class(c) => { VType::Class(ClassWithLoader{ class_name: c.clone(), loader: loader.clone() }) }
                ReferenceType::Array(p) => { VType::ArrayReferenceType(p.deref().clone()) }
            }
        }
    }
}


impl Clone for VType {
    fn clone(&self) -> Self {
        match self {
            VType::DoubleType => VType::DoubleType,
            VType::FloatType => VType::FloatType,
            VType::IntType => VType::IntType,
            VType::LongType => VType::LongType,
            VType::Class(cl) => VType::Class(cl.clone()),
            VType::ArrayReferenceType(at) => VType::ArrayReferenceType(at.clone()),
            VType::VoidType => VType::VoidType,
            VType::TopType => VType::TopType,
            VType::NullType => VType::NullType,
            VType::Uninitialized(uvi) => VType::Uninitialized(uvi.clone()),
            VType::UninitializedThis => VType::UninitializedThis,
            VType::TwoWord => VType::TwoWord,
            VType::OneWord => VType::OneWord,
            VType::Reference => VType::TwoWord,
            VType::UninitializedEmpty => VType::OneWord,
            VType::UninitializedThisOrClass(t) => VType::UninitializedThisOrClass(t.clone())
        }
    }
}
