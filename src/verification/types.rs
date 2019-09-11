use std::io::Write;
use std::io;
use verification::{PrologGenContext, BOOTSTRAP_LOADER_NAME};

pub struct Byte {}

pub struct Char {}

pub struct Double {}

pub struct Float {}

pub struct Int {}

pub struct Long {}

pub struct Reference {
    pub class_name: Box<String>
}

pub struct Short {}

pub struct Boolean {}

pub struct ArrayReference {
    pub sub_type: Box<Type>
}

pub struct Void {}

pub enum Type {
    ByteType(Byte),
    CharType(Char),
    DoubleType(Double),
    FloatType(Float),
    IntType(Int),
    LongType(Long),
    ReferenceType(Reference),
    ShortType(Short),
    BooleanType(Boolean),
    ArrayReferenceType(ArrayReference),
    VoidType(Void),
}

pub struct MethodDescriptor{ pub parameter_types: Vec<Type>, pub return_type: Type }

pub struct FieldDescriptor{ pub field_type: Type }

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
        '[' => {
            let str_without_brace = eat_one(str_);
            let end_index = str_without_brace.find(';').expect("unterminated object in descriptor");
            assert!(str_.chars().nth(end_index).expect("") == ';');
            let class_name = str_[0..end_index].to_string();
            dbg!(&class_name);
            let remaining_to_parse = &str_without_brace[end_index..str_without_brace.len()];
            Some((remaining_to_parse, Type::ReferenceType(Reference { class_name:Box::new(class_name) })))
        }
        _ => return None
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
            parse_object_type(str_)
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
    while remaining_to_parse.chars().nth(0)? == ')' {
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

pub fn write_type_prolog(context: &PrologGenContext,type_: Type,  w: &mut dyn Write) -> Result<(), io::Error>{
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
            if context.using_bootsrap_loader {
                write!(w,"class('")?;
                write!(w,"{}",ref_.class_name)?;
                write!(w,"',{})",BOOTSTRAP_LOADER_NAME)?;
            } else {
                unimplemented!()
            }
        },
        Type::ArrayReferenceType(arr) => {
            write!(w, "arrayOf(")?;
            write_type_prolog(context, *arr.sub_type, w)?;
            write!(w, ")")?;
        },
        Type::VoidType(_) => {
            write!(w,"void")?;
        },
    }
    Ok(())
}