use std::cmp::Ordering;
use std::iter;
use itertools::Itertools;
use crate::classfile::UninitializedVariableInfo;
use crate::compressed_classfile::class_names::{CClassName, CompressedClassName};
use crate::compressed_classfile::compressed_descriptors::{CompressedMethodDescriptor, mangling_escape};
use crate::compressed_classfile::string_pool::CompressedClassfileStringPool;
use crate::loading::{ClassWithLoader, LoaderName};
use crate::ptype::{PType, ReferenceType};
use crate::runtime_type::{RuntimeRefType, RuntimeType};
use crate::vtype::VType;
use std::num::NonZeroU8;
use std::ops::Deref;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CompressedParsedVerificationType {
    TopType,
    IntType,
    FloatType,
    DoubleType,
    LongType,
    NullType,
    UninitializedThis,
    Uninitialized(UninitializedVariableInfo),
    Ref(CompressedParsedRefType),
}

pub type CPRefType = CompressedParsedRefType;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum CompressedParsedRefType {
    Class(CompressedClassName) = 0,
    Array {
        base_type: NonArrayCompressedParsedDescriptorType,
        num_nested_arrs: NonZeroU8,
    } = 1,
}

impl CompressedParsedRefType {
    pub(crate) fn short_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            CompressedParsedRefType::Array { base_type, num_nested_arrs } => {
                format!("{}{}", base_type.to_cpdtype().short_representation(string_pool), iter::repeat("[]").take(num_nested_arrs.get() as usize).join(""))
            }
            CompressedParsedRefType::Class(c) => {
                c.0.to_str(string_pool).split('/').last().unwrap().to_string()
            }
        }
    }

    pub fn unwrap_object_name(&self) -> CClassName {
        match self {
            CompressedParsedRefType::Array { .. } => panic!(),
            CompressedParsedRefType::Class(ccn) => *ccn,
        }
    }

    pub fn to_verification_type(&self, loader: LoaderName) -> VType {
        match self {
            CompressedParsedRefType::Array { base_type, num_nested_arrs } => VType::ArrayReferenceType(CPDType::new_array_or_normal(*base_type, num_nested_arrs.get() - 1)),
            CompressedParsedRefType::Class(obj) => VType::Class(ClassWithLoader { class_name: *obj, loader }),
        }
    }
    pub fn to_runtime_type(&self) -> RuntimeRefType {
        match self {
            CompressedParsedRefType::Array {
                base_type, num_nested_arrs
            } => {
                RuntimeRefType::Array(CPDType::new_array_or_normal(*base_type, num_nested_arrs.get() - 1))
            }
            CompressedParsedRefType::Class(class_name) => {
                RuntimeRefType::Class(*class_name)
            }
        }
    }

    pub fn try_unwrap_name(&self) -> Option<CClassName> {
        match self {
            CompressedParsedRefType::Array { .. } => None,
            CompressedParsedRefType::Class(ccn) => Some(*ccn),
        }
    }

    pub fn unwrap_name(&self) -> CClassName {
        self.try_unwrap_name().unwrap()
    }

    pub fn try_unwrap_ref_type(&self) -> Option<CompressedParsedDescriptorType> {
        match self {
            CompressedParsedRefType::Array { base_type, .. } => Some(base_type.to_cpdtype()),
            CompressedParsedRefType::Class(_) => None,
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            CompressedParsedRefType::Array { .. } => true,
            CompressedParsedRefType::Class(_) => false,
        }
    }

    pub fn unwrap_array_type(&self) -> CPDType {
        match self {
            CompressedParsedRefType::Array { base_type, num_nested_arrs } => CPDType::new_array_or_normal(*base_type, num_nested_arrs.get() - 1),
            CompressedParsedRefType::Class(_) => panic!(),
        }
    }

    pub fn recursively_unwrap_array_type(&self) -> NonArrayCompressedParsedDescriptorType {
        match self {
            CompressedParsedRefType::Array { base_type, num_nested_arrs: _ } => *base_type,
            CompressedParsedRefType::Class(_) => panic!(),
        }
    }

    pub fn java_source_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            CompressedParsedRefType::Array { base_type, num_nested_arrs } => {
                format!("{}{}", base_type.to_cpdtype().java_source_representation(string_pool), iter::repeat("[]").take(num_nested_arrs.get() as usize).join(""))
            }
            CompressedParsedRefType::Class(c) => {
                c.0.to_str(string_pool)
            }
        }
    }

    pub fn jvm_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            Self::Class(c) => {
                format!("L{};", c.0.to_str(string_pool))
            }
            Self::Array { base_type, num_nested_arrs } => {
                format!("{}{}", iter::repeat("[").take(num_nested_arrs.get() as usize).join(""), base_type.to_cpdtype().jvm_representation(string_pool))
            }
        }
    }

    pub fn mangled_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            Self::Class(c) => {
                format!("{}{}{}", mangling_escape("L"), mangling_escape(c.0.to_str(string_pool)),mangling_escape(";"))
            }
            Self::Array { base_type, num_nested_arrs } => {
                format!("{}{}", iter::repeat(mangling_escape("[")).take(num_nested_arrs.get() as usize).join(""), base_type.to_cpdtype().mangled_representation(string_pool))
            }
        }
    }

    pub fn to_cpdtype(&self) -> CPDType {
        match *self {
            CompressedParsedRefType::Class(ccn) => CPDType::Class(ccn),
            CompressedParsedRefType::Array { base_type, num_nested_arrs } => CPDType::Array { base_type, num_nested_arrs }
        }
    }
}

