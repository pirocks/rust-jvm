use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::{CClassName, CompressedClassName};

pub type RType = RuntimeType;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum RuntimeType {
    IntType,
    FloatType,
    DoubleType,
    LongType,
    Ref(RuntimeRefType),
    TopType,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
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

