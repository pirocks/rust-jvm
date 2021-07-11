use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Mutex, RwLock};

use itertools::Itertools;
use num_traits::one;

use add_only_static_vec::{AddOnlyId, AddOnlyIdMap, AddOnlyVecIDType};

use crate::classfile::{AttributeType, BootstrapMethods, Classfile, ConstantKind, FieldInfo, MethodInfo, UninitializedVariableInfo};
use crate::classnames::{class_name, ClassName};
use crate::descriptor_parser::{MethodDescriptor, parse_field_descriptor, parse_method_descriptor};
use crate::loading::{ClassWithLoader, LoaderName};
use crate::ptype::{PType, ReferenceType};
use crate::vtype::VType;

pub struct CompressedClassfileStringPool {
    pool: AddOnlyIdMap<String>,
}

static mut ONLY_ONE: bool = false;

impl CompressedClassfileStringPool {
    fn add_builtin_name(pool: &AddOnlyIdMap<String>, cname: ClassName, id: AddOnlyVecIDType) {
        let res = pool.push(cname.get_referred_name().to_string());
        assert_eq!(res, AddOnlyId(id));
    }

    pub fn new() -> Self {
        unsafe {
            if ONLY_ONE {
                panic!("should only be one CompressedClassfileStringPool")
            }
            ONLY_ONE = true;
        }
        let pool: AddOnlyIdMap<String> = AddOnlyIdMap::new();
        Self::add_builtin_name(&pool, ClassName::object(), COMPRESSED_ID_OBJECT);
        Self::add_builtin_name(&pool, ClassName::class(), COMPRESSED_ID_CLASS);
        Self::add_builtin_name(&pool, ClassName::method_handle(), COMPRESSED_ID_METHOD_HANDLE);
        Self::add_builtin_name(&pool, ClassName::throwable(), COMPRESSED_ID_THROWABLE);
        Self::add_builtin_name(&pool, ClassName::method_type(), COMPRESSED_ID_METHOD_TYPE);
        Self::add_builtin_name(&pool, ClassName::string(), COMPRESSED_ID_STRING);
        Self::add_builtin_name(&pool, ClassName::serializable(), COMPRESSED_ID_SERIALIZABLE);
        Self::add_builtin_name(&pool, ClassName::cloneable(), COMPRESSED_ID_CLONEABLE);
        Self { pool }
    }

    pub fn add_name(&self, str: impl Into<String>) -> CompressedClassfileString {
        let id = self.pool.push(str.into());
        CompressedClassfileString {
            id
        }
    }

    pub fn lookup(&self, id: CompressedClassfileString) -> &String {
        self.pool.lookup(id.id)
    }
}

pub type CCString = CompressedClassfileString;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct CompressedClassfileString {
    id: AddOnlyId,
}


impl CompressedClassfileString {
    pub fn to_str(&self, pool: &CompressedClassfileStringPool) -> String {
        pool.pool.lookup(self.id).to_string()
    }
}

pub const COMPRESSED_ID_OBJECT: AddOnlyVecIDType = 0;
pub const COMPRESSED_ID_CLASS: AddOnlyVecIDType = 1;
pub const COMPRESSED_ID_METHOD_HANDLE: AddOnlyVecIDType = 2;
pub const COMPRESSED_ID_THROWABLE: AddOnlyVecIDType = 3;
pub const COMPRESSED_ID_METHOD_TYPE: AddOnlyVecIDType = 4;
pub const COMPRESSED_ID_STRING: AddOnlyVecIDType = 5;
pub const COMPRESSED_ID_CLONEABLE: AddOnlyVecIDType = 6;
pub const COMPRESSED_ID_SERIALIZABLE: AddOnlyVecIDType = 7;
// pub const COMPRESSED_ID_OBJECT: AddOnlyVecIDType = 0;
// pub const COMPRESSED_ID_OBJECT: AddOnlyVecIDType = 0;
// pub const COMPRESSED_ID_OBJECT: AddOnlyVecIDType = 0;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct CompressedClassName(pub CompressedClassfileString);

pub type CClassName = CompressedClassName;

impl CompressedClassName {
    fn from_raw_id(raw_id: AddOnlyVecIDType) -> Self {
        Self {
            0: CompressedClassfileString { id: AddOnlyId(raw_id) }
        }
    }