pub type CPDType = CompressedParsedDescriptorType;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum CompressedParsedDescriptorType {
    //make sure this stays in sync with CompressedParsedDescriptorTypeNativeDiscriminant
    BooleanType = 0,
    ByteType = 1,
    ShortType = 2,
    CharType = 3,
    IntType = 4,
    LongType = 5,
    FloatType = 6,
    DoubleType = 7,
    VoidType = 8,
    Class(CompressedClassName) = 9,
    Array {
        base_type: NonArrayCompressedParsedDescriptorType,
        num_nested_arrs: NonZeroU8,
    } = 10,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(u8)]
pub enum NonArrayCompressedParsedDescriptorType {
    BooleanType = 0,
    ByteType = 1,
    ShortType = 2,
    CharType = 3,
    IntType = 4,
    LongType = 5,
    FloatType = 6,
    DoubleType = 7,
    VoidType = 8,
    Class(CClassName) = 9,
}

impl NonArrayCompressedParsedDescriptorType {
    pub fn to_cpdtype(&self) -> CPDType {
        match self {
            NonArrayCompressedParsedDescriptorType::BooleanType => {
                CPDType::BooleanType
            }
            NonArrayCompressedParsedDescriptorType::ByteType => {
                CPDType::ByteType
            }
            NonArrayCompressedParsedDescriptorType::ShortType => {
                CPDType::ShortType
            }
            NonArrayCompressedParsedDescriptorType::CharType => {
                CPDType::CharType
            }
            NonArrayCompressedParsedDescriptorType::IntType => {
                CPDType::IntType
            }
            NonArrayCompressedParsedDescriptorType::LongType => {
                CPDType::LongType
            }
            NonArrayCompressedParsedDescriptorType::FloatType => {
                CPDType::FloatType
            }
            NonArrayCompressedParsedDescriptorType::DoubleType => {
                CPDType::DoubleType
            }
            NonArrayCompressedParsedDescriptorType::VoidType => {
                CPDType::VoidType
            }
            NonArrayCompressedParsedDescriptorType::Class(ccn) => {
                CPDType::Class(*ccn)
            }
        }
    }
}

