use crate::classnames::ClassName;
use crate::classfile::UninitializedVariableInfo;
use crate::loading::LoaderArc;
use std::sync::Arc;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use std::ops::Deref;

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub struct ArrayType {
    pub sub_type: Box<ParsedType>
}

impl Clone for ArrayType {
    fn clone(&self) -> Self {
        ArrayType { sub_type: Box::new(self.sub_type.deref().clone()) }
    }
}

pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: LoaderArc,
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
pub enum ParsedType {
    ByteType,
    CharType,
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Class(ClassWithLoader),
    ShortType,
    BooleanType,
    ArrayReferenceType(ArrayType),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,

    //todo hack. so b/c stackmapframes doesn't really know what type to give to UnitialziedThis, b/c invoke special could have happened or not
    // I suspect that Uninitialized might work for this, but making my own anyway
    UninitializedThisOrClass(Box<ParsedType>),

}

impl ParsedType {
    pub fn to_verification_type(&self) -> VType {
        match self {
            ParsedType::ByteType => VType::IntType,
            ParsedType::CharType => VType::IntType,
            ParsedType::DoubleType => VType::DoubleType,
            ParsedType::FloatType => VType::FloatType,
            ParsedType::IntType => VType::IntType,
            ParsedType::LongType => VType::LongType,
            ParsedType::Class(cl) => VType::Class(cl.clone()),
            ParsedType::ShortType => VType::IntType,
            ParsedType::BooleanType => VType::IntType,
            ParsedType::ArrayReferenceType(at) => VType::ArrayReferenceType(at.clone()),
            ParsedType::VoidType => VType::VoidType,
            ParsedType::TopType => VType::TopType,
            ParsedType::NullType => VType::NullType,
            ParsedType::Uninitialized(uvi) => VType::Uninitialized(uvi.clone()),
            ParsedType::UninitializedThis => VType::UninitializedThis,
            ParsedType::UninitializedThisOrClass(c) => VType::UninitializedThisOrClass(Box::new(c.to_verification_type()))
        }
    }
    pub fn unwrap_array_type(&self) -> ParsedType{
        match self {
            ParsedType::ArrayReferenceType(a) => {
                a.sub_type.deref().clone()
            },
            _ => panic!()
        }
    }
    pub fn unwrap_class_type(&self) -> ClassWithLoader{
        match self {
            ParsedType::Class(c) => {
                c.clone()
            },
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
    ArrayReferenceType(ArrayType),
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

impl Clone for ParsedType {
    fn clone(&self) -> Self {
        match self {
            ParsedType::ByteType => ParsedType::ByteType,
            ParsedType::CharType => ParsedType::CharType,
            ParsedType::DoubleType => ParsedType::DoubleType,
            ParsedType::FloatType => ParsedType::FloatType,
            ParsedType::IntType => ParsedType::IntType,
            ParsedType::LongType => ParsedType::LongType,
            ParsedType::Class(cl) => ParsedType::Class(cl.clone()),
            ParsedType::ShortType => ParsedType::ShortType,
            ParsedType::BooleanType => ParsedType::BooleanType,
            ParsedType::ArrayReferenceType(at) => ParsedType::ArrayReferenceType(at.clone()),
            ParsedType::VoidType => ParsedType::VoidType,
            ParsedType::TopType => ParsedType::TopType,
            ParsedType::NullType => ParsedType::NullType,
            ParsedType::Uninitialized(uvi) => ParsedType::Uninitialized(uvi.clone()),
            ParsedType::UninitializedThis => ParsedType::UninitializedThis,
            ParsedType::UninitializedThisOrClass(t) => ParsedType::UninitializedThisOrClass(t.clone())
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
