use std::io::Write;
use std::io;
use verification::{PrologGenContext, BOOTSTRAP_LOADER_NAME};

#[derive(Debug)]
pub struct Byte {}

#[derive(Debug)]
pub struct Char {}

#[derive(Debug)]
pub struct Double {}

#[derive(Debug)]
pub struct Float {}

#[derive(Debug)]
pub struct Int {}

#[derive(Debug)]
pub struct Long {}

#[derive(Debug)]
pub struct Reference<'l> {
    pub class_name: &'l str
}

#[derive(Debug)]
pub struct Short {}

#[derive(Debug)]
pub struct Boolean {}

#[derive(Debug)]
pub struct ArrayReference<'l> {
    pub sub_type: Box<Type<'l>>
}

#[derive(Debug)]
pub struct Void {}

#[derive(Debug)]
pub enum Type<'l> {
    ByteType(Byte),
    CharType(Char),
    DoubleType(Double),
    FloatType(Float),
    IntType(Int),
    LongType(Long),
    ReferenceType(Reference<'l>),
    ShortType(Short),
    BooleanType(Boolean),
    ArrayReferenceType(ArrayReference<'l>),
    VoidType(Void),
}

#[derive(Debug)]
pub struct MethodDescriptor<'l>{ pub parameter_types: Vec<Type<'l>>, pub return_type: Type<'l> }

pub struct FieldDescriptor<'l>{ pub field_type: Type<'l> }

pub fn eat_one(str_: &str) -> &str {
    &str_[1..str_.len()]
}

pub fn parse_base_type(str_: &str) -> Option<(&str, Type)> {
    Some((eat_one(str_), match str_.chars().nth(0)? {
        'B' => Type::ByteType(Byte {}),
        'C' => Type::CharType(Char {}),
        'D' => Type::DoubleType(Double {}),
        'F' => Type::FloatType(Float {}),
        'I' => Type::IntType(Int {}),
        'J' => Type::LongType(Long {}),
        'S' => Type::ShortType(Short {}),
        'Z' => Type::BooleanType(Boolean {}),
        _ => return None
    }))
}

pub fn parse_object_type(str_: &str) -> Option<(&str, Type)> {
    match str_.chars().nth(0)? {
        'L' => {
            let str_without_l = eat_one(str_);
            let end_index = str_without_l.find(';').expect("unterminated object in descriptor") + 1;
            assert_eq!(str_without_l.chars().nth(end_index - 1).expect(""), ';');
            let class_name = &str_without_l[0..end_index - 1];
//            dbg!(&class_name);
            let remaining_to_parse = &str_without_l[(end_index)..str_without_l.len()];
            Some((remaining_to_parse, Type::ReferenceType(Reference { class_name })))
        }
        _ => {
            return None
        }
    }
}

pub fn parse_array_type(str_: &str) -> Option<(&str, Type)> {
    match str_.chars().nth(0)? {
        '[' => {
            let (remaining_to_parse,sub_type) = parse_component_type(&str_[1..str_.len()])?;
            let array_type = Type::ArrayReferenceType(ArrayReference { sub_type: Box::from(sub_type) });
            Some((remaining_to_parse,array_type))
        }
        _ => None
    }
}

pub fn parse_field_type(str_: &str) -> Option<(&str, Type)> {
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

pub fn parse_component_type(str_: &str) -> Option<(&str, Type)> {
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

pub fn parse_parameter_descriptor(str_: &str) -> Option<(&str, Type)> {
    parse_field_type(str_)
}

pub fn parse_void_descriptor(str_: &str) -> Option<(&str, Type)> {
    match str_.chars().nth(0)? {
        'V' => Some((eat_one(str_),Type::VoidType(Void {}))),
        _ => return None
    }
}

pub fn parse_return_descriptor(str_: &str) -> Option<(&str, Type)> {
    parse_void_descriptor(str_).or_else(|| {
        parse_field_type(str_)
    })
}

pub fn write_type_prolog(context: &PrologGenContext,type_: &Type,  w: &mut dyn Write) -> Result<(), io::Error>{
    match type_{
        Type::ByteType(_) => {
            write!(w,"byte")?;
        },
        Type::CharType(_) => {
            write!(w,"char")?;
        },
        Type::DoubleType(_) => {
            write!(w,"double")?;
        },
        Type::FloatType(_) => {
            write!(w,"float")?;
        },
        Type::IntType(_) => {
            write!(w,"int")?;
        },
        Type::LongType(_) => {
            write!(w,"long")?;
        },
        Type::ShortType(_) => {
            write!(w,"short")?;
        },
        Type::BooleanType(_) => {
            write!(w,"boolean")?;
        },
        Type::ReferenceType(ref_) => {
            if context.state.using_bootstrap_loader {
                write!(w,"class('")?;
                write!(w,"{}",ref_.class_name)?;
                write!(w,"',{})",BOOTSTRAP_LOADER_NAME)?;
            } else {
                unimplemented!()
            }
        },
        Type::ArrayReferenceType(arr) => {
            write!(w, "arrayOf(")?;
            write_type_prolog(context, &arr.sub_type, w)?;
            write!(w, ")")?;
        },
        Type::VoidType(_) => {
            write!(w,"void")?;
        },
    }
    Ok(())
}