impl CompressedParsedDescriptorType {
    pub fn unwrap_non_array(&self) -> NonArrayCompressedParsedDescriptorType {
        match self {
            CompressedParsedDescriptorType::BooleanType => NonArrayCompressedParsedDescriptorType::BooleanType,
            CompressedParsedDescriptorType::ByteType => NonArrayCompressedParsedDescriptorType::ByteType,
            CompressedParsedDescriptorType::ShortType => NonArrayCompressedParsedDescriptorType::ShortType,
            CompressedParsedDescriptorType::CharType => NonArrayCompressedParsedDescriptorType::CharType,
            CompressedParsedDescriptorType::IntType => NonArrayCompressedParsedDescriptorType::IntType,
            CompressedParsedDescriptorType::LongType => NonArrayCompressedParsedDescriptorType::LongType,
            CompressedParsedDescriptorType::FloatType => NonArrayCompressedParsedDescriptorType::FloatType,
            CompressedParsedDescriptorType::DoubleType => NonArrayCompressedParsedDescriptorType::DoubleType,
            CompressedParsedDescriptorType::VoidType => NonArrayCompressedParsedDescriptorType::VoidType,
            CompressedParsedDescriptorType::Array { .. } => panic!(),
            CompressedParsedDescriptorType::Class(ccn) => {
                NonArrayCompressedParsedDescriptorType::Class(*ccn)
            }
        }
    }

    pub fn new_array_or_normal(inner: NonArrayCompressedParsedDescriptorType, nested: u8) -> CPDType {
        match NonZeroU8::new(nested) {
            None => {
                inner.to_cpdtype()
            }
            Some(nested) => {
                CPDType::Array { base_type: inner, num_nested_arrs: nested }
            }
        }
    }

