use rust_jvm_common::unified_types::{UnifiedType, ArrayType};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::unified_types::ClassType;
use rust_jvm_common::loading::Loader;
use std::sync::Arc;

//todo there are two of these, only one can live forever
#[derive(Debug)]
pub struct MethodDescriptor { pub parameter_types: Vec<UnifiedType>, pub return_type: UnifiedType }

pub struct FieldDescriptor { pub field_type: UnifiedType }

pub fn eat_one(str_: &str) -> &str {
    &str_[1..str_.len()]
}

pub fn parse_base_type(str_: &str) -> Option<(&str, UnifiedType)> {
    Some((eat_one(str_), match str_.chars().nth(0)? {
        'B' => UnifiedType::ByteType,
        'C' => UnifiedType::CharType,
        'D' => UnifiedType::DoubleType,
        'F' => UnifiedType::FloatType,
        'I' => UnifiedType::IntType,
        'J' => UnifiedType::LongType,
        'S' => UnifiedType::ShortType,
        'Z' => UnifiedType::BooleanType,
        _ => return None
    }))
}

pub fn parse_object_type<'a, 'b>(loader: &'a Arc<Loader>, str_: &'b str) -> Option<(&'b str, UnifiedType)> {
    match str_.chars().nth(0)? {
        'L' => {
            let str_without_l = eat_one(str_);
            let end_index = str_without_l.find(';').expect("unterminated object in descriptor") + 1;
            assert_eq!(str_without_l.chars().nth(end_index - 1).expect(""), ';');
            let class_name = &str_without_l[0..end_index - 1];
            let remaining_to_parse = &str_without_l[(end_index)..str_without_l.len()];
            let class_name = ClassName::Str(class_name.to_string());
            Some((remaining_to_parse, UnifiedType::Class(ClassType { class_name,  loader:loader.clone()  })))
        }
        _ => {
            return None;
        }
    }
}

pub fn parse_array_type<'a, 'b>(loader: &'a Arc<Loader>, str_: &'b str) -> Option<(&'b str, UnifiedType)> {
    match str_.chars().nth(0)? {
        '[' => {
            let (remaining_to_parse, sub_type) = parse_component_type(loader,&str_[1..str_.len()])?;
            let array_type = UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::from(sub_type) });
            Some((remaining_to_parse, array_type))
        }
        _ => None
    }
}

pub fn parse_field_type<'a, 'b>(loader: &'a Arc<Loader>, str_: &'b str) -> Option<(&'b str, UnifiedType)> {
    parse_array_type(loader,str_).or_else(|| {
        parse_base_type(str_).or_else(|| {
            parse_object_type(loader,str_).or_else(|| {
                panic!("{}", str_)
            })
        })
    })
}


pub fn parse_field_descriptor(loader: &Arc<Loader>,str_: &str) -> Option<FieldDescriptor> {
    if let Some((should_be_empty, field_type)) = parse_field_type(loader,str_) {
        if should_be_empty.is_empty() {
            Some(FieldDescriptor { field_type })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_component_type<'a, 'b>(loader: &'a Arc<Loader>, str_: &'b str) -> Option<(&'b str, UnifiedType)> {
    parse_field_type(loader,str_)
}

pub fn parse_method_descriptor(loader: &Arc<Loader>,str_: &str) -> Option<MethodDescriptor> {
    if str_.chars().nth(0)? != '(' {
        return None;
    }
    let mut remaining_to_parse = eat_one(str_);
    let mut parameter_types = Vec::new();
    while remaining_to_parse.chars().nth(0)? != ')' {
        if let Some((rem, type_)) = parse_field_type(loader,remaining_to_parse) {
            remaining_to_parse = rem;
            parameter_types.push(type_);
        } else {
            return None;
        }
    }
    remaining_to_parse = eat_one(remaining_to_parse);
    if let Some((should_be_empty, return_type)) = parse_return_descriptor(loader,remaining_to_parse) {
        if should_be_empty.is_empty() {
            Some(MethodDescriptor { return_type, parameter_types })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_parameter_descriptor<'a, 'b>(loader: &'a Arc<Loader>, str_: &'b str) -> Option<(&'b str, UnifiedType)> {
    parse_field_type(loader,str_)
}

pub fn parse_void_descriptor(str_: &str) -> Option<(&str, UnifiedType)> {
    match str_.chars().nth(0)? {
        'V' => Some((eat_one(str_), UnifiedType::VoidType)),
        _ => return None
    }
}

pub fn parse_return_descriptor<'a, 'b>(loader: &'a Arc<Loader>, str_: &'b str) -> Option<(&'b str, UnifiedType)> {
    parse_void_descriptor(str_).or_else(|| {
        parse_field_type(loader,str_)
    })
}