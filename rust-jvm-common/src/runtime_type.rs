use crate::compressed_classfile::CPDType;
use crate::compressed_classfile::names::{CClassName, CompressedClassName};

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

    pub fn compatible_with_dumb(&self, other: &RuntimeType) -> bool {
        match self {
            RuntimeType::IntType => {
                matches!(other, RuntimeType::IntType)
            }
            RuntimeType::FloatType => {
                matches!(other, RuntimeType::FloatType)
            }
            RuntimeType::DoubleType => {
                matches!(other, RuntimeType::DoubleType)
            }
            RuntimeType::LongType => {
                matches!(other, RuntimeType::LongType)
            }
            RuntimeType::Ref(ref_1) => {
                match other {
                    RuntimeType::Ref(_ref_2) => {
                        match ref_1 {
                            RuntimeRefType::Array(arr_2) => {
                                match ref_1 {
                                    RuntimeRefType::Array(arr_1) => {
                                        arr_1 == arr_2
                                    }
                                    RuntimeRefType::Class(_) => {
                                        todo!()
                                    }
                                    RuntimeRefType::NullType => {
                                        true
                                    }
                                }
                            }
                            RuntimeRefType::Class(_c2) => {
                                match ref_1 {
                                    RuntimeRefType::Array(_) => {
                                        todo!()
                                    }
                                    RuntimeRefType::Class(_c1) => {
                                        true
                                    }
                                    RuntimeRefType::NullType => {
                                        true
                                    }
                                }
                            }
                            RuntimeRefType::NullType => true
                        }
                    }
                    RuntimeType::TopType => {
                        todo!()
                    }
                    _ => false
                }
            }
            RuntimeType::TopType => {
                todo!()
            }
        }
    }

    pub fn try_back_to_cpdtype(&self) -> Option<CPDType> {
        Some(match self {
            RuntimeType::IntType => CPDType::IntType,
            RuntimeType::FloatType => CPDType::FloatType,
            RuntimeType::DoubleType => CPDType::DoubleType,
            RuntimeType::LongType => CPDType::LongType,
            RuntimeType::Ref(ref_) => match ref_ {
                RuntimeRefType::Array(arr) => CPDType::array(*arr),
                RuntimeRefType::Class(class) => CPDType::Class(*class),
                RuntimeRefType::NullType => CPDType::object()
            }
            RuntimeType::TopType => return None,
        })
    }
}