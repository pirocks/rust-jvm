use crate::classnames::ClassName;
use crate::classfile::UninitializedVariableInfo;
use crate::loading::LoaderArc;
use std::sync::Arc;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::ops::Deref;
use std::hash::{Hash, Hasher};

//#[derive(Hash)]
pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: LoaderArc,
}

impl Hash for ClassWithLoader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
        self.loader.name().hash(state);
    }
}

impl PartialEq for ClassWithLoader {
    fn eq(&self, other: &ClassWithLoader) -> bool {
        self.class_name == other.class_name &&
            Arc::ptr_eq(&self.loader, &other.loader)
    }
}

impl Clone for ClassWithLoader {
    fn clone(&self) -> Self {
        ClassWithLoader { class_name: self.class_name.clone(), loader: self.loader.clone() }
    }
}

impl Eq for ClassWithLoader {}


impl Debug for ClassWithLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{},{}>", &self.class_name.get_referred_name(), self.loader.name())
    }
}

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

    //todo hack. so b/c stackmapframes doesn't really know what type to give to UnitialziedThis, b/c invoke special could have happened or not
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

impl Clone for ReferenceType{
    fn clone(&self) -> Self {
        match self{
            ReferenceType::Class(c) => ReferenceType::Class(c.clone()),
            ReferenceType::Array(a) => ReferenceType::Array(a.clone()),
        }
    }
}

impl PType {
    pub fn to_verification_type(&self, loader: &LoaderArc) -> VType {
        match self {
            PType::ByteType => VType::IntType,
            PType::CharType => VType::IntType,
            PType::DoubleType => VType::DoubleType,
            PType::FloatType => VType::FloatType,
            PType::IntType => VType::IntType,
            PType::LongType => VType::LongType,
//            PType::Class(cl) => VType::Class(cl.clone()),
            PType::ShortType => VType::IntType,
            PType::BooleanType => VType::IntType,
//            PType::ArrayReferenceType(at) => VType::ArrayReferenceType(at.clone()),
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
}

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
