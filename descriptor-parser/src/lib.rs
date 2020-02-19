use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::classfile::{MethodInfo, Classfile};
use rust_jvm_common::view::ptype_view::{PTypeView, ReferenceTypeView};


#[derive(Debug)]
pub struct MethodDescriptor { pub parameter_types: Vec<PTypeView>, pub return_type: PTypeView }

impl MethodDescriptor {
    pub fn from(method_info: &MethodInfo, classfile: &Classfile) -> Self {
        parse_method_descriptor( method_info.descriptor_str(classfile).as_str()).unwrap()
    }
}

impl PartialEq for MethodDescriptor{
    fn eq(&self, other: &Self) -> bool {
        self.parameter_types == other.parameter_types &&
            self.return_type == other.return_type
    }
}

#[derive(Debug)]
pub struct FieldDescriptor { pub field_type: PTypeView }


#[derive(Debug)]
pub enum Descriptor<'l> {
    Method(&'l MethodDescriptor),
    Field(&'l FieldDescriptor),
}

pub fn eat_one(str_: &str) -> &str {
    &str_[1..str_.len()]
}

pub fn parse_base_type(str_: &str) -> Option<(&str, PTypeView)> {
    Some((eat_one(str_), match str_.chars().nth(0)? {
        'B' => PTypeView::ByteType,
        'C' => PTypeView::CharType,
        'D' => PTypeView::DoubleType,
        'F' => PTypeView::FloatType,
        'I' => PTypeView::IntType,
        'J' => PTypeView::LongType,
        'S' => PTypeView::ShortType,
        'Z' => PTypeView::BooleanType,
        _ => return None
    }))
}

pub fn parse_object_type(str_: &str) -> Option<(&str, PTypeView)> {
    match str_.chars().nth(0)? {
        'L' => {
            let str_without_l = eat_one(str_);
            let end_index = str_without_l.find(';').expect("unterminated object in descriptor") + 1;
            assert_eq!(str_without_l.chars().nth(end_index - 1).expect(""), ';');
            let class_name = &str_without_l[0..end_index - 1];
            let remaining_to_parse = &str_without_l[(end_index)..str_without_l.len()];
            let class_name = ClassName::Str(class_name.to_string());
            Some((remaining_to_parse, PTypeView::Ref(ReferenceTypeView::Class(class_name))))
        }
        _ => {
            return None;
        }
    }
}

pub fn parse_array_type(str_: &str) -> Option<(&str, PTypeView)> {
    match str_.chars().nth(0)? {
        '[' => {
            let (remaining_to_parse, sub_type) = parse_component_type(&str_[1..str_.len()])?;
            let array_type = PTypeView::Ref(ReferenceTypeView::Array(Box::from(sub_type)));
            Some((remaining_to_parse, array_type))
        }
        _ => None
    }
}

pub fn parse_field_type(str_: &str) -> Option<(&str, PTypeView)> {
    parse_array_type( str_).or_else(|| {
        parse_base_type(str_).or_else(|| {
            parse_object_type( str_).or_else(|| {
                panic!("{}", str_)
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

pub fn parse_component_type(str_: &str) -> Option<(&str, PTypeView)> {
    parse_field_type(str_)
}

pub fn parse_method_descriptor(str_: &str) -> Option<MethodDescriptor> {
    if str_.chars().nth(0)? != '(' {
        return None;
    }
    let mut remaining_to_parse = eat_one(str_);
    let mut parameter_types = Vec::new();
    while remaining_to_parse.chars().nth(0)? != ')' {
        if let Some((rem, type_)) = parse_field_type(remaining_to_parse) {
            remaining_to_parse = rem;
            parameter_types.push(type_);
        } else {
            return None;
        }
    }
    remaining_to_parse = eat_one(remaining_to_parse);
    if let Some((should_be_empty, return_type)) = parse_return_descriptor( remaining_to_parse) {
        if should_be_empty.is_empty() {
            Some(MethodDescriptor { return_type, parameter_types })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_parameter_descriptor(str_: &str) -> Option<(&str, PTypeView)> {
    parse_field_type( str_)
}

pub fn parse_void_descriptor(str_: &str) -> Option<(&str, PTypeView)> {
    match str_.chars().nth(0)? {
        'V' => Some((eat_one(str_), PTypeView::VoidType)),
        _ => return None
    }
}

pub fn parse_return_descriptor(str_: &str) -> Option<(&str, PTypeView)> {
    parse_void_descriptor(str_).or_else(|| {
        parse_field_type( str_)
    })
}

