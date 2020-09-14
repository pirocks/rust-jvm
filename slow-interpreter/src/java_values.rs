use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use std::sync::Arc;

use classfile_view::view::HasAccessFlags;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use rust_jvm_common::classnames::ClassName;

use crate::{InterpreterStateGuard, JVMState};
use crate::interpreter_util::check_inited_class;
use crate::runtime_class::RuntimeClass;
use crate::threading::monitors::Monitor;

// #[derive(Copy)]
pub enum JavaValue {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),

    Float(f32),
    Double(f64),
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
        if o.class_pointer.view().name() == ClassName::class() {
            write!(f, "(Class Object:{:?})", o.class_object_type.as_ref().unwrap().ptypeview())?;//todo needs a JClass type interface
        } else if o.class_pointer.view().name() == ClassName::string() {
            let fields_borrow = o.fields.borrow();
            let value_field = fields_borrow.get("value").unwrap();
            match &value_field.unwrap_object() {
                None => {
                    write!(f, "(String Object: {:?})", "weird af string obj.")?;
                }
                Some(_) => {
                    write!(f, "(String Object: {:?})", value_field.unwrap_array().unwrap_char_array())?;
                }
            }
        } else {
            write!(f, "{:?}", &o.class_pointer.view().name())?;
            write!(f, "-")?;
//        write!(f, "{:?}", self.class_pointer.static_vars)?;
            write!(f, "-")?;
            o.fields.borrow().iter().for_each(|(n, v)| {
                write!(f, "({},", n).unwrap();
                v.cycle_fmt(prev, f).unwrap();
                write!(f, ")\n").unwrap();
            });
            write!(f, "-")?;
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
        self.try_unwrap_int().unwrap()
    }

    pub fn try_unwrap_int(&self) -> Option<i32> {
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
                return None
            }
        }.into()
    }

    pub fn unwrap_float(&self) -> f32 {
        self.try_unwrap_float().unwrap()
    }
    pub fn try_unwrap_float(&self) -> Option<f32> {
        match self {
            JavaValue::Float(f) => {
                (*f).into()
            }
            _ => None
        }
    }
    pub fn unwrap_double(&self) -> f64 {
        self.try_unwrap_double().unwrap()
    }

    pub fn unwrap_long(&self) -> i64 {
        self.try_unwrap_long().unwrap()
    }


    pub fn try_unwrap_double(&self) -> Option<f64> {
        match self {
            JavaValue::Double(f) => {
                (*f).into()
            }
            _ => None
        }
    }

    pub fn try_unwrap_long(&self) -> Option<i64> {
        match self {
            JavaValue::Long(l) => {
                (*l).into()
            }
            _ => None
        }
    }

    pub fn unwrap_byte(&self) -> i8 {
        self.unwrap_int() as i8
    }

    pub fn unwrap_boolean(&self) -> u8 {
        match self {
            JavaValue::Boolean(b) => {
                *b
            }
            JavaValue::Int(i) => {
                *i as u8
            }
            JavaValue::Byte(b) => {
                *b as u8
            }
            _ => {
                dbg!(self);
                panic!()
            }
        }
    }

    pub fn unwrap_short(&self) -> i16 {
        match self {
            JavaValue::Short(s) => {
                *s
            }
            _ => panic!()
        }
    }


    pub fn unwrap_object(&self) -> Option<Arc<Object>> {
        self.try_unwrap_object().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> Arc<Object> {
        self.try_unwrap_object()
            .unwrap()
            .unwrap()
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
                Some(o.clone())
            }
            other => {
                dbg!(other);
                None
            }
        }
    }

    pub fn deep_clone(&self, jvm: &'static JVMState) -> Self {
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
                        Arc::new(o.deref().deep_clone(jvm)).into()
                    }
                })
            }
            JavaValue::Top => unimplemented!(),
        }
    }
    pub fn empty_byte_array(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> JavaValue {
        JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            vec![],
            PTypeView::ByteType,
            jvm.thread_state.new_monitor("".to_string()),
        )))))
    }
    pub fn new_object(jvm: &'static JVMState, runtime_class: Arc<RuntimeClass>, class_object_type: Option<Arc<RuntimeClass>>) -> Option<Arc<Object>> {
        assert!(!runtime_class.view().is_abstract());
        Arc::new(Object::Object(NormalObject {
            monitor: jvm.thread_state.new_monitor("".to_string()),
            class_pointer: runtime_class,
            fields: RefCell::new(HashMap::new()),
            class_object_type,
        })).into()
    }

    pub fn new_vec(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, len: usize, val: JavaValue, elem_type: PTypeView) -> Option<Arc<Object>> {
        let mut buf: Vec<JavaValue> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Some(Arc::new(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            buf,
            elem_type,
            jvm.thread_state.new_monitor("array object monitor".to_string()),
        ))))
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject {
        //todo these are longer than ideal
        self.try_unwrap_normal_object().unwrap()
    }


    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject> {
        //todo these are longer than ideal
        match self {
            JavaValue::Object(ref_) => match match ref_.as_ref() {
                None => return None,
                Some(obj) => obj.deref(),
            } {
                Object::Array(_) => None,
                Object::Object(o) => o.into(),
            },
            _ => None
        }
    }

    pub fn unwrap_char(&self) -> u16 {
        self.unwrap_int() as u16
        // match self {
        //     JavaValue::Char(c) => {
        //         *c
        //     }
        //     _ => {
        //         dbg!(self);
        //         panic!()
        //     }
        // }
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

unsafe impl Send for Object {}

unsafe impl Sync for Object {}

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

    pub fn deep_clone(&self, jvm: &'static JVMState) -> Self {
        match &self {
            Object::Array(a) => {
                let sub_array = a.elems.borrow().iter().map(|x| x.deep_clone(jvm)).collect();
                Object::Array(ArrayObject { elems: RefCell::new(sub_array), elem_type: a.elem_type.clone(), monitor: jvm.thread_state.new_monitor("".to_string()) })
            }
            Object::Object(o) => {
                let new_fields = RefCell::new(o.fields.borrow().iter().map(|(s, jv)| { (s.clone(), jv.deep_clone(jvm)) }).collect());
                Object::Object(NormalObject {
                    monitor: jvm.thread_state.new_monitor("".to_string()),
                    fields: new_fields,
                    class_pointer: o.class_pointer.clone(),
                    class_object_type: o.class_object_type.clone(),
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

    pub fn object_array(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, object_array: Vec<JavaValue>, class_type: PTypeView) -> Object {
        Object::Array(ArrayObject::new_array(jvm, int_state, object_array, class_type, jvm.thread_state.new_monitor("".to_string())))
    }

    pub fn monitor(&self) -> &Monitor {
        match self {
            Object::Array(a) => &a.monitor,
            Object::Object(o) => &o.monitor,
        }
    }

    pub fn monitor_unlock(&self, jvm: &'static JVMState) {
        self.monitor().unlock(jvm);
    }

    pub fn monitor_lock(&self, jvm: &'static JVMState) {
        self.monitor().lock(jvm);
    }
}

#[derive(Debug)]
pub struct ArrayObject {
    pub elems: RefCell<Vec<JavaValue>>,
    pub elem_type: PTypeView,
    pub monitor: Arc<Monitor>,
}

impl ArrayObject {
    pub fn new_array(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, elems: Vec<JavaValue>, type_: PTypeView, monitor: Arc<Monitor>) -> Self {
        check_inited_class(jvm, int_state, &PTypeView::Ref(ReferenceTypeView::Array(box type_.clone())), jvm.bootstrap_loader.clone());
        Self {
            elems: RefCell::new(elems),
            elem_type: type_,
            monitor,
        }
    }
}

pub struct NormalObject {
    pub monitor: Arc<Monitor>,
    //I guess this never changes so unneeded?
    pub fields: RefCell<HashMap<String, JavaValue>>,
    //todo this refcell should be for the elememts.
    pub class_pointer: Arc<RuntimeClass>,
    //todo this should just point to the actual class object.
    pub class_object_type: Option<Arc<RuntimeClass>>, //points to the object represented by this class object of relevant
}

impl Debug for NormalObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.cycle_fmt(&vec![], f)
    }
}

pub fn default_value(type_: PTypeView) -> JavaValue {
    match type_ {
        PTypeView::ByteType => JavaValue::Byte(0),
        PTypeView::CharType => JavaValue::Char('\u{000000}' as u16),
        PTypeView::DoubleType => JavaValue::Double(0.0),
        PTypeView::FloatType => JavaValue::Float(0.0),
        PTypeView::IntType => JavaValue::Int(0),
        PTypeView::LongType => JavaValue::Long(0),
        PTypeView::Ref(_) => JavaValue::Object(None),
        PTypeView::ShortType => JavaValue::Short(0),
        PTypeView::BooleanType => JavaValue::Boolean(0),
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
        self.elems.borrow().iter().for_each(|x| { res.push(x.unwrap_int() as u8 as char) });
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