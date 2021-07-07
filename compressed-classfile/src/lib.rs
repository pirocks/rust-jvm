#![feature(box_syntax)]

use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use add_only_static_vec::AddOnlyIdMap;
use rust_jvm_common::classfile::{class_name, Classfile, ConstantKind, FieldInfo, MethodInfo, UninitializedVariableInfo};
use rust_jvm_common::descriptor_parser::{MethodDescriptor, parse_field_descriptor, parse_method_descriptor};
use rust_jvm_common::ptype::{PType, ReferenceType};

pub struct CompressedClassfileStringPool {
    pool: AddOnlyIdMap<String>,
}

static mut ONLY_ONE: bool = false;

impl CompressedClassfileStringPool {
    pub fn new() -> Self {
        unsafe {
            if ONLY_ONE {
                panic!("should only be one CompressedClassfileStringPool")
            }
            ONLY_ONE = true;
        }
        Self { pool: AddOnlyIdMap::new() }
    }

    pub fn add_name(&self, str: String) -> ClassfileString {
        let mut lock_guard = self.pool.lock().unwrap();
        let new_id = lock_guard.len();
        let id = *lock_guard.entry(str).or_insert(new_id);
        ClassfileString {
            id
        }
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct ClassfileString {
    id: usize,
}


impl ClassfileString {
    pub fn to_str(self, pool: &CompressedClassfileStringPool) -> &str {
        pool.pool[self.id].as_str()
    }
}

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct CompressedClassName(ClassfileString);

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

pub enum CompressedParsedRefType {
    Array(Box<CompressedParsedDescriptorType>),
    Object(CompressedClassName),
}

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
                        CompressedParsedRefType::Object(CompressedClassName(pool.add_name(class_name.get_referred_name().to_string())))
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

pub struct CompressedMethodDescriptor {
    arg_types: Vec<CompressedParsedDescriptorType>,
    return_type: CompressedParsedDescriptorType,
}

pub struct CompressedFieldInfo {
    access_flags: u16,
    name: CompressedClassName,
    descriptor_type: CompressedParsedDescriptorType,
    // attributes: Vec<AttributeInfo>,
}

pub struct CompressedMethodInfo {
    access_flags: u16,
    name: CompressedClassName,
    descriptor: CompressedMethodDescriptor,
}

pub struct CompressedClassfile {
    pub minor_version: u16,
    pub major_version: u16,
    // constant_pool: Vec<ConstantInfo>,
    pub access_flags: u16,
    pub this_class: CompressedClassName,
    pub super_class: Option<CompressedClassName>,
    pub interfaces: Vec<CompressedClassName>,
    fields: Vec<CompressedFieldInfo>,
    methods: Vec<CompressedMethodInfo>,
    // attributes: Vec<AttributeInfo>,
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
        let this = class_name(classfile).get_referred_name();
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
                name: CompressedClassName(pool.add_name(constant_pool[*name_index as usize].extract_string_from_utf8().to_string())),
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
                name: CompressedClassName(pool.add_name(constant_pool[*name_index as usize].extract_string_from_utf8().to_string())),
                descriptor: CompressedMethodDescriptor { arg_types, return_type },
            }
        }).collect_vec();
        Self {
            minor_version: *minor_version,
            major_version: *major_version,
            access_flags: *access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
        }
    }
}