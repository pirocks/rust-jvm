use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use std::ptr::{null, null_mut};
use std::sync::Arc;

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jbyte, jobject};
use rust_jvm_common::classnames::ClassName;

use crate::class_loading::check_resolved_class;
use crate::interpreter_state::InterpreterStateGuard;
use crate::jvm_state::JVMState;
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
        write!(f, "\n")?;
        for _ in 0..prev.len() {
            write!(f, " ")?;
        }
        match &self {
            Object::Array(a) => {
                write!(f, "[")?;
                unsafe {
                    a.elems.get().as_ref().unwrap()
                }.iter().for_each(|x| {
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
        if o.class_pointer.view().name() == ClassName::class().into() {
            write!(f, "need a jvm pointer here to give more info on class object")?;
        } else if o.class_pointer.view().name() == ClassName::string().into() {
            let fields_borrow = o.fields_mut();
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
            o.fields_mut().iter().for_each(|(n, v)| {
                write!(f, "({},", n).unwrap();
                v.cycle_fmt(prev, f).unwrap();
                write!(f, ")").unwrap();
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
    pub fn null() -> Self {
        Self::Object(None)
    }

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
                return None;
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
        let res = self.unwrap_int();
        assert!(res <= jbyte::MAX as i32);
        assert!(res >= jbyte::MIN as i32);
        res as i8
    }

    pub fn unwrap_boolean(&self) -> u8 {
        let res = self.unwrap_int();
        assert!(res <= u8::MAX as i32);
        assert!(res >= u8::MIN as i32);
        res as u8
    }

    pub fn unwrap_short(&self) -> i16 {
        let res = self.unwrap_int();
        assert!(res <= i16::MAX as i32);
        assert!(res >= i16::MIN as i32);
        res as i16
    }


    pub fn unwrap_object(&self) -> Option<Arc<Object>> {
        self.try_unwrap_object().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> Arc<Object> {
        match match self.try_unwrap_object() {
            Some(x) => x,
            None => unimplemented!(),
        } {
            Some(x) => x,
            None => unimplemented!(),
        }
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
            _ => {
                // dbg!(other);
                None
            }
        }
    }

    pub fn deep_clone(&self, jvm: &JVMState) -> Self {
        match &self {
            JavaValue::Object(o) => {
                JavaValue::Object(match o {
                    None => None,
                    Some(o) => {
                        Arc::new(o.deref().deep_clone(jvm)).into()
                    }
                })
            }
            JavaValue::Top => panic!(),
            jv => (*jv).clone()
        }
    }
    pub fn empty_byte_array(jvm: &JVMState, int_state: &mut InterpreterStateGuard) -> JavaValue {
        JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            vec![],
            PTypeView::ByteType,
            jvm.thread_state.new_monitor("".to_string()),
        )))))
    }
    pub fn new_object(jvm: &JVMState, runtime_class: Arc<RuntimeClass>) -> Option<Arc<Object>> {
        assert!(!runtime_class.view().is_abstract());
        Arc::new(Object::Object(NormalObject {
            monitor: jvm.thread_state.new_monitor("".to_string()),
            class_pointer: runtime_class,
            fields: UnsafeCell::new(HashMap::new()),
        })).into()
    }

    pub fn new_vec(jvm: &JVMState, int_state: &mut InterpreterStateGuard, len: usize, val: JavaValue, elem_type: PTypeView) -> Option<Arc<Object>> {
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

    pub fn new_vec_from_vec(jvm: &JVMState, vals: Vec<JavaValue>, elem_type: PTypeView) -> JavaValue {
        JavaValue::Object(Some(Arc::new(Object::Array(ArrayObject {
            elems: UnsafeCell::new(vals),
            elem_type,
            monitor: jvm.thread_state.new_monitor("".to_string()),
        }))))
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

    pub fn to_type(&self) -> PTypeView {
        match self {
            JavaValue::Long(_) => PTypeView::LongType,
            JavaValue::Int(_) => PTypeView::IntType,
            JavaValue::Short(_) => PTypeView::ShortType,
            JavaValue::Byte(_) => PTypeView::ByteType,
            JavaValue::Boolean(_) => PTypeView::BooleanType,
            JavaValue::Char(_) => PTypeView::CharType,
            JavaValue::Float(_) => PTypeView::FloatType,
            JavaValue::Double(_) => PTypeView::DoubleType,
            JavaValue::Object(obj) => {
                match obj {
                    None => PTypeView::NullType,
                    Some(not_null) => PTypeView::Ref(match not_null.deref() {
                        Object::Array(array) => {
                            ReferenceTypeView::Array(array.elem_type.clone().into())
                        }
                        Object::Object(obj) => {
                            ReferenceTypeView::Class(obj.class_pointer.ptypeview().unwrap_class_type())
                        }
                    })
                }
            }
            JavaValue::Top => PTypeView::TopType
        }
    }

    pub fn is_size_2(&self) -> bool {
        match self {
            JavaValue::Long(_) => true,
            JavaValue::Double(_) => true,
            _ => false,
        }
    }


    pub fn is_size_1(&self) -> bool {
        !self.is_size_2()
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
                            None => x1.is_none(),
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
                matches!(other, JavaValue::Top)
            }
        }
    }
}

#[derive(Debug)]
pub enum Object {
    Array(ArrayObject),
    Object(NormalObject),
}

//todo should really fix this
unsafe impl Send for Object {}

unsafe impl Sync for Object {}

