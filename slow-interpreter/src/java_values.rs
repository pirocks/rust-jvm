use crate::runtime_class::RuntimeClass;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Error};
use rust_jvm_common::classnames::{class_name, ClassName};
use rust_jvm_common::classfile::ACC_ABSTRACT;

use std::ops::Deref;
use classfile_view::view::ptype_view::PTypeView;

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

pub trait CycleDetectingDebug {
    fn cycle_fmt(&self, prev: &Vec<&Arc<Object>>, f: &mut Formatter<'_>) -> Result<(), Error>;
}

impl CycleDetectingDebug for JavaValue {
    fn cycle_fmt(&self, prev: &Vec<&Arc<Object>>, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            JavaValue::Long(l) => { write!(f, "{}", l) }
            JavaValue::Int(l) => { write!(f, "{}", l) }
            JavaValue::Short(l) => { write!(f, "{}", l) }
            JavaValue::Byte(l) => { write!(f, "{}", l) }
            JavaValue::Boolean(l) => { write!(f, "{}", l) }
            JavaValue::Char(l) => { write!(f, "{}", l) }
            JavaValue::Float(l) => { write!(f, "{}", l) }
            JavaValue::Double(l) => { write!(f, "{}", l) }
            JavaValue::Object(o) => {
                match o {
                    None => {
                        write!(f, "null", )
                    }
                    Some(s) => {
                        if prev.iter().any(|above| Arc::ptr_eq(above, s)) {
                            write!(f, "<cycle>")
                        } else {
                            let mut new = prev.clone();
                            new.push(s);
                            s.cycle_fmt(&new, f)
                        }
                    }
                }
            }
            JavaValue::Top => { write!(f, "top") }
        }
    }
}

impl CycleDetectingDebug for Object {
    fn cycle_fmt(&self, prev: &Vec<&Arc<Object>>, f: &mut Formatter<'_>) -> Result<(), Error> {
        match &self {
            Object::Array(a) => {
                write!(f, "[")?;
                a.elems.borrow().iter().for_each(|x| {
                    x.cycle_fmt(prev, f).unwrap();
                    write!(f, ",").unwrap();
                });
                write!(f, "]")
            }
            Object::Object(o) => {
                o.cycle_fmt(prev, f)
            }
        }
    }
}

impl CycleDetectingDebug for NormalObject {
    fn cycle_fmt(&self, prev: &Vec<&Arc<Object>>, f: &mut Formatter<'_>) -> Result<(), Error> {
        let o = self;
        if o.class_pointer.class_view.name() == ClassName::class() {
            write!(f, "(Class Object:{:?})", o.class_object_ptype)?;
        } else if o.class_pointer.class_view.name() == ClassName::string() {
            write!(f, "(String Object: {:?})", o.fields.borrow().get("value").unwrap().unwrap_array().unwrap_char_array())?;
        } else {
            write!(f, "{:?}", class_name(&o.class_pointer.classfile).get_referred_name())?;
            write!(f, "-")?;
//        write!(f, "{:?}", self.class_pointer.static_vars)?;
            write!(f, "-")?;
            o.fields.borrow().iter().for_each(|(n, v)| {
                write!(f, "({},", n).unwrap();
                v.cycle_fmt(prev, f).unwrap();
                write!(f, ")\n").unwrap();
            });
            write!(f, "-")?;
            write!(f, "{:?}", o.bootstrap_loader)?;
        }
        Result::Ok(())
    }
}

