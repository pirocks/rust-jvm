use core::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};

use wtf8::Wtf8Buf;

use sketch_jvm_version_of_utf8::ValidationError;

use crate::compressed_classfile::code::LiveObjectIndex;
use crate::compressed_classfile::CPRefType;
use crate::compressed_classfile::names::CClassName;
use crate::loading::ClassfileParsingError::UTFValidationError;

pub trait LivePoolGetter {
    fn elem_type(&self, idx: LiveObjectIndex) -> CPRefType;
}

pub struct NoopLivePoolGetter {}

impl LivePoolGetter for NoopLivePoolGetter {
    fn elem_type(&self, _idx: LiveObjectIndex) -> CPRefType {
        panic!()
    }
}

#[derive(Debug)]
pub enum ClassfileParsingError {
    EOF,
    WrongMagic,
    NoAttributeName,
    EndOfInstructions,
    WrongInstructionType,
    ATypeWrong,
    WrongPtype,
    UsedReservedStackMapEntry,
    WrongStackMapFrameType,
    WrongTag,
    WromngCPEntry,
    UTFValidationError(ValidationError),
    WrongDescriptor,
}

impl From<ValidationError> for ClassfileParsingError {
    fn from(err: ValidationError) -> Self {
        Self::UTFValidationError(err)
    }
}

impl From<wtf8::Wtf8Buf> for ClassfileParsingError {
    fn from(_: Wtf8Buf) -> Self {
        UTFValidationError(ValidationError::InvalidCodePoint)
    }
}

impl std::error::Error for ClassfileParsingError {}

impl Display for ClassfileParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub enum ClassLoadingError {
    ClassNotFoundException,
    ClassFileInvalid(ClassfileParsingError),
    // ClassFormatError , UnsupportedClassVersionError
    ClassVerificationError, // java.lang.VerifyError
}

impl From<ClassfileParsingError> for ClassLoadingError {
    fn from(error: ClassfileParsingError) -> Self {
        ClassLoadingError::ClassFileInvalid(error)
    }
}

impl Display for ClassLoadingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ClassNotFoundException")
    }
}

impl std::error::Error for ClassLoadingError {}

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct LoaderIndex(pub usize);

#[derive(Debug, Eq, Clone, Hash, Copy)]
pub enum LoaderName {
    UserDefinedLoader(LoaderIndex),
    BootstrapLoader,
}

impl PartialEq for LoaderName {
    fn eq(&self, other: &LoaderName) -> bool {
        match self {
            LoaderName::BootstrapLoader => match other {
                LoaderName::BootstrapLoader => true,
                LoaderName::UserDefinedLoader(_) => false,
            },

            LoaderName::UserDefinedLoader(idx) => match other {
                LoaderName::UserDefinedLoader(other_idx) => other_idx == idx,
                LoaderName::BootstrapLoader => false,
            },
        }
    }
}

impl Display for LoaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoaderName::BootstrapLoader => {
                write!(f, "<bl>")
            }
            LoaderName::UserDefinedLoader(idx) => {
                write!(f, "{}", idx.0)
            }
        }
    }
}

#[derive(Debug)]
pub struct ClassWithLoader {
    pub class_name: CClassName,
    pub loader: LoaderName,
}

impl Hash for ClassWithLoader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
        self.loader.hash(state);
    }
}

impl PartialEq for ClassWithLoader {
    fn eq(&self, other: &ClassWithLoader) -> bool {
        self.class_name == other.class_name && self.loader == other.loader
    }
}

impl Clone for ClassWithLoader {
    fn clone(&self) -> Self {
        ClassWithLoader {
            class_name: self.class_name.clone(),
            loader: self.loader.clone(),
        }
    }
}

impl Eq for ClassWithLoader {}

/*impl Debug for ClassWithLoader {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "<{},{}>", &self.class_name.get_referred_name(), self.loader)
    }
}*/