    pub fn java_source_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            Self::ByteType => "byte".to_string(),
            Self::CharType => "char".to_string(),
            Self::DoubleType => "double".to_string(),
            Self::FloatType => "float".to_string(),
            Self::IntType => "int".to_string(),
            Self::LongType => "long".to_string(),
            Self::Class(ccn) => CPRefType::Class(*ccn).java_source_representation(string_pool),
            Self::ShortType => "short".to_string(),
            Self::BooleanType => "boolean".to_string(),
            Self::VoidType => "void".to_string(),
            Self::Array { base_type, num_nested_arrs } => CPRefType::Array { base_type: *base_type, num_nested_arrs: *num_nested_arrs }.java_source_representation(string_pool)
        }
    }

    pub fn jvm_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            Self::ByteType => "B".to_string(),
            Self::CharType => "C".to_string(),
            Self::DoubleType => "D".to_string(),
            Self::FloatType => "F".to_string(),
            Self::IntType => "I".to_string(),
            Self::LongType => "J".to_string(),
            Self::Class(ccn) => CPRefType::Class(*ccn).jvm_representation(string_pool),
            Self::Array { base_type, num_nested_arrs } => CPRefType::Array { base_type: *base_type, num_nested_arrs: *num_nested_arrs }.jvm_representation(string_pool),
            Self::ShortType => "S".to_string(),
            Self::BooleanType => "Z".to_string(),
            Self::VoidType => "V".to_string(),
        }
    }

    pub fn mangled_representation(&self, string_pool: &CompressedClassfileStringPool) -> String{
        match self {
            Self::ByteType => "B".to_string(),
            Self::CharType => "C".to_string(),
            Self::DoubleType => "D".to_string(),
            Self::FloatType => "F".to_string(),
            Self::IntType => "I".to_string(),
            Self::LongType => "J".to_string(),
            Self::Class(ccn) => CPRefType::Class(*ccn).mangled_representation(string_pool),
            Self::Array { base_type, num_nested_arrs } => CPRefType::Array { base_type: *base_type, num_nested_arrs: *num_nested_arrs }.mangled_representation(string_pool),
            Self::ShortType => "S".to_string(),
            Self::BooleanType => "Z".to_string(),
            Self::VoidType => "V".to_string(),
        }
    }

    pub fn short_representation(&self, string_pool: &CompressedClassfileStringPool) -> String {
        match self {
            Self::ByteType => "B".to_string(),
            Self::CharType => "C".to_string(),
            Self::DoubleType => "D".to_string(),
            Self::FloatType => "F".to_string(),
            Self::IntType => "I".to_string(),
            Self::LongType => "J".to_string(),
            Self::Class(ccn) => CPRefType::Class(*ccn).short_representation(string_pool),
            Self::Array { base_type, num_nested_arrs } => CPRefType::Array { base_type: *base_type, num_nested_arrs: *num_nested_arrs }.short_representation(string_pool),
            Self::ShortType => "S".to_string(),
            Self::BooleanType => "Z".to_string(),
            Self::VoidType => "V".to_string(),
        }
    }


    pub fn unwrap_ref_type(&self) -> CompressedParsedRefType {
        self.try_unwrap_ref_type().unwrap()
    }

    pub fn try_unwrap_ref_type(&self) -> Option<CPRefType> {
        match self {
            CompressedParsedDescriptorType::Class(ccn) => Some(CompressedParsedRefType::Class(*ccn)),
            CompressedParsedDescriptorType::Array { base_type, num_nested_arrs } => Some(CompressedParsedRefType::Array { base_type: *base_type, num_nested_arrs: *num_nested_arrs }),
            _ => None,
        }
    }

    pub fn unwrap_class_type(&self) -> CClassName {
        self.try_unwrap_class_type().unwrap()
    }

    pub fn try_unwrap_class_type(&self) -> Option<CClassName> {
        match self {
            CompressedParsedDescriptorType::Class(ccn) => Some(*ccn),
            _ => None,
        }
    }

    pub fn try_unwrap_array_type(&self) -> Option<CPDType> {
        match self {
            Self::Array { base_type, num_nested_arrs } => Some(CPDType::new_array_or_normal(*base_type, num_nested_arrs.get() - 1)),
            Self::Class(_) => None,
            _ => None,
        }
    }

    pub fn unwrap_array_type(&self) -> CPDType {
        self.try_unwrap_array_type().unwrap()
    }

    pub fn to_verification_type(&self, loader: LoaderName) -> VType {
        match self {
            CompressedParsedDescriptorType::BooleanType => VType::IntType,
            CompressedParsedDescriptorType::ByteType => VType::IntType,
            CompressedParsedDescriptorType::ShortType => VType::IntType,
            CompressedParsedDescriptorType::CharType => VType::IntType,
            CompressedParsedDescriptorType::IntType => VType::IntType,
            CompressedParsedDescriptorType::LongType => VType::LongType,
            CompressedParsedDescriptorType::FloatType => VType::FloatType,
            CompressedParsedDescriptorType::DoubleType => VType::DoubleType,
            CompressedParsedDescriptorType::VoidType => VType::VoidType,
            CompressedParsedDescriptorType::Array { base_type, num_nested_arrs } => VType::ArrayReferenceType(CPDType::new_array_or_normal(*base_type, num_nested_arrs.get() - 1)),
            CompressedParsedDescriptorType::Class(obj) => VType::Class(ClassWithLoader { class_name: *obj, loader }),
        }
    }

    pub fn to_runtime_type(&self) -> Option<RuntimeType> {
        Some(match self {
            CompressedParsedDescriptorType::BooleanType => RuntimeType::IntType,
            CompressedParsedDescriptorType::ByteType => RuntimeType::IntType,
            CompressedParsedDescriptorType::ShortType => RuntimeType::IntType,
            CompressedParsedDescriptorType::CharType => RuntimeType::IntType,
            CompressedParsedDescriptorType::IntType => RuntimeType::IntType,
            CompressedParsedDescriptorType::LongType => RuntimeType::LongType,
            CompressedParsedDescriptorType::FloatType => RuntimeType::FloatType,
            CompressedParsedDescriptorType::DoubleType => RuntimeType::DoubleType,
            CompressedParsedDescriptorType::VoidType => None?,
            CompressedParsedDescriptorType::Array { base_type, num_nested_arrs } => RuntimeType::Ref(RuntimeRefType::Array(CPDType::new_array_or_normal(*base_type, num_nested_arrs.get() - 1))),
            CompressedParsedDescriptorType::Class(ccn) => RuntimeType::Ref(RuntimeRefType::Class(*ccn)),
        })
    }

    pub fn is_primitive(&self) -> bool {
        match self {
            CompressedParsedDescriptorType::BooleanType => true,
            CompressedParsedDescriptorType::ByteType => true,
            CompressedParsedDescriptorType::ShortType => true,
            CompressedParsedDescriptorType::CharType => true,
            CompressedParsedDescriptorType::IntType => true,
            CompressedParsedDescriptorType::LongType => true,
            CompressedParsedDescriptorType::FloatType => true,
            CompressedParsedDescriptorType::DoubleType => true,
            CompressedParsedDescriptorType::VoidType => true,
            CompressedParsedDescriptorType::Class(_) => false,
            CompressedParsedDescriptorType::Array { .. } => false,
        }
    }

    pub fn is_array(&self) -> bool {
        matches!(self, CompressedParsedDescriptorType::Array { .. })
    }

    pub fn is_signed_integer(&self) -> bool{
        match self {
            CompressedParsedDescriptorType::BooleanType => {
                false
            }
            CompressedParsedDescriptorType::ByteType => {
                true
            }
            CompressedParsedDescriptorType::ShortType => {
                true
            }
            CompressedParsedDescriptorType::CharType => {
                false
            }
            CompressedParsedDescriptorType::IntType => {
                true
            }
            CompressedParsedDescriptorType::LongType => {
                true
            }
            CompressedParsedDescriptorType::FloatType => {
                false
            }
            CompressedParsedDescriptorType::DoubleType => {
                false
            }
            CompressedParsedDescriptorType::VoidType => {
                false
            }
            CompressedParsedDescriptorType::Class(_) => {
                false
            }
            CompressedParsedDescriptorType::Array { .. } => {
                false
            }
        }
    }

    pub fn is_void(&self) -> bool {
        matches!(self, CompressedParsedDescriptorType::VoidType)
    }

    pub fn array(sub_type: Self) -> Self {
        let sub_type = match sub_type {
            CompressedParsedDescriptorType::BooleanType => NonArrayCompressedParsedDescriptorType::BooleanType,
            CompressedParsedDescriptorType::ByteType => NonArrayCompressedParsedDescriptorType::ByteType,
            CompressedParsedDescriptorType::ShortType => NonArrayCompressedParsedDescriptorType::ShortType,
            CompressedParsedDescriptorType::CharType => NonArrayCompressedParsedDescriptorType::CharType,
            CompressedParsedDescriptorType::IntType => NonArrayCompressedParsedDescriptorType::IntType,
            CompressedParsedDescriptorType::LongType => NonArrayCompressedParsedDescriptorType::LongType,
            CompressedParsedDescriptorType::FloatType => NonArrayCompressedParsedDescriptorType::FloatType,
            CompressedParsedDescriptorType::DoubleType => NonArrayCompressedParsedDescriptorType::DoubleType,
            CompressedParsedDescriptorType::VoidType => NonArrayCompressedParsedDescriptorType::VoidType,
            CompressedParsedDescriptorType::Array { base_type, num_nested_arrs } => {
                return CompressedParsedDescriptorType::Array { base_type, num_nested_arrs: NonZeroU8::new(num_nested_arrs.get() + 1).unwrap() };
            }
            CompressedParsedDescriptorType::Class(class_name) => {
                NonArrayCompressedParsedDescriptorType::Class(class_name)
            }
        };
        CompressedParsedDescriptorType::Array {
            base_type: sub_type,
            num_nested_arrs: NonZeroU8::new(1).unwrap(),
        }
    }

    pub fn n_nested_arrays(sub_type: Self, n: NonZeroU8) -> Self {
        match sub_type {
            CompressedParsedDescriptorType::BooleanType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::BooleanType, num_nested_arrs: n },
            CompressedParsedDescriptorType::ByteType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::ByteType, num_nested_arrs: n },
            CompressedParsedDescriptorType::ShortType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::ShortType, num_nested_arrs: n },
            CompressedParsedDescriptorType::CharType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::CharType, num_nested_arrs: n },
            CompressedParsedDescriptorType::IntType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::IntType, num_nested_arrs: n },
            CompressedParsedDescriptorType::LongType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::LongType, num_nested_arrs: n },
            CompressedParsedDescriptorType::FloatType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::FloatType, num_nested_arrs: n },
            CompressedParsedDescriptorType::DoubleType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::DoubleType, num_nested_arrs: n },
            CompressedParsedDescriptorType::VoidType => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::VoidType, num_nested_arrs: n },
            CompressedParsedDescriptorType::Class(ccn) => CompressedParsedDescriptorType::Array { base_type: NonArrayCompressedParsedDescriptorType::Class(ccn), num_nested_arrs: n },
            CompressedParsedDescriptorType::Array { base_type, num_nested_arrs } => CompressedParsedDescriptorType::Array { base_type, num_nested_arrs: NonZeroU8::new(n.get() + num_nested_arrs.get()).unwrap() },
        }
    }

    pub fn object() -> Self {
        Self::Class(CompressedClassName::object())
    }

    pub fn class() -> Self {
        Self::Class(CompressedClassName::class())
    }

    pub fn is_double_or_long(&self) -> bool {
        match self {
            CompressedParsedDescriptorType::BooleanType => false,
            CompressedParsedDescriptorType::ByteType => false,
            CompressedParsedDescriptorType::ShortType => false,
            CompressedParsedDescriptorType::CharType => false,
            CompressedParsedDescriptorType::IntType => false,
            CompressedParsedDescriptorType::LongType => true,
            CompressedParsedDescriptorType::FloatType => false,
            CompressedParsedDescriptorType::DoubleType => true,
            CompressedParsedDescriptorType::VoidType => false,
            CompressedParsedDescriptorType::Class(_) => false,
            CompressedParsedDescriptorType::Array { .. } => false,
        }
    }

    pub fn from_ptype(ptype: &PType, pool: &CompressedClassfileStringPool) -> Self {
        match ptype {
            PType::ByteType => Self::ByteType,
            PType::CharType => Self::CharType,
            PType::DoubleType => Self::DoubleType,
            PType::FloatType => Self::FloatType,
            PType::IntType => Self::IntType,
            PType::LongType => Self::LongType,
            PType::Ref(ref_) => match ref_ {
                ReferenceType::Class(class_name) => CPDType::Class(CompressedClassName(pool.add_name(class_name.get_referred_name().to_string(), true))),
                ReferenceType::Array(arr) => CPDType::array(Self::from_ptype(arr.deref(), pool)),
            },
            PType::ShortType => Self::ShortType,
            PType::BooleanType => Self::BooleanType,
            PType::VoidType => Self::VoidType,
            PType::TopType => panic!(),
            PType::NullType => panic!(),
            PType::Uninitialized(_) => panic!(),
            PType::UninitializedThis => panic!(),
            PType::UninitializedThisOrClass(_) => panic!(),
        }
    }
}

