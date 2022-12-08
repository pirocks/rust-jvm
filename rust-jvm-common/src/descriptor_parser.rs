use std::hash::{Hash, Hasher};

use itertools::Itertools;

use crate::classfile::{Classfile, MethodInfo};
use crate::classnames::ClassName;

use crate::compressed_classfile::compressed_descriptors::CFieldDescriptor;
use crate::compressed_classfile::compressed_types::CMethodDescriptor;
use crate::ptype::{PType, ReferenceType};

#[derive(Debug, Eq, Clone)]
pub struct MethodDescriptor {
    pub parameter_types: Vec<PType>,
    pub return_type: PType,
}

pub struct MethodDescriptorCache {}

impl MethodDescriptor {
    pub fn from_legacy(method_info: &MethodInfo, classfile: &Classfile) -> Self {
        parse_method_descriptor(method_info.descriptor_str(classfile).as_str()).unwrap()
        //todo get rid of this unwrap and this function
    }

    pub fn jvm_representation(&self) -> String {
        format!("({}){}", self.parameter_types.iter().map(|param| param.jvm_representation()).join(""), self.return_type.jvm_representation())
    }
}

impl PartialEq for MethodDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.parameter_types == other.parameter_types && self.return_type == other.return_type
    }
}

impl Hash for MethodDescriptor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for ptype in &self.parameter_types {
            ptype.hash(state)
        }
        self.return_type.hash(state)
    }
}

#[derive(Debug)]
pub struct FieldDescriptor {
    pub field_type: PType,
}

#[derive(Debug)]
pub enum Descriptor<'l> {
    Method(&'l CMethodDescriptor),
    Field(&'l CFieldDescriptor),
}

pub fn eat_one(str_: &str) -> &str {
    &str_[1..str_.len()]
}

pub fn parse_base_type(str_: &str) -> Option<(&str, PType)> {
    Some((
        eat_one(str_),
        match str_.chars().next()? {
            'B' => PType::ByteType,
            'C' => PType::CharType,
            'D' => PType::DoubleType,
            'F' => PType::FloatType,
            'I' => PType::IntType,
            'J' => PType::LongType,
            'S' => PType::ShortType,
            'Z' => PType::BooleanType,
            _ => return None,
        },
    ))
}

pub fn parse_object_type(str_: &str) -> Option<(&str, PType)> {
    match str_.chars().next()? {
        'L' => {
            let str_without_l = eat_one(str_);
            let end_index = str_without_l.find(';').expect("unterminated object in descriptor") + 1; //todo this needs to be a result
            assert_eq!(str_without_l.chars().nth(end_index - 1).expect(""), ';');
            let class_name = &str_without_l[0..end_index - 1];
            let remaining_to_parse = &str_without_l[(end_index)..str_without_l.len()];
            let sub_type = if class_name == "int" {
                PType::IntType
            } else if class_name == "boolean" {
                PType::BooleanType
            } else if class_name == "double" {
                PType::DoubleType
            } else if class_name == "float" {
                PType::FloatType
            } else if class_name == "long" {
                PType::LongType
            } else if class_name == "void" {
                PType::VoidType
            } else if class_name == "short" {
                PType::ShortType
            } else if class_name == "char" {
                PType::CharType
            } else {
                let class_name = ClassName::Str(class_name.to_string());
                PType::Ref(ReferenceType::Class(class_name))
            };
            Some((remaining_to_parse, sub_type))
        }
        _ => None,
    }
}

pub fn parse_array_type(str_: &str) -> Option<(&str, PType)> {
    match str_.chars().next()? {
        '[' => {
            let (remaining_to_parse, sub_type) = parse_component_type(&str_[1..str_.len()])?;
            let array_type = PType::Ref(ReferenceType::Array(Box::from(sub_type)));
            Some((remaining_to_parse, array_type))
        }
        _ => None,
    }
}

pub fn parse_field_type(str_: &str) -> Option<(&str, PType)> {
    parse_array_type(str_).or_else(|| {
        parse_base_type(str_).or_else(|| {
            parse_object_type(str_).or_else(|| {
                ("", PType::Ref(ReferenceType::Class(ClassName::Str(str_.to_string())))).into()
                //todo fallback for when parsing maformedtypes names
            })
        })
    })
}

pub fn parse_field_descriptor(str_: &str) -> Option<FieldDescriptor> {
    if let Some((should_be_empty, field_type)) = parse_field_type(str_) {
        if should_be_empty.is_empty() {
            Some(FieldDescriptor { field_type })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_component_type(str_: &str) -> Option<(&str, PType)> {
    parse_field_type(str_)
}

pub fn parse_method_descriptor(str_: &str) -> Option<MethodDescriptor> {
    if str_.chars().next()? != '(' {
        return None;
    }
    let mut remaining_to_parse = eat_one(str_);
    let mut parameter_types = Vec::new();
    while remaining_to_parse.chars().next()? != ')' {
        if let Some((rem, type_)) = parse_field_type(remaining_to_parse) {
            remaining_to_parse = rem;
            parameter_types.push(type_);
        } else {
            return None;
        }
    }
    remaining_to_parse = eat_one(remaining_to_parse);
    if let Some((should_be_empty, return_type)) = parse_return_descriptor(remaining_to_parse) {
        if should_be_empty.is_empty() {
            Some(MethodDescriptor { return_type, parameter_types })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_parameter_descriptor(str_: &str) -> Option<(&str, PType)> {
    parse_field_type(str_)
}

pub fn parse_void_descriptor(str_: &str) -> Option<(&str, PType)> {
    match str_.chars().next()? {
        'V' => Some((eat_one(str_), PType::VoidType)),
        _ => None,
    }
}

pub fn parse_return_descriptor(str_: &str) -> Option<(&str, PType)> {
    parse_void_descriptor(str_).or_else(|| parse_field_type(str_))
}

pub fn parse_class_name(str_: &str) -> PType {
    if let Some(subtype_descriptor) = str_.strip_prefix('[') {
        let field_descriptor = parse_field_descriptor(subtype_descriptor).expect("Invalid field descriptor but this should have been validated when classnames etc where validated").field_type;
        PType::Ref(ReferenceType::Array(box field_descriptor))
    } else if str_.ends_with(';') {
        parse_field_descriptor(str_).expect("Invalid field descriptor but this should have been validated when classnames etc where validated").field_type
    } else {
        PType::Ref(ReferenceType::Class(ClassName::Str(str_.to_string())))
    }
}