    pub fn object() -> Self {
        Self::from_raw_id(COMPRESSED_ID_OBJECT)
    }

    pub fn class() -> Self {
        Self::from_raw_id(COMPRESSED_ID_CLASS)
    }

    pub fn method_handle() -> Self {
        Self::from_raw_id(COMPRESSED_ID_METHOD_HANDLE)
    }
    pub fn throwable() -> Self {
        Self::from_raw_id(COMPRESSED_ID_THROWABLE)
    }
    pub fn method_type() -> Self {
        Self::from_raw_id(COMPRESSED_ID_METHOD_TYPE)
    }
    pub fn string() -> Self {
        Self::from_raw_id(COMPRESSED_ID_STRING)
    }

    pub fn serializable() -> Self {
        Self::from_raw_id(COMPRESSED_ID_STRING)
    }

    pub fn cloneable() -> Self {
        Self::from_raw_id(COMPRESSED_ID_STRING)
    }
}

impl From<CompressedClassName> for CompressedParsedRefType {
    fn from(ccn: CompressedClassName) -> Self {
        Self::Class(ccn)
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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


#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum CompressedParsedRefType {
    Array(Box<CompressedParsedDescriptorType>),
    Class(CompressedClassName),
}

impl CompressedParsedRefType {
    pub fn unwrap_object_name(&self) -> CClassName {
        match self {
            CompressedParsedRefType::Array(_) => panic!(),
            CompressedParsedRefType::Class(ccn) => *ccn
        }
    }

    pub fn to_verification_type(&self, loader: LoaderName) -> VType {
        match self {
            CompressedParsedRefType::Array(arr) => {
                VType::ArrayReferenceType(arr.deref().clone())
            }
            CompressedParsedRefType::Class(obj) => {
                VType::Class(ClassWithLoader { class_name: *obj, loader })
            }
        }
    }

    pub fn unwrap_name(&self) -> CClassName {
        match self {
            CompressedParsedRefType::Array(_) => panic!(),
            CompressedParsedRefType::Class(ccn) => *ccn
        }
    }
}

pub type CPDType = CompressedParsedDescriptorType;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum CompressedParsedDescriptorType {
    BooleanType,
    ByteType,
    ShortType,
    CharType,
    IntType,
    LongType,
    FloatType,
    DoubleType,
    VoidType,
    Ref(CompressedParsedRefType),
}

impl CompressedParsedDescriptorType {
    pub fn unwrap_ref_type(&self) -> &CompressedParsedRefType {
        match self {
            CompressedParsedDescriptorType::Ref(ref_) => ref_,
            _ => panic!()
        }
    }

    pub fn unwrap_class_type(&self) -> CClassName {
        match self {
            CompressedParsedDescriptorType::Ref(ref_) => {
                match ref_ {
                    CompressedParsedRefType::Array(arr) => panic!(),
                    CompressedParsedRefType::Class(obj) => *obj
                }
            }
            _ => panic!()
        }
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
            CompressedParsedDescriptorType::Ref(ref_) => {
                match ref_ {
                    CompressedParsedRefType::Array(a) => VType::ArrayReferenceType(a.deref().clone()),
                    CompressedParsedRefType::Class(obj) => VType::Class(ClassWithLoader { class_name: *obj, loader })
                }
            },
        }
    }

    pub fn array(sub_type: Self) -> Self {
        Self::Ref(CPRefType::Array(box sub_type))
    }
    pub fn object() -> Self {
        Self::Ref(CPRefType::Class(CompressedClassName::object()))
    }
}