impl From<CompressedClassName> for CompressedParsedDescriptorType {
    fn from(ccn: CompressedClassName) -> Self {
        CPDType::Class(ccn)
    }
}

impl From<CompressedParsedRefType> for CompressedParsedDescriptorType{
    fn from(value: CompressedParsedRefType) -> Self {
        value.to_cpdtype()
    }
}

pub type CMethodDescriptor = CompressedMethodDescriptor;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct CPDTypeOrderWrapper(pub CPDType);

//todo replace with a derive
impl Ord for CPDTypeOrderWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0 {
            CPDType::BooleanType => match other.0 {
                CPDType::BooleanType => Ordering::Equal,
                CPDType::ByteType => Ordering::Greater,
                CPDType::ShortType => Ordering::Greater,
                CPDType::CharType => Ordering::Greater,
                CPDType::IntType => Ordering::Greater,
                CPDType::LongType => Ordering::Greater,
                CPDType::FloatType => Ordering::Greater,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::ByteType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Equal,
                CPDType::ShortType => Ordering::Greater,
                CPDType::CharType => Ordering::Greater,
                CPDType::IntType => Ordering::Greater,
                CPDType::LongType => Ordering::Greater,
                CPDType::FloatType => Ordering::Greater,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::ShortType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Equal,
                CPDType::CharType => Ordering::Greater,
                CPDType::IntType => Ordering::Greater,
                CPDType::LongType => Ordering::Greater,
                CPDType::FloatType => Ordering::Greater,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::CharType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Equal,
                CPDType::IntType => Ordering::Greater,
                CPDType::LongType => Ordering::Greater,
                CPDType::FloatType => Ordering::Greater,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::IntType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Less,
                CPDType::IntType => Ordering::Equal,
                CPDType::LongType => Ordering::Greater,
                CPDType::FloatType => Ordering::Greater,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::LongType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Less,
                CPDType::IntType => Ordering::Less,
                CPDType::LongType => Ordering::Equal,
                CPDType::FloatType => Ordering::Greater,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::FloatType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Less,
                CPDType::IntType => Ordering::Less,
                CPDType::LongType => Ordering::Less,
                CPDType::FloatType => Ordering::Equal,
                CPDType::DoubleType => Ordering::Greater,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::DoubleType => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Less,
                CPDType::IntType => Ordering::Less,
                CPDType::LongType => Ordering::Less,
                CPDType::FloatType => Ordering::Less,
                CPDType::DoubleType => Ordering::Equal,
                CPDType::VoidType => Ordering::Greater,
                CPDType::Class(_) => Ordering::Greater,
                CPDType::Array { .. } => Ordering::Greater,
            },
            CPDType::VoidType => todo!(),
            CPDType::Class(ccn) => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Less,
                CPDType::IntType => Ordering::Less,
                CPDType::LongType => Ordering::Less,
                CPDType::FloatType => Ordering::Less,
                CPDType::DoubleType => Ordering::Less,
                CPDType::VoidType => Ordering::Less,
                CPDType::Class(ccn_other) => {
                    ccn.0.cmp(&ccn_other.0)
                }
                CPDType::Array { .. } => Ordering::Greater,
            }
            CPDType::Array { base_type: this_base_type, num_nested_arrs: this_num_nested_arrs } => match other.0 {
                CPDType::BooleanType => Ordering::Less,
                CPDType::ByteType => Ordering::Less,
                CPDType::ShortType => Ordering::Less,
                CPDType::CharType => Ordering::Less,
                CPDType::IntType => Ordering::Less,
                CPDType::LongType => Ordering::Less,
                CPDType::FloatType => Ordering::Less,
                CPDType::DoubleType => Ordering::Less,
                CPDType::VoidType => Ordering::Less,
                CPDType::Class(_) => Ordering::Less,
                CPDType::Array { base_type: other_base_type, num_nested_arrs: other_num_nested_arrs } => {
                    match this_num_nested_arrs.cmp(&other_num_nested_arrs) {
                        Ordering::Less => Ordering::Less,
                        Ordering::Equal => CPDTypeOrderWrapper(this_base_type.to_cpdtype()).cmp(&CPDTypeOrderWrapper(other_base_type.to_cpdtype())),
                        Ordering::Greater => Ordering::Greater
                    }
                }
            }
        }
    }
}

impl PartialOrd for CPDTypeOrderWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
