use crate::loading::{LoaderArc, ClassWithLoader};
use std::ops::Deref;
use crate::unified_types::ReferenceType;
use crate::classfile::UninitializedVariableInfo;
use crate::classnames::ClassName;
use crate::vtype::VType;


#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub enum PTypeView {
    ByteType,
    CharType,
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Ref(ReferenceTypeView),
    ShortType,
    BooleanType,
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,

    //todo hack. so b/c stackmapframes doesn't really know what type to give to UnitialziedThis, b/c invoke special could have happened or not
    // I suspect that Uninitialized might work for this, but making my own anyway
    UninitializedThisOrClass(Box<PTypeView>),
}


impl PTypeView {
    pub fn to_verification_type(&self, loader: &LoaderArc) -> VType {
        match self {
            PTypeView::ByteType => VType::IntType,
            PTypeView::CharType => VType::IntType,
            PTypeView::DoubleType => VType::DoubleType,
            PTypeView::FloatType => VType::FloatType,
            PTypeView::IntType => VType::IntType,
            PTypeView::LongType => VType::LongType,
//            PTypeView::Class(cl) => VType::Class(cl.clone()),
            PTypeView::ShortType => VType::IntType,
            PTypeView::BooleanType => VType::IntType,
//            PTypeView::ArrayReferenceType(at) => VType::ArrayReferenceType(at.clone()),
            PTypeView::VoidType => VType::VoidType,
            PTypeView::TopType => VType::TopType,
            PTypeView::NullType => VType::NullType,
            PTypeView::Uninitialized(uvi) => VType::Uninitialized(uvi.clone()),
            PTypeView::UninitializedThis => VType::UninitializedThis,
            PTypeView::UninitializedThisOrClass(c) => VType::UninitializedThisOrClass(Box::new(c.to_verification_type(loader))),
            PTypeView::Ref(r) => {
                match r {
                    ReferenceTypeView::Class(c) => { VType::Class(ClassWithLoader { class_name: c.clone(), loader: loader.clone() }) }
                    ReferenceTypeView::Array(p) => { VType::ArrayReferenceType(p.deref().clone()) }
                }
            }
        }
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub enum ReferenceTypeView {
    Class(ClassName),
    Array(Box<PTypeView>),
}


impl Clone for ReferenceTypeView{
    fn clone(&self) -> Self {
        match self{
            ReferenceTypeView::Class(c) => ReferenceTypeView::Class(c.clone()),
            ReferenceTypeView::Array(a) => ReferenceTypeView::Array(Box::new(a.deref().clone())),
        }
    }
}


impl Clone for PTypeView {
    fn clone(&self) -> Self {
        match self {
            PTypeView::ByteType => PTypeView::ByteType,
            PTypeView::CharType => PTypeView::CharType,
            PTypeView::DoubleType => PTypeView::DoubleType,
            PTypeView::FloatType => PTypeView::FloatType,
            PTypeView::IntType => PTypeView::IntType,
            PTypeView::LongType => PTypeView::LongType,
            PTypeView::ShortType => PTypeView::ShortType,
            PTypeView::BooleanType => PTypeView::BooleanType,
            PTypeView::VoidType => PTypeView::VoidType,
            PTypeView::TopType => PTypeView::TopType,
            PTypeView::NullType => PTypeView::NullType,
            PTypeView::Uninitialized(uvi) => PTypeView::Uninitialized(uvi.clone()),
            PTypeView::UninitializedThis => PTypeView::UninitializedThis,
            PTypeView::UninitializedThisOrClass(t) => PTypeView::UninitializedThisOrClass(t.clone()),
            PTypeView::Ref(r) => PTypeView::Ref(r.clone())
        }
    }
}