impl CompressedParsedDescriptorType {
    pub fn from_ptype(ptype: &PType, pool: &CompressedClassfileStringPool) -> Self {
        match ptype {
            PType::ByteType => Self::ByteType,
            PType::CharType => Self::CharType,
            PType::DoubleType => Self::DoubleType,
            PType::FloatType => Self::FloatType,
            PType::IntType => Self::IntType,
            PType::LongType => Self::LongType,
            PType::Ref(ref_) => {
                Self::Ref(match ref_ {
                    ReferenceType::Class(class_name) => {
                        CompressedParsedRefType::Class(CompressedClassName(pool.add_name(class_name.get_referred_name().to_string())))
                    }
                    ReferenceType::Array(arr) => {
                        CompressedParsedRefType::Array(box CompressedParsedDescriptorType::from_ptype(ptype, pool))
                    }
                })
            }
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
        Self::Ref(CompressedParsedRefType::Class(ccn))
    }
}

pub type CMethodDescriptor = CompressedMethodDescriptor;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct CompressedMethodDescriptor {
    pub arg_types: Vec<CompressedParsedDescriptorType>,
    pub return_type: CompressedParsedDescriptorType,
}

pub struct CompressedFieldInfo {
    pub access_flags: u16,
    pub name: CCString,
    pub descriptor_type: CompressedParsedDescriptorType,
    // attributes: Vec<AttributeInfo>,
}

pub struct CompressedMethodInfo {
    pub access_flags: u16,
    pub name: CompressedClassfileString,
    pub descriptor: CompressedMethodDescriptor,
}

pub struct CompressedClassfile {
    pub minor_version: u16,
    pub major_version: u16,
    // constant_pool: Vec<ConstantInfo>,
    pub access_flags: u16,
    pub this_class: CompressedClassName,
    pub super_class: Option<CompressedClassName>,
    pub interfaces: Vec<CompressedClassName>,
    pub fields: Vec<CompressedFieldInfo>,
    pub methods: Vec<CompressedMethodInfo>,
    // attributes: Vec<AttributeInfo>,
    pub bootstrap_methods: Option<CompressedBootstrapMethods>,
}

pub struct CompressedBootstrapMethods {
    inner: BootstrapMethods,
}

impl CompressedClassfile {
    pub fn new(pool: &CompressedClassfileStringPool, classfile: &Classfile) -> Self {
        let Classfile {
            magic,
            minor_version,
            major_version,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes
        } = classfile;
        let super_class = classfile.super_class_name().map(|name| CompressedClassName(pool.add_name(name.get_referred_name().to_string())));
        let this = class_name(classfile).get_referred_name().to_string();
        let this_class = CompressedClassName(pool.add_name(this.to_string()));

        let interfaces = interfaces.iter().map(|interface| {
            let interface = *interface as usize;
            match &constant_pool[interface].kind {
                ConstantKind::Class(c) => {
                    CompressedClassName(pool.add_name(constant_pool[c.name_index as usize].extract_string_from_utf8()))
                }
                _ => panic!()
            }
        }).collect_vec();

        let fields = fields.iter().map(|field_info| {
            let FieldInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes
            } = field_info;
            let desc_str = classfile.constant_pool[*descriptor_index as usize].extract_string_from_utf8();
            let parsed = parse_field_descriptor(desc_str.as_str()).unwrap();
            CompressedFieldInfo {
                access_flags: *access_flags,
                name: pool.add_name(constant_pool[*name_index as usize].extract_string_from_utf8().to_string()),
                descriptor_type: CompressedParsedDescriptorType::from_ptype(&parsed.field_type, pool),
            }
        }).collect_vec();
        let methods = methods.iter().map(|method_info| {
            let MethodInfo {
                access_flags,
                name_index,
                descriptor_index,
                attributes
            } = method_info;
            let MethodDescriptor { parameter_types, return_type } = parse_method_descriptor(constant_pool[*descriptor_index as usize].extract_string_from_utf8().as_str()).unwrap();
            let return_type = CompressedParsedDescriptorType::from_ptype(&return_type, pool);
            let arg_types = parameter_types.iter().map(|ptype| CompressedParsedDescriptorType::from_ptype(ptype, pool)).collect_vec();
            CompressedMethodInfo {
                access_flags: *access_flags,
                name: pool.add_name(constant_pool[*name_index as usize].extract_string_from_utf8().to_string()),
                descriptor: CompressedMethodDescriptor { arg_types, return_type },
            }
        }).collect_vec();
        let bootstrap_methods = classfile.attributes.iter().find_map(|x| {
            match &x.attribute_type {
                AttributeType::BootstrapMethods(bm) => Some(bm.clone()),
                _ => None
            }
        });
        Self {
            minor_version: *minor_version,
            major_version: *major_version,
            access_flags: *access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            bootstrap_methods: (|| { Some(CompressedBootstrapMethods { inner: bootstrap_methods? }) })(),
        }
    }
}
