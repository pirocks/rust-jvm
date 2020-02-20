use crate::runtime_class::RuntimeClass;
use std::sync::Arc;
use rust_jvm_common::unified_types::PType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Error};
use rust_jvm_common::classnames::class_name;
use rust_jvm_common::classfile::ACC_ABSTRACT;
use rust_jvm_common::view::ptype_view::PTypeView;

//#[derive(Debug)]
pub enum JavaValue {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(bool),
    Char(char),

    Float(f32),
    Double(f64),

    //    Array(Option<(ParsedType,Arc<RefCell<Vec<JavaValue>>>)>),
    Object(Option<Arc<Object>>),

    Top,//should never be interacted with by the bytecode
}

impl Debug for JavaValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            JavaValue::Long(l) => { write!(f, "{}", l) }
            JavaValue::Int(l) => { write!(f, "{}", l) }
            JavaValue::Short(l) => { write!(f, "{}", l) }
            JavaValue::Byte(l) => { write!(f, "{}", l) }
            JavaValue::Boolean(l) => { write!(f, "{}", l) }
            JavaValue::Char(l) => { write!(f, "{}", l) }
            JavaValue::Float(l) => { write!(f, "{}", l) }
            JavaValue::Double(l) => { write!(f, "{}", l) }
            JavaValue::Object(o) => { write!(f, "{:?}", o) }
            JavaValue::Top => { write!(f, "top") }
        }
    }
}

impl JavaValue {
    pub fn unwrap_int(&self) -> i32 {
        match self {
            JavaValue::Int(i) => {
                *i
            }
            JavaValue::Byte(i) => {
                *i as i32
            }
            JavaValue::Boolean(i) => {
                *i as i32
            }
            _ => {
                dbg!(self);
                panic!()
            }
        }
    }

    pub fn unwrap_float(&self) -> f32 {
        match self {
            JavaValue::Float(f) => {
                *f
            }
            _ => panic!()
        }
    }
    pub fn unwrap_double(&self) -> f64 {
        match self {
            JavaValue::Double(f) => {
                *f
            }
            _ => panic!()
        }
    }

    pub fn unwrap_long(&self) -> i64 {
        match self {
            JavaValue::Long(l) => {
                *l
            }
            _ => panic!()
        }
    }


    pub fn unwrap_object(&self) -> Option<Arc<Object>> {
        self.try_unwrap_object().unwrap()
    }


    pub fn try_unwrap_object(&self) -> Option<Option<Arc<Object>>> {
        match self {
            JavaValue::Object(o) => {
                let option = o.as_ref();
                Some(match option {
                    None => None,
                    Some(o) => o.clone().into(),
                })
            }
            other => {
                dbg!(other);
                None
            }
        }
    }
}

impl Clone for JavaValue {
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
//            JavaValue::Array(a) => JavaValue::Array(a.clone()),
            JavaValue::Object(o) => JavaValue::Object(o.clone()),
            JavaValue::Top => JavaValue::Top,
        }
    }
}

