use rust_jvm_common::compressed_classfile::{CClassName, CPDType};

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
