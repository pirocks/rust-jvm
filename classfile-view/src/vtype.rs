use rust_jvm_common::classfile::UninitializedVariableInfo;

use crate::loading::ClassWithLoader;
use crate::view::ptype_view::PTypeView;

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
    ArrayReferenceType(PTypeView),
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

pub static mut VCLONE_COUNT: usize = 0;

impl Clone for VType {
    fn clone(&self) -> Self {
        unsafe { VCLONE_COUNT += 1; }
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
