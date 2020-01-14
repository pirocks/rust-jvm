use crate::classnames::ClassName;
use crate::classfile::UninitializedVariableInfo;
use crate::loading::Loader;
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

impl Clone for ArrayType{
    fn clone(&self) -> Self {
        ArrayType { sub_type: Box::new(self.sub_type.deref().clone()) }
    }
}

pub struct ClassWithLoader {
    pub class_name: ClassName,
    pub loader: Arc<dyn Loader + Sync + Send>,
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
}

impl ParsedType {
    pub fn to_verification_type(&self) -> VerificationType {
        match self {
            ParsedType::ByteType => VerificationType::IntType,
            ParsedType::CharType => VerificationType::IntType,
            ParsedType::DoubleType => VerificationType::DoubleType,
            ParsedType::FloatType => VerificationType::FloatType,
            ParsedType::IntType => VerificationType::IntType,
            ParsedType::LongType => VerificationType::LongType,
            ParsedType::Class(cl) => VerificationType::Class(cl.clone()),
            ParsedType::ShortType => VerificationType::IntType,
            ParsedType::BooleanType => VerificationType::IntType,
            ParsedType::ArrayReferenceType(at) => VerificationType::ArrayReferenceType(at.clone()),
            ParsedType::VoidType => VerificationType::VoidType,
            ParsedType::TopType => VerificationType::TopType,
            ParsedType::NullType => VerificationType::NullType,
            ParsedType::Uninitialized(uvi) => VerificationType::Uninitialized(uvi.clone()),
            ParsedType::UninitializedThis => VerificationType::UninitializedThis
        }
    }
}

#[derive(Debug)]
#[derive(Eq, PartialEq)]
pub enum VerificationType {// todo perhaps this should reside in the verifier
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
            ParsedType::UninitializedThis => ParsedType::UninitializedThis
        }
    }
}


impl Clone for VerificationType {
    fn clone(&self) -> Self {
        match self {
            VerificationType::DoubleType => VerificationType::DoubleType,
            VerificationType::FloatType => VerificationType::FloatType,
            VerificationType::IntType => VerificationType::IntType,
            VerificationType::LongType => VerificationType::LongType,
            VerificationType::Class(cl) => VerificationType::Class(cl.clone()),
            VerificationType::ArrayReferenceType(at) => VerificationType::ArrayReferenceType(at.clone()),
            VerificationType::VoidType => VerificationType::VoidType,
            VerificationType::TopType => VerificationType::TopType,
            VerificationType::NullType => VerificationType::NullType,
            VerificationType::Uninitialized(uvi) => VerificationType::Uninitialized(uvi.clone()),
            VerificationType::UninitializedThis => VerificationType::UninitializedThis,
            VerificationType::TwoWord => VerificationType::TwoWord,
            VerificationType::OneWord => VerificationType::OneWord,
            VerificationType::Reference => VerificationType::TwoWord,
            VerificationType::UninitializedEmpty => VerificationType::OneWord,
        }
    }
}
