use std::ops::Deref;

use crate::classfile::UninitializedVariableInfo;
use crate::compressed_classfile::{CompressedClassfileStringPool, CPDType};
use crate::compressed_classfile::names::{CClassName, CompressedClassName};
use crate::loading::{ClassWithLoader, LoaderName};
use crate::ptype::{PType, ReferenceType};
use crate::runtime_type::{RuntimeRefType, RuntimeType};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VType {
    //VType for VerificationType
    // this doesn't reside in the verifier b/c class view needs to_verification_type on PTypeView
    DoubleType,
    FloatType,
    IntType,
    LongType,
    Class(ClassWithLoader),
    ArrayReferenceType(CPDType),
    VoidType,
    TopType,
    NullType,
    Uninitialized(UninitializedVariableInfo),
    UninitializedThis,
    //todo hack. so b/c stackmapframes doesn't really know what type to give to UnitialziedThis, b/c invoke special could have happened or not
    // I suspect that Uninitialized might work for this, but making my own anyway
    UninitializedThisOrClass(CPDType),
    //below here used internally in isAssignable
    TwoWord,
    OneWord,
    Reference,
    UninitializedEmpty,
}

impl VType {
    pub fn from_ptype(ptype: &PType, loader: LoaderName, pool: &CompressedClassfileStringPool) -> Self {
        match ptype {
            PType::ByteType => VType::IntType,
            PType::CharType => VType::IntType,
            PType::DoubleType => VType::DoubleType,
            PType::FloatType => VType::FloatType,
            PType::IntType => VType::IntType,
            PType::LongType => VType::LongType,
            PType::Ref(ref_) => match ref_ {
                ReferenceType::Class(ccn) => VType::Class(ClassWithLoader {
                    class_name: CompressedClassName(pool.add_name(ccn.get_referred_name().clone(), true)),
                    loader,
                }),
                ReferenceType::Array(arr) => VType::ArrayReferenceType(CPDType::from_ptype(arr.deref(), pool)),
            },
            PType::ShortType => VType::IntType,
            PType::BooleanType => VType::IntType,
            PType::VoidType => VType::VoidType,
            PType::TopType => VType::TopType,
            PType::NullType => VType::NullType,
            PType::Uninitialized(uninitvarinfo) => VType::Uninitialized(*uninitvarinfo),
            PType::UninitializedThis => VType::UninitializedThis,
            PType::UninitializedThisOrClass(ptype) => VType::UninitializedThisOrClass(CPDType::from_ptype(ptype.deref(), pool)),
        }
    }

    pub fn to_runtime_type(&self) -> RuntimeType {
        match self {
            VType::DoubleType => RuntimeType::DoubleType,
            VType::FloatType => RuntimeType::FloatType,
            VType::IntType => RuntimeType::IntType,
            VType::LongType => RuntimeType::LongType,
            VType::Class(c) => RuntimeType::Ref(RuntimeRefType::Class(c.class_name)),
            VType::ArrayReferenceType(array_ref) => RuntimeType::Ref(RuntimeRefType::Array(*array_ref)),
            VType::VoidType => panic!(),
            VType::TopType => RuntimeType::TopType,
            VType::NullType => RuntimeType::Ref(RuntimeRefType::NullType),
            VType::Uninitialized(_) => RuntimeType::Ref(RuntimeRefType::Class(CClassName::object())),
            VType::UninitializedThis => RuntimeType::Ref(RuntimeRefType::Class(CClassName::object())),
            VType::UninitializedThisOrClass(_) => RuntimeType::Ref(RuntimeRefType::Class(CClassName::object())),
            VType::TwoWord => panic!(),
            VType::OneWord => panic!(),
            VType::Reference => panic!(),
            VType::UninitializedEmpty => RuntimeType::Ref(RuntimeRefType::Class(CClassName::object())),
        }
    }
}