impl Debug for JavaValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.cycle_fmt(&vec![], f)
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
            JavaValue::Char(c) => {
                *c as i32
            }
            JavaValue::Short(i) => {
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

    pub fn unwrap_object_nonnull(&self) -> Arc<Object> {
        self.try_unwrap_object().unwrap().unwrap()
    }

    pub fn unwrap_array(&self) -> &ArrayObject {
        match self {
            JavaValue::Object(o) => {
                o.as_ref().unwrap().unwrap_array()
            }
            _ => panic!()
        }
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

    pub fn deep_clone(&self) -> Self {
        match &self {
            JavaValue::Long(_) => unimplemented!(),
            JavaValue::Int(_) => unimplemented!(),
            JavaValue::Short(_) => unimplemented!(),
            JavaValue::Byte(_) => unimplemented!(),
            JavaValue::Boolean(_) => unimplemented!(),
            JavaValue::Char(_) => unimplemented!(),
            JavaValue::Float(_) => unimplemented!(),
            JavaValue::Double(_) => unimplemented!(),
            JavaValue::Object(o) => {
                JavaValue::Object(match o {
                    None => None,
                    Some(o) => {
                        Arc::new(o.deref().deep_clone()).into()
                    }
                })
            }
            JavaValue::Top => unimplemented!(),
        }
    }
    pub fn empty_byte_array() -> JavaValue {
        JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject { elems: RefCell::new(vec![]), elem_type: PTypeView::ByteType }))))
    }
    pub fn new_object(runtime_class: Arc<RuntimeClass>) -> Option<Arc<Object>> {
        assert_eq!(runtime_class.classfile.access_flags & ACC_ABSTRACT, 0);
        Arc::new(Object::Object(NormalObject {
            gc_reachable: true,
            class_pointer: runtime_class,
            fields: RefCell::new(HashMap::new()),
            bootstrap_loader: false,
            // object_class_object_pointer: RefCell::new(None),
            // array_class_object_pointer: RefCell::new(None),
            class_object_ptype: RefCell::new(None),
        })).into()
    }

    pub fn new_vec(len: usize, val: JavaValue, elem_type: PTypeView) -> Option<Arc<Object>> {
        let mut buf: Vec<JavaValue> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Some(Arc::new(Object::Array(ArrayObject { elems: buf.into(), elem_type })))
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject {
        //todo these are longer than ideal
        match self {
            JavaValue::Object(ref_) => {
                match ref_.as_ref().unwrap().deref() {
                    Object::Array(_) => panic!(),
                    Object::Object(o) => { o }
                }
            }
            _ => panic!()
        }
    }

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

#[derive(Debug)]
pub enum Object {
    Array(ArrayObject),
    Object(NormalObject),
}

impl Object {
    pub fn lookup_field(&self, s: &str) -> JavaValue {
        self.unwrap_normal_object().fields.borrow().get(s).unwrap().clone()
    }

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

    pub fn deep_clone(&self) -> Self {
        match &self {
            Object::Array(a) => {
                let sub_array = a.elems.borrow().iter().map(|x| x.deep_clone()).collect();
                Object::Array(ArrayObject { elems: RefCell::new(sub_array), elem_type: a.elem_type.clone() })
            }
            Object::Object(o) => {
                let new_fields = RefCell::new(o.fields.borrow().iter().map(|(s, jv)| { (s.clone(), jv.deep_clone()) }).collect());
                Object::Object(NormalObject {
                    gc_reachable: o.gc_reachable,
                    fields: new_fields,
                    class_pointer: o.class_pointer.clone(),
                    bootstrap_loader: o.bootstrap_loader,
                    // object_class_object_pointer: o.object_class_object_pointer.clone(),
                    // array_class_object_pointer: o.array_class_object_pointer.clone(),
                    class_object_ptype: RefCell::new(None),
                })
            }
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Object::Array(_) => true,
            Object::Object(_) => false,
        }
    }

    pub fn object_array(object_array: Vec<JavaValue>, class_type: PTypeView) -> Object {
        Object::Array(ArrayObject {
            elems: RefCell::new(object_array),
            elem_type: class_type,
        })
    }
}

#[derive(Debug)]
pub struct ArrayObject {
    pub elems: RefCell<Vec<JavaValue>>,
    pub elem_type: PTypeView,
}

//#[derive(Debug)]
pub struct NormalObject {
    pub gc_reachable: bool,
    //I guess this never changes so unneeded?
    pub fields: RefCell<HashMap<String, JavaValue>>,
    pub class_pointer: Arc<RuntimeClass>,
    //todo this should just point to the actual class object.
    pub bootstrap_loader: bool,
    pub class_object_ptype: RefCell<Option<PTypeView>>,
    //points to the object represented by this class object of relevant
    //might be simpler to ccombine these into a ptype
    // pub object_class_object_pointer: RefCell<Option<Arc<RuntimeClass>>>,
    //todo why are these refcell?
    // pub array_class_object_pointer: RefCell<Option<PType>>,// is type of array sub type,not including the array.
}

impl NormalObject {
    pub fn class_object_to_ptype(&self) -> PTypeView {
        self.class_object_ptype.borrow().clone().unwrap()
    }
}

impl Debug for NormalObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.cycle_fmt(&vec![], f)
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


impl ArrayObject {
    pub fn unwrap_object_array(&self) -> Vec<Option<Arc<Object>>> {
        self.elems.borrow().iter().map(|x| { x.unwrap_object() }).collect()
    }
    pub fn unwrap_object_array_nonnull(&self) -> Vec<Arc<Object>> {
        self.elems.borrow().iter().map(|x| { x.unwrap_object_nonnull() }).collect()
    }
    pub fn unwrap_byte_array(&self) -> Vec<i8> {//todo in future use jbyte for this kinda thing, and in all places where this is an issue
        assert_eq!(self.elem_type, PTypeView::ByteType);
        self.elems.borrow().iter().map(|x| { x.unwrap_int() as i8 }).collect()
    }
    pub fn unwrap_char_array(&self) -> String {
        assert_eq!(self.elem_type, PTypeView::CharType);
        let mut res = String::new();
        self.elems.borrow().iter().for_each(|x| { res.push(x.unwrap_int()  as u8 as char) });
        res
    }
}

impl std::convert::From<Option<Arc<Object>>> for JavaValue {
    fn from(f: Option<Arc<Object>>) -> Self {
        JavaValue::Object(f)
    }
}

// impl Clone for NormalObject {
//     fn clone(&self) -> Self {
//         NormalObject {
//             gc_reachable: self.gc_reachable,
//             fields: self.fields.clone(),
//             class_pointer: self.class_pointer.clone(),
//             bootstrap_loader: self.bootstrap_loader,
//             class_object_ptype: self.class_object_ptype.clone()
//         }
//     }
// }