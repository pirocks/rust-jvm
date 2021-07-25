use std::ops::Deref;

use crate::classfile::UninitializedVariableInfo;
use crate::compressed_classfile::{CompressedClassfileStringPool, CPDType};
use crate::compressed_classfile::names::CompressedClassName;
use crate::loading::{ClassWithLoader, LoaderName};
use crate::ptype::{PType, ReferenceType};

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum VType {
    //VType for VerificationType
    // this doesn't reside in the verifier b/c class view needs to_verification_type on PTypeView
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Class(ClassWithLoader),
    ArrayReferenceType(CPDType),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,
    //todo hack. so b/c stackmapframes doesn't really know what type to give to UnitialziedThis, b/c invoke special could have happened or not
    // I suspect that Uninitialized might work for this, but making my own anyway
    UninitializedThisOrClass(CPDType),
    //below here used internally in isAssignable

    TwoWord,
    OneWord,
    Reference,
    UninitializedEmpty,
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

impl VType {
    pub fn from_ptype(ptype: &PType, loader: LoaderName, pool: &CompressedClassfileStringPool) -> Self {
        match ptype {
            PType::ByteType => VType::IntType,
            PType::CharType => VType::IntType,
            PType::DoubleType => VType::DoubleType,
            PType::FloatType => VType::FloatType,
            PType::IntType => VType::IntType,
            PType::LongType => VType::LongType,
            PType::Ref(ref_) => {
                match ref_ {
                    ReferenceType::Class(ccn) => {
                        VType::Class(ClassWithLoader { class_name: CompressedClassName(pool.add_name(ccn.get_referred_name().clone(), true)), loader })
                    }
                    ReferenceType::Array(arr) => {
                        VType::ArrayReferenceType(CPDType::from_ptype(arr.deref(), pool))
                    }
                }
            }
            PType::ShortType => VType::IntType,
            PType::BooleanType => VType::IntType,
            PType::VoidType => VType::VoidType,
            PType::TopType => VType::TopType,
            PType::NullType => VType::NullType,
            PType::Uninitialized(uninitvarinfo) => VType::Uninitialized(uninitvarinfo.clone()),
            PType::UninitializedThis => VType::UninitializedThis,
            PType::UninitializedThisOrClass(ptype) => VType::UninitializedThisOrClass(CPDType::from_ptype(ptype.deref(), pool))
        }
    }
}

