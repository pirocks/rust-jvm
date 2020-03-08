use std::ops::Deref;
use rust_jvm_common::classfile::UninitializedVariableInfo;
use rust_jvm_common::classnames::ClassName;
use crate::vtype::VType;
use rust_jvm_common::ptype::{PType, ReferenceType};
use crate::loading::{LoaderArc, ClassWithLoader};


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
    pub fn to_ptype(&self) -> PType{
        match self{
            PTypeView::ByteType => PType::ByteType,
            PTypeView::CharType => PType::CharType,
            PTypeView::DoubleType => PType::DoubleType,
            PTypeView::FloatType => PType::FloatType,
            PTypeView::IntType => PType::IntType,
            PTypeView::LongType => PType::LongType,
            PTypeView::Ref(r) => PType::Ref(r.to_reference_type()),
            PTypeView::ShortType => PType::ShortType,
            PTypeView::BooleanType => PType::BooleanType,
            PTypeView::VoidType => PType::VoidType,
            PTypeView::TopType => PType::TopType,
            PTypeView::NullType => PType::NullType,
            PTypeView::Uninitialized(u) => PType::Uninitialized(u.clone()),
            PTypeView::UninitializedThis => PType::UninitializedThis,
            PTypeView::UninitializedThisOrClass(u) => PType::UninitializedThisOrClass(u.deref().to_ptype().into()),
        }
    }

    pub fn from_ptype(p : &PType) -> PTypeView{
        match p{
            PType::ByteType => PTypeView::ByteType,
            PType::CharType => PTypeView::CharType,
            PType::DoubleType => PTypeView::DoubleType,
            PType::FloatType => PTypeView::FloatType,
            PType::IntType => PTypeView::IntType,
            PType::LongType => PTypeView::LongType,
            PType::Ref(r) => PTypeView::Ref(ReferenceTypeView::from_reference_type(r)),
            PType::ShortType => PTypeView::ShortType,
            PType::BooleanType => PTypeView::BooleanType,
            PType::VoidType => PTypeView::VoidType,
            PType::TopType => PTypeView::TopType,
            PType::NullType => PTypeView::NullType,
            PType::Uninitialized(u) => PTypeView::Uninitialized(u.clone()),
            PType::UninitializedThis => PTypeView::UninitializedThis,
            PType::UninitializedThisOrClass(u) => PTypeView::UninitializedThisOrClass(PTypeView::from_ptype(u.deref()).into()),
        }
    }

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

    pub fn is_primitive(&self) -> bool{
        match self{
            PTypeView::ByteType => true,
            PTypeView::CharType => true,
            PTypeView::DoubleType => true,
            PTypeView::FloatType => true,
            PTypeView::IntType => true,
            PTypeView::LongType => true,
            PTypeView::Ref(_) => false,
            PTypeView::ShortType => true,
            PTypeView::BooleanType => true,
            PTypeView::VoidType => true,
            PTypeView::TopType => false,
            PTypeView::NullType => false,
            PTypeView::Uninitialized(_) => false,
            PTypeView::UninitializedThis => false,
            PTypeView::UninitializedThisOrClass(_) => false,
        }
    }

    pub fn primitive_name(&self) -> &'static str{
        match self {
            PTypeView::ByteType => "byte",
            PTypeView::CharType => "char",
            PTypeView::DoubleType => "double",
            PTypeView::FloatType => "float",
            PTypeView::IntType => "int",
            PTypeView::LongType => "long",
            PTypeView::ShortType => "short",
            PTypeView::BooleanType => "boolean",
            PTypeView::VoidType => "void",
            _ => panic!(),
        }
    }

    pub fn object() -> Self{
        PTypeView::Ref(ReferenceTypeView::Class(ClassName::object()))
    }

    pub fn array(of : Self) -> Self{
        PTypeView::Ref(ReferenceTypeView::Array(of.into()))
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
#[derive(Hash)]
pub enum ReferenceTypeView {
    Class(ClassName),
    Array(Box<PTypeView>),
}

impl ReferenceTypeView{
    pub fn to_reference_type(&self) -> ReferenceType{
        match self{
            ReferenceTypeView::Class(c) => ReferenceType::Class(c.clone()),
            ReferenceTypeView::Array(a) => ReferenceType::Array(a.deref().to_ptype().into()),
        }
    }

    pub fn from_reference_type(ref_ : &ReferenceType) -> ReferenceTypeView{
        match ref_{
            ReferenceType::Class(c) => ReferenceTypeView::Class(c.clone()),
            ReferenceType::Array(a) => ReferenceTypeView::Array(PTypeView::from_ptype(a.deref()).into()),
        }
    }
    pub fn unwrap_name(&self) -> ClassName{
        match self {
            ReferenceTypeView::Class(c) => c.clone(),
            ReferenceTypeView::Array(_) => panic!(),
        }
    }

    pub fn unwrap_arrays_to_name(&self) -> Option<ClassName> {
        match self {
            ReferenceTypeView::Class(c) => c.clone().into(),
            ReferenceTypeView::Array(a) => {
                match a.deref().try_unwrap_ref_type(){
                    None => return None,
                    Some(ref_) => {
                        ref_.unwrap_arrays_to_name()
                    },
                }
            },
        }
    }

    pub fn unwrap_array(&self) -> PTypeView{
        match self {
            ReferenceTypeView::Class(_) => panic!(),
            ReferenceTypeView::Array(a) => {
                a.deref().clone()
            },
        }
    }

    pub fn is_array(&self) -> bool{
        match self{
            ReferenceTypeView::Class(_) => false,
            ReferenceTypeView::Array(_) => true,
        }
    }
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

impl PTypeView {
    pub fn unwrap_array_type(&self) -> PTypeView {
        match self {
            PTypeView::Ref(r) => {
                match r {
                    ReferenceTypeView::Class(_) => panic!(),
                    ReferenceTypeView::Array(a) => {
                        a.deref().clone()
                    }
                }
            }
            _ => panic!()
        }
    }
    pub fn unwrap_class_type(&self) -> ClassName {
        match self {
            PTypeView::Ref(r) => {
                match r {
                    ReferenceTypeView::Class(c) => c.clone(),
                    ReferenceTypeView::Array(_) => panic!(),
                }
            }
            _ => panic!()
        }
    }
    pub fn unwrap_ref_type(&self) -> &ReferenceTypeView{
        match self {
            PTypeView::Ref(r) => r,
            _ => panic!(),
        }
    }
    pub fn try_unwrap_ref_type(&self) -> Option<&ReferenceTypeView>{
        match self {
            PTypeView::Ref(r) => r.into(),
            _ => None,
        }
    }

    pub fn unwrap_type_to_name(&self) -> Option<ClassName> {
        match self{
            PTypeView::ByteType => ClassName::byte().into(),
            PTypeView::CharType => ClassName::character().into(),
            PTypeView::DoubleType => ClassName::double().into(),
            PTypeView::FloatType => ClassName::float().into(),
            PTypeView::IntType => ClassName::int().into(),
            PTypeView::LongType => ClassName::long().into(),
            PTypeView::Ref(r) => r.unwrap_arrays_to_name(),
            PTypeView::ShortType => ClassName::short().into(),
            PTypeView::BooleanType => ClassName::boolean().into(),
            PTypeView::VoidType => ClassName::void().into(),
            _ => panic!(),
        }
    }

    pub fn is_array(&self) -> bool{
        match self{
            PTypeView::ByteType => false,
            PTypeView::CharType => false,
            PTypeView::DoubleType => false,
            PTypeView::FloatType => false,
            PTypeView::IntType => false,
            PTypeView::LongType => false,
            PTypeView::Ref(r) => match r{
                ReferenceTypeView::Class(_) => false,
                ReferenceTypeView::Array(_) => true,
            },
            PTypeView::ShortType => false,
            PTypeView::BooleanType => false,
            PTypeView::VoidType => false,
            PTypeView::TopType => false,
            PTypeView::NullType => false,
            PTypeView::Uninitialized(_) => false,
            PTypeView::UninitializedThis => false,
            PTypeView::UninitializedThisOrClass(_) => false,
        }
    }
}
