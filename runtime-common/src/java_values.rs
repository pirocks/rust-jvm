use crate::runtime_class::RuntimeClass;
use std::sync::Arc;
use std::iter::Map;
use rust_jvm_common::unified_types::ParsedType;
use rust_jvm_common::classfile::ConstantInfo;
use rust_jvm_common::classfile::ConstantKind;
use std::mem::transmute;
//use std::alloc::{alloc, dealloc, Layout};

#[derive(Debug)]
pub enum JavaValue {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(bool),
    Char(char),

    Float(f32),
    Double(f64),

    Array(Option<VecPointer>),
    Object(Option<ObjectPointer>),

    Top,//should never be interacted with by the bytecode
}

impl Clone for JavaValue{
    fn clone(&self) -> Self {
        match self {
            JavaValue::Long(l) => JavaValue::Long(*l),
            JavaValue::Int(i) => JavaValue::Int(*i),
            JavaValue::Short(s) => JavaValue::Short(*s),
            JavaValue::Byte(b) => JavaValue::Byte(*b),
            JavaValue::Boolean(b) => JavaValue::Boolean(*b),
            JavaValue::Char(c) => JavaValue::Char(*c),
            JavaValue::Float(f) => JavaValue::Float(*f),
            JavaValue::Double(d) => JavaValue::Double(*d),
            JavaValue::Array(a) => JavaValue::Array(a.clone()),
            JavaValue::Object(o) => JavaValue::Object(o.clone()),
            JavaValue::Top => JavaValue::Top,
        }
    }
}

#[derive(Debug)]
pub struct ObjectPointer {
    object: *const Object
}

impl Clone for ObjectPointer{
    fn clone(&self) -> Self {
        ObjectPointer { object: self.object }
    }
}

#[derive(Debug)]
pub struct VecPointer {
    pub object: *const Vec<JavaValue>
}

impl Clone for VecPointer{
    fn clone(&self) -> Self {
        VecPointer { object: self.object }
    }
}

pub struct Object {
    gc_reachable: bool,
    class_pointer: Arc<RuntimeClass>,
    //I guess this never changes so unneeded?
    fields: Map<String, JavaValue>,
}

pub fn default_value(type_: ParsedType) -> JavaValue {
    match type_ {
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

impl JavaValue{
    pub fn from_constant_pool_entry(c: &ConstantInfo) -> Self {
        match &c.kind {
            ConstantKind::Integer(i) => JavaValue::Int(unsafe { transmute(i.bytes) }),
            ConstantKind::Float(f) => JavaValue::Float(unsafe { transmute(f.bytes) }),
            ConstantKind::Long(l) => JavaValue::Long(unsafe {
                let high = (l.high_bytes as u64) << 32;
                let low = l.low_bytes as u64;
                transmute(high | low)
            }),
            ConstantKind::Double(d) => JavaValue::Double(unsafe {
                let high = (d.high_bytes as u64) << 32;
                let low = d.low_bytes as u64;
                transmute(high | low)
            }),
            ConstantKind::String(_) => unimplemented!(),
            _ => panic!()
        }
    }

}