impl PartialEq for JavaValue {
    fn eq(&self, other: &Self) -> bool {
        match self {
            JavaValue::Long(x) => {
                match other {
                    JavaValue::Long(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Int(x) => {
                match other {
                    JavaValue::Int(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Short(x) => {
                match other {
                    JavaValue::Short(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Byte(x) => {
                match other {
                    JavaValue::Byte(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Boolean(x) => {
                match other {
                    JavaValue::Boolean(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Char(x) => {
                match other {
                    JavaValue::Char(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Float(x) => {
                match other {
                    JavaValue::Float(x1) => x == x1,
                    _ => false
                }
            }
            JavaValue::Double(x) => {
                match other {
                    JavaValue::Double(x1) => x == x1,
                    _ => false
                }
            }
            /*JavaValue::Array(x) => {
                match other {
                    JavaValue::Array(x1) => x == x1,
                    _ => false
                }
            }*/
            JavaValue::Object(x) => {
                match other {
                    JavaValue::Object(x1) => {
                        match x {
                            None => match x1 {
                                None => true,
                                Some(_) => false
                            },
                            Some(o) => match x1 {
                                None => false,
                                Some(o1) => Arc::ptr_eq(o, o1),
                            },
                        }
                    }
                    _ => false
                }
            }
            JavaValue::Top => {
                match other {
                    JavaValue::Top => true,
                    _ => false
                }
            }
        }
    }
}

//#[derive(Debug)]
//pub struct ObjectPointer {
//    pub object: Arc<Object>
//}

impl JavaValue {
    pub fn new_object(runtime_class: Arc<RuntimeClass>) -> Option<Arc<Object>> {
        assert_eq!(runtime_class.classfile.access_flags & ACC_ABSTRACT, 0);
        Arc::new(Object::Object(NormalObject {
            gc_reachable: true,
            class_pointer: runtime_class,
            fields: RefCell::new(HashMap::new()),
            bootstrap_loader: false,
            object_class_object_pointer: RefCell::new(None),
            array_class_object_pointer: RefCell::new(None),
        })).into()
    }
}

//impl PartialEq for ObjectPointer {
//    fn eq(&self, other: &Self) -> bool {
//        Arc::ptr_eq(&self.object.class_pointer, &other.object.class_pointer) && self.object.fields == self.object.fields
//    }
//}

//impl Clone for ObjectPointer {
//    fn clone(&self) -> Self {
//        ObjectPointer { object: self.object.clone() }
//    }
//}

//#[derive(Debug)]
//pub struct VecPointer {
//    pub object: Arc<RefCell<Vec<JavaValue>>>
//}

impl JavaValue {
    pub fn new_vec(len: usize, val: JavaValue, elem_type: PType) -> Option<Arc<Object>> {
        let mut buf: Vec<JavaValue> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Some(Arc::new(Object::Array(ArrayObject { elems: buf.into(), elem_type })))
    }
}

//impl PartialEq for VecPointer {
//    fn eq(&self, other: &Self) -> bool {
//        self.object == other.object
//    }
//}
//
//impl Clone for VecPointer {
//    fn clone(&self) -> Self {
//        VecPointer { object: self.object.clone() }
//    }
//}

#[derive(Debug)]
pub enum Object {
    Array(ArrayObject),
    Object(NormalObject),
}

impl Object {
    pub fn unwrap_normal_object(&self) -> &NormalObject {
        match self {
            Object::Array(_) => panic!(),
            Object::Object(o) => o,
        }
    }
    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject> {
        match self {
            Object::Array(_) => None,
            Object::Object(o) => Some(o),
        }
    }


    pub fn unwrap_array(&self) -> &ArrayObject {
        match self {
            Object::Array(a) => a,
            Object::Object(_) => panic!(),
        }
    }
}

#[derive(Debug)]
pub struct ArrayObject {
    pub elems: RefCell<Vec<JavaValue>>,
    pub elem_type: PType,
}

//#[derive(Debug)]
pub struct NormalObject {
    pub gc_reachable: bool,
    //I guess this never changes so unneeded?
    pub fields: RefCell<HashMap<String, JavaValue>>,
    pub class_pointer: Arc<RuntimeClass>,
    pub bootstrap_loader: bool,
    //points to the object represented by this class object of relevant
    pub object_class_object_pointer: RefCell<Option<Arc<RuntimeClass>>>,
    pub array_class_object_pointer: RefCell<Option<PType>>,
}

impl Debug for NormalObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", class_name(&self.class_pointer.classfile).get_referred_name())?;
        write!(f, "-")?;
//        write!(f, "{:?}", self.class_pointer.static_vars)?;
        write!(f, "-")?;
        write!(f, "{:?}", self.fields)?;
        write!(f, "-")?;
        write!(f, "{:?}", self.bootstrap_loader)?;
        Result::Ok(())
    }
}

pub fn default_value(type_: PTypeView) -> JavaValue {
    match type_ {
        PTypeView::ByteType => JavaValue::Byte(0),
        PTypeView::CharType => JavaValue::Char('\u{000000}'),
        PTypeView::DoubleType => JavaValue::Double(0.0),
        PTypeView::FloatType => JavaValue::Float(0.0),
        PTypeView::IntType => JavaValue::Int(0),
        PTypeView::LongType => JavaValue::Long(0),
        PTypeView::Ref(_) => JavaValue::Object(None),
        PTypeView::ShortType => JavaValue::Short(0),
        PTypeView::BooleanType => JavaValue::Boolean(false),
        PTypeView::VoidType => panic!(),
        PTypeView::TopType => JavaValue::Top,
        PTypeView::NullType => JavaValue::Object(None),
        PTypeView::Uninitialized(_) => unimplemented!(),
        PTypeView::UninitializedThis => unimplemented!(),
        PTypeView::UninitializedThisOrClass(_) => panic!(),
    }
}

impl JavaValue {
    /*
        pub fn unwrap_array(&self) -> (ParsedType,Arc<RefCell<Vec<JavaValue>>>) {
            match self {
                JavaValue::Array(a) => {
                    a.as_ref().unwrap().clone()
                }
                _ => {
                    dbg!(self);
                    panic!()
                }
            }
        }
    */

    pub fn unwrap_char(&self) -> char {
        match self {
            JavaValue::Char(c) => {
                c.clone()
            }
            _ => {
                dbg!(self);
                panic!()
            }
        }
    }
}
