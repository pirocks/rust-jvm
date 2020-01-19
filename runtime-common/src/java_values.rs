use crate::runtime_class::RuntimeClass;
use std::sync::Arc;
use std::iter::Map;
use rust_jvm_common::unified_types::ParsedType;
use rust_jvm_common::classfile::ConstantInfo;
use rust_jvm_common::classfile::ConstantKind;
use std::mem::transmute;
use std::mem;
use std::alloc::{alloc, dealloc, Layout};
use std::cell::RefCell;
use std::collections::HashMap;

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

impl PartialEq for JavaValue{
    fn eq(&self, other: &Self) -> bool {
        match self{
            JavaValue::Long(x) => {
                match other {
                    JavaValue::Long(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Int(x) => {
                match other {
                    JavaValue::Int(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Short(x) => {
                match other {
                    JavaValue::Short(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Byte(x) => {
                match other {
                    JavaValue::Byte(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Boolean(x) => {
                match other {
                    JavaValue::Boolean(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Char(x) => {
                match other {
                    JavaValue::Char(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Float(x) => {
                match other {
                    JavaValue::Float(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Double(x) => {
                match other {
                    JavaValue::Double(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Array(x) => {
                match other {
                    JavaValue::Array(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Object(x) => {
                match other {
                    JavaValue::Object(x1) => x == x1,
                    _ => false
                }
            },
            JavaValue::Top => {
                match other {
                    JavaValue::Top => true,
                    _ => false
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct ObjectPointer {
    pub object: Arc<Object>
}

impl ObjectPointer{
    pub fn new(runtime_class: Arc<RuntimeClass>) -> ObjectPointer{
        ObjectPointer {
            object: Arc::new(Object {
                gc_reachable: true,
                class_pointer: runtime_class,
                fields: RefCell::new(HashMap::new())
            })
        }
    }
}

impl PartialEq for ObjectPointer{
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.object.class_pointer , &other.object.class_pointer) && self.object.fields == self.object.fields
    }
}

impl Clone for ObjectPointer{
    fn clone(&self) -> Self {
        ObjectPointer { object: self.object.clone() }
    }
}

#[derive(Debug)]
pub struct VecPointer {
    pub object: *const Vec<JavaValue>
}

impl VecPointer{
    pub fn new(len : usize) -> VecPointer{
        let mut buf:Vec<JavaValue> = Vec::with_capacity(len);
        mem::forget(&buf);
        unsafe {VecPointer {object: transmute(&buf)}}
    }
}

impl PartialEq for VecPointer{
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl Clone for VecPointer{
    fn clone(&self) -> Self {
        VecPointer { object: self.object }
    }
}

#[derive(Debug)]
pub struct Object {
    gc_reachable: bool,
    //I guess this never changes so unneeded?
    pub fields: RefCell<HashMap<String, JavaValue>>,
    class_pointer: Arc<RuntimeClass>,
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
