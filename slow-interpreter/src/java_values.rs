use crate::runtime_class::RuntimeClass;
use std::sync::Arc;
use std::iter::Map;
use rust_jvm_common::unified_types::ParsedType;
//use std::alloc::{alloc, dealloc, Layout};

pub enum JavaValue{
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(bool),
    Char(char),

    Float(f32),
    Double(f64),

    Array(Vec<JavaValue>),
    Object(Option<ObjectPointer>),

    Top//should never be interacted with by the bytecode
}

pub struct ObjectPointer{
    object: *const Object
}

pub struct Object {
    gc_reachable: bool,
    class_pointer: Arc<RuntimeClass>,//I guess this never changes so uneeded?
    fields : Map<String,JavaValue>
}

pub fn default_value(type_ : ParsedType)-> JavaValue{
    match type_{
        ParsedType::ByteType => JavaValue::Byte(0),
        ParsedType::CharType => JavaValue::Char('\u{000000}'),
        ParsedType::DoubleType => JavaValue::Double(0.0),
        ParsedType::FloatType => JavaValue::Float(0.0),
        ParsedType::IntType => JavaValue::Int(0),
        ParsedType::LongType => JavaValue::Long(0),
        ParsedType::Class(_) => JavaValue::Object(None),
        ParsedType::ShortType => JavaValue::Short(0),
        ParsedType::BooleanType => JavaValue::Boolean(false),
        ParsedType::ArrayReferenceType(_) => JavaValue::Object(None),
        ParsedType::VoidType => panic!(),
        ParsedType::TopType => JavaValue::Top,
        ParsedType::NullType => JavaValue::Object(None),
        ParsedType::Uninitialized(_) => unimplemented!(),
        ParsedType::UninitializedThis => unimplemented!(),
    }
}