impl Object {
    pub fn lookup_field(&self, s: &str) -> JavaValue {
        self.unwrap_normal_object().fields_mut().get(s).unwrap().clone()
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

    pub fn deep_clone(&self, jvm: &JVMState) -> Self {
        match &self {
            Object::Array(a) => {
                let sub_array = unsafe { a.elems.get().as_ref() }.unwrap().iter().map(|x| x.deep_clone(jvm)).collect();
                Object::Array(ArrayObject { elems: UnsafeCell::new(sub_array), elem_type: a.elem_type.clone(), monitor: jvm.thread_state.new_monitor("".to_string()) })
            }
            Object::Object(o) => {
                let new_fields = UnsafeCell::new(o.fields_mut().iter().map(|(s, jv)| { (s.clone(), jv.deep_clone(jvm)) }).collect());
                Object::Object(NormalObject {
                    monitor: jvm.thread_state.new_monitor("".to_string()),
                    fields: new_fields,
                    class_pointer: o.class_pointer.clone(),
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

    pub fn object_array(jvm: &JVMState, int_state: &mut InterpreterStateGuard, object_array: Vec<JavaValue>, class_type: PTypeView) -> Object {
        Object::Array(ArrayObject::new_array(jvm, int_state, object_array, class_type, jvm.thread_state.new_monitor("".to_string())))
    }

    pub fn monitor(&self) -> &Monitor {
        match self {
            Object::Array(a) => &a.monitor,
            Object::Object(o) => &o.monitor,
        }
    }

    pub fn monitor_unlock(&self, jvm: &JVMState) {
        self.monitor().unlock(jvm);
    }

    pub fn monitor_lock(&self, jvm: &JVMState) {
        self.monitor().lock(jvm);
    }
}

#[derive(Debug)]
pub struct ArrayObject {
    pub elems: UnsafeCell<Vec<JavaValue>>,
    pub elem_type: PTypeView,
    pub monitor: Arc<Monitor>,
}

impl ArrayObject {
    pub fn mut_array(&self) -> &mut Vec<JavaValue> {
        unsafe { self.elems.get().as_mut().unwrap() }
    }

    pub fn new_array(jvm: &JVMState, int_state: &mut InterpreterStateGuard, elems: Vec<JavaValue>, type_: PTypeView, monitor: Arc<Monitor>) -> Self {
        check_resolved_class(jvm, int_state, PTypeView::Ref(ReferenceTypeView::Array(box type_.clone()))).unwrap();//todo pass the error up
        Self {
            elems: UnsafeCell::new(elems),
            elem_type: type_,
            monitor,
        }
    }
}

pub struct NormalObject {
    pub monitor: Arc<Monitor>,
    pub fields: UnsafeCell<HashMap<String, JavaValue>>,
    //todo this refcell should be by class pointer, to avoid same name clashes.
    pub class_pointer: Arc<RuntimeClass>,
}

impl NormalObject {
    pub fn fields_mut(&self) -> &mut HashMap<String, JavaValue> {
        unsafe { self.fields.get().as_mut().unwrap() }
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
        unsafe { self.elems.get().as_ref() }.unwrap().iter().map(|x| { x.unwrap_object() }).collect()
    }

    pub fn unwrap_mut(&self) -> &mut Vec<JavaValue> {
        unsafe { self.elems.get().as_mut() }.unwrap()
    }
    pub fn unwrap_object_array_nonnull(&self) -> Vec<Arc<Object>> {
        self.mut_array().iter().map(|x| { x.unwrap_object_nonnull() }).collect()
    }
    pub fn unwrap_byte_array(&self) -> Vec<jbyte> {
        assert_eq!(self.elem_type, PTypeView::ByteType);
        self.mut_array().iter().map(|x| { x.unwrap_byte() }).collect()
    }
    pub fn unwrap_char_array(&self) -> String {
        assert_eq!(self.elem_type, PTypeView::CharType);
        let mut res = String::new();
        unsafe { self.elems.get().as_ref() }.unwrap().iter().for_each(|x| { res.push(x.unwrap_int() as u8 as char) });
        res
    }
}

impl std::convert::From<Option<Arc<Object>>> for JavaValue {
    fn from(f: Option<Arc<Object>>) -> Self {
        JavaValue::Object(f)
    }
}


pub trait ExceptionReturn {
    fn invalid_default() -> Self;
}

impl ExceptionReturn for i64 {
    fn invalid_default() -> Self {
        i64::MAX
    }
}

impl ExceptionReturn for i32 {
    fn invalid_default() -> Self {
        i32::MAX
    }
}

impl ExceptionReturn for i16 {
    fn invalid_default() -> Self {
        i16::MAX
    }
}

impl ExceptionReturn for i8 {
    fn invalid_default() -> Self {
        i8::MAX
    }
}

impl ExceptionReturn for u8 {
    fn invalid_default() -> Self {
        u8::MAX
    }
}

impl ExceptionReturn for u16 {
    fn invalid_default() -> Self {
        u16::MAX
    }
}

impl ExceptionReturn for f32 {
    fn invalid_default() -> Self {
        f32::MAX
    }
}

impl ExceptionReturn for f64 {
    fn invalid_default() -> Self {
        f64::MAX
    }
}

impl ExceptionReturn for jobject {
    fn invalid_default() -> Self {
        null_mut()
    }
}

impl ExceptionReturn for *const i8 {
    fn invalid_default() -> Self {
        null()
    }
}

impl ExceptionReturn for JavaValue {
    fn invalid_default() -> Self {
        JavaValue::Top
    }
}

impl ExceptionReturn for () {
    fn invalid_default() -> Self {
        ()
    }
}