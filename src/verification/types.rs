use std::io::Write;
use std::io;
use verification::prolog_info_writer::BOOTSTRAP_LOADER_NAME;
use verification::unified_type::UnifiedType;
use verification::unified_type::ArrayType;
use verification::classnames::{ClassName, get_referred_name};

#[derive(Debug)]
pub struct MethodDescriptor{ pub parameter_types: Vec<UnifiedType>, pub return_type: UnifiedType }

pub struct FieldDescriptor{ pub field_type: UnifiedType }

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

pub fn parse_object_type(str_: &str) -> Option<(&str, UnifiedType)> {
    match str_.chars().nth(0)? {
        'L' => {
            let str_without_l = eat_one(str_);
            let end_index = str_without_l.find(';').expect("unterminated object in descriptor") + 1;
            assert_eq!(str_without_l.chars().nth(end_index - 1).expect(""), ';');
            let class_name = &str_without_l[0..end_index - 1];
            let remaining_to_parse = &str_without_l[(end_index)..str_without_l.len()];
            Some((remaining_to_parse, UnifiedType::ReferenceType(ClassName::Str(class_name.to_string()))))
        }
        _ => {
            return None
        }
    }
}

pub fn parse_array_type(str_: &str) -> Option<(&str, UnifiedType)> {
    match str_.chars().nth(0)? {
        '[' => {
            let (remaining_to_parse,sub_type) = parse_component_type(&str_[1..str_.len()])?;
            let array_type = UnifiedType::ArrayReferenceType(ArrayType { sub_type: Box::from(sub_type) });
            Some((remaining_to_parse,array_type))
        }
        _ => None
    }
}

pub fn parse_field_type(str_: &str) -> Option<(&str, UnifiedType)> {
    parse_array_type(str_).or_else(|| {
        parse_base_type(str_).or_else(|| {
            parse_object_type(str_).or_else(|| {
                panic!("{}",str_)

            })
        })
    })
}


pub fn parse_field_descriptor(str_: &str) -> Option<FieldDescriptor> {
    if let Some((should_be_empty,field_type)) = parse_field_type(str_){
        if should_be_empty.is_empty() {
            Some(FieldDescriptor{field_type})
        } else {
            None
        }
    }else {
        None
    }
}

pub fn parse_component_type(str_: &str) -> Option<(&str, UnifiedType)> {
    parse_field_type(str_)
}

pub fn parse_method_descriptor(str_: &str) -> Option<MethodDescriptor> {
    if str_.chars().nth(0)? != '(' {
        return None
    }
    let mut remaining_to_parse = eat_one(str_);
    let mut parameter_types = Vec::new();
    while remaining_to_parse.chars().nth(0)? != ')' {
        if let Some((rem,type_)) = parse_field_type(remaining_to_parse){
            remaining_to_parse = rem;
            parameter_types.push(type_);
        }else {
            return None
        }
    }
    remaining_to_parse = eat_one(remaining_to_parse);
    if let Some ((should_be_empty,return_type)) = parse_return_descriptor(remaining_to_parse){
        if should_be_empty.is_empty() {
            Some(MethodDescriptor{ return_type, parameter_types})
        } else {
            None
        }
    }else {
        None
    }
}

pub fn parse_parameter_descriptor(str_: &str) -> Option<(&str, UnifiedType)> {
    parse_field_type(str_)
}

pub fn parse_void_descriptor(str_: &str) -> Option<(&str, UnifiedType)> {
    match str_.chars().nth(0)? {
        'V' => Some((eat_one(str_),UnifiedType::VoidType)),
        _ => return None
    }
}

pub fn parse_return_descriptor(str_: &str) -> Option<(&str, UnifiedType)> {
    parse_void_descriptor(str_).or_else(|| {
        parse_field_type(str_)
    })
}

pub fn write_type_prolog(type_: &UnifiedType,  w: &mut dyn Write) -> Result<(), io::Error>{
    match type_{
        UnifiedType::ByteType => {
            write!(w,"int")?;
        },
        UnifiedType::CharType => {
            write!(w,"int")?;
        },
        UnifiedType::DoubleType => {
            write!(w,"double")?;
        },
        UnifiedType::FloatType => {
            write!(w,"float")?;
        },
        UnifiedType::IntType => {
            write!(w,"int")?;
        },
        UnifiedType::LongType => {
            write!(w,"long")?;
        },
        UnifiedType::ShortType => {
            write!(w,"int")?;
        },
        UnifiedType::BooleanType => {
            write!(w,"int")?;
        },
        UnifiedType::ReferenceType(ref_) => {
//            if context.state.using_bootstrap_loader {
                write!(w,"class('")?;
                write!(w,"{}",get_referred_name(ref_))?;
                write!(w,"',{})",BOOTSTRAP_LOADER_NAME)?;
//            } else {
//                unimplemented!()
//            }
        },
        UnifiedType::ArrayReferenceType(arr) => {
            write!(w, "arrayOf(")?;
            write_type_prolog(&arr.sub_type, w)?;
            write!(w, ")")?;
        },
        UnifiedType::VoidType => {
            write!(w,"void")?;
        },
        _ => {panic!("Case wasn't coverred with non-unified types")}
    }
    Ok(())
}