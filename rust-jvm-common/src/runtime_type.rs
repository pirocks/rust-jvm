use crate::compressed_classfile::class_names::{CClassName, CompressedClassName};
use crate::compressed_classfile::compressed_types::CPDType;

pub type RType = RuntimeType;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum RuntimeType {
    IntType,
    FloatType,
    DoubleType,
    LongType,
    Ref(RuntimeRefType),
    TopType,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub enum RuntimeRefType {
    Array(CPDType),
    Class(CClassName),
    NullType,
}

impl From<CompressedClassName> for RuntimeType {
    fn from(ccn: CompressedClassName) -> Self {
        Self::Ref(RuntimeRefType::Class(ccn))
    }
}

impl RuntimeType {
    pub fn is_array(&self) -> bool {
        match self {
            RuntimeType::Ref(ref_) => match ref_ {
                RuntimeRefType::Array(_) => true,
                RuntimeRefType::Class(_) => false,
                RuntimeRefType::NullType => false,
            },
            _ => false,
        }
    }

    pub fn unwrap_ref_type(&self) -> &RuntimeRefType {
        match self {
            RuntimeType::Ref(ref_) => ref_,
            _ => panic!(),
        }
    }

    pub fn object() -> Self {
        Self::Ref(RuntimeRefType::Class(CClassName::object()))
    }

}