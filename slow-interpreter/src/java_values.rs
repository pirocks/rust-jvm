use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::{Debug, Error, Formatter};
use std::ops::Deref;
use std::ptr::{NonNull, null, null_mut};
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};

use itertools::{Itertools, repeat_n};

use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use jvmti_jni_bindings::{jbyte, jfieldID, jmethodID, jobject};

use crate::class_loading::check_resolved_class;
use crate::interpreter::WasException;
use crate::interpreter_state::InterpreterStateGuard;
use crate::jvm_state::JVMState;
use crate::runtime_class::{RuntimeClass, RuntimeClassClass};
use crate::threading::monitors::Monitor;

pub struct GC<'gc_life> {
    //doesn't really need to be atomic usize
    reentrant_roots: RwLock<HashMap<NonNull<Object<'gc_life>>, AtomicUsize>>,
}

impl<'gc_life> GC<'gc_life> {
    pub fn register_root_reentrant(&'gc_life self, ptr: NonNull<Object<'gc_life>>) {
        let mut guard = self.reentrant_roots.write().unwrap();
        let count = guard.entry(ptr).or_insert(AtomicUsize::new(0));
        count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn deregister_root_reentrant(&'gc_life self, ptr: NonNull<Object<'gc_life>>) {
        let mut guard = self.reentrant_roots.read().unwrap();
        let count = guard.get(&ptr).unwrap();
        count.fetch_sub(1, Ordering::SeqCst);
        if count.load(Ordering::SeqCst) == 0 {
            drop(guard);
            self.reentrant_roots.write().unwrap().remove(&ptr);
        }
    }

    pub fn allocate_object(&'gc_life self, object: Object<'gc_life>) -> GcManagedObject<'gc_life> {
        let ptr = NonNull::new(Box::into_raw(box object)).unwrap();
        GcManagedObject {
            raw_ptr: ptr,
            gc: self,
        }
    }

    pub fn new() -> Self {
        Self {
            reentrant_roots: RwLock::new(Default::default())
        }
    }
}

pub struct GcManagedObject<'gc_life> {
    raw_ptr: NonNull<Object<'gc_life>>,
    //allocated from a box
    gc: &'gc_life GC<'gc_life>,
}

impl<'gc_life> Deref for GcManagedObject<'gc_life> {
    type Target = Object<'gc_life>;

    fn deref(&self) -> &Self::Target {
        unsafe { self.raw_ptr.as_ref() }
    }
}


impl<'gc_life> Clone for GcManagedObject<'gc_life> {
    fn clone(&self) -> Self {
        //this doesn't leak b/c if we ever try to create a cycle we put into a field and deregister as a root.
        unsafe {
            self.gc.register_root_reentrant(self.raw_ptr);
            Self {
                raw_ptr: self.raw_ptr,
                gc: self.gc,
            }
        }
    }
}

impl Drop for GcManagedObject<'_> {
    fn drop(&mut self) {
        self.gc.deregister_root_reentrant(self.raw_ptr)
    }
}


impl<'gc_life> GcManagedObject<'gc_life> {
    pub fn lookup_field(&self, field_name: impl Into<String>) -> JavaValue<'gc_life> {
        self.deref().lookup_field(field_name.into().as_str())
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc_life> {
        self.deref().unwrap_normal_object()
    }

    pub fn ptr_eq(one: &GcManagedObject<'gc_life>, two: &GcManagedObject<'gc_life>) -> bool {
        one.raw_ptr.as_ptr() == two.raw_ptr.as_ptr()
    }

    pub fn raw_ptr_usize(&self) -> usize {
        self.raw_ptr.as_ptr() as usize
    }
}

// #[derive(Copy)]
pub enum JavaValue<'gc_life> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),

    Float(f32),
    Double(f64),
    Object(Option<GcManagedObject<'gc_life>>),

    Top,//should never be interacted with by the bytecode
}

// pub trait CycleDetectingDebug {
//     fn cycle_fmt<'gc_life>(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error>;
// }

// impl<'gc_life> CycleDetectingDebug for JavaValue<'gc_life> {
//     fn cycle_fmt(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error> {
//         match self {
//             JavaValue::Long(l) => { write!(f, "{}", l) }
//             JavaValue::Int(l) => { write!(f, "{}", l) }
//             JavaValue::Short(l) => { write!(f, "{}", l) }
//             JavaValue::Byte(l) => { write!(f, "{}", l) }
//             JavaValue::Boolean(l) => { write!(f, "{}", l) }
//             JavaValue::Char(l) => { write!(f, "{}", l) }
//             JavaValue::Float(l) => { write!(f, "{}", l) }
//             JavaValue::Double(l) => { write!(f, "{}", l) }
//             JavaValue::Object(o) => {
//                 match o {
//                     None => {
//                         write!(f, "null", )
//                     }
//                     Some(s) => {
//                         if prev.iter().any(|above| Arc::ptr_eq(above, s)) {
//                             write!(f, "<cycle>")
//                         } else {
//                             let mut new = prev.clone();
//                             new.push(s);
//                             s.cycle_fmt(&new, f)
//                         }
//                     }
//                 }
//             }
//             JavaValue::Top => { write!(f, "top") }
//         }
//     }
// }
//
// impl<'gc_life> CycleDetectingDebug for Object<'gc_life> {
//     fn cycle_fmt(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error> {
//         write!(f, "\n")?;
//         for _ in 0..prev.len() {
//             write!(f, " ")?;
//         }
//         match &self {
//             Object::Array(a) => {
//                 write!(f, "[")?;
//                 unsafe {
//                     a.elems.get().as_ref().unwrap()
//                 }.iter().for_each(|x| {
//                     x.cycle_fmt(prev, f).unwrap();
//                     write!(f, ",").unwrap();
//                 });
//                 write!(f, "]")
//             }
//             Object::Object(o) => {
//                 o.cycle_fmt(prev, f)
//             }
//         }
//     }
// }
//
// impl<'gc_life> CycleDetectingDebug for NormalObject<'gc_life> {
//     fn cycle_fmt(&self, prev: &Vec<&GcManagedObject<'gc_life>>, f: &mut Formatter<'_>) -> Result<(), Error> {
// //         let o = self;
// //         if o.class_pointer.view().name() == ClassName::class().into() {
// //             write!(f, "need a jvm pointer here to give more info on class object")?;
// //         } else if o.class_pointer.view().name() == ClassName::string().into() {
// //             let fields_borrow = o.fields_mut();
// //             let value_field = fields_borrow.get("value").unwrap();
// //             match &value_field.unwrap_object() {
// //                 None => {
// //                     write!(f, "(String Object: {:?})", "weird af string obj.")?;
// //                 }
// //                 Some(_) => {
// //                     write!(f, "(String Object: {:?})", value_field.unwrap_array().unwrap_char_array())?;
// //                 }
// //             }
// //         } else {
// //             write!(f, "{:?}", &o.class_pointer.view().name())?;
// //             write!(f, "-")?;
// // //        write!(f, "{:?}", self.class_pointer.static_vars)?;
// //             write!(f, "-")?;
// //             o.fields_mut().iter().for_each(|(n, v)| {
// //                 write!(f, "({},", n).unwrap();
// //                 v.cycle_fmt(prev, f).unwrap();
// //                 write!(f, ")").unwrap();
// //             });
// //             write!(f, "-")?;
// //         }
// //         Result::Ok(())
//         writeln!(f, "object")
//     }
// }

impl<'gc_life> Debug for JavaValue<'gc_life> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        todo!()
    }
}

impl<'gc_life> JavaValue<'gc_life> {
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


    pub fn unwrap_object(&self) -> Option<GcManagedObject<'gc_life>> {
        self.try_unwrap_object().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> GcManagedObject<'gc_life> {
        match match self.try_unwrap_object() {
            Some(x) => x,
            None => unimplemented!(),
        } {
            Some(x) => x,
            None => unimplemented!(),
        }
    }

    pub fn unwrap_array(&self) -> &ArrayObject<'gc_life> {
        match self {
            JavaValue::Object(o) => {
                o.as_ref().unwrap().unwrap_array()
            }
            _ => panic!()
        }
    }


    pub fn try_unwrap_object(&self) -> Option<Option<GcManagedObject<'gc_life>>> {
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

    pub fn deep_clone(&self, jvm: &'_ JVMState<'gc_life>) -> Self {
        match &self {
            JavaValue::Object(o) => {
                JavaValue::Object(match o {
                    None => None,
                    Some(o) => {
                        jvm.allocate_object(o.deref().deep_clone(jvm)).into()
                    }
                })
            }
            JavaValue::Top => panic!(),
            jv => (*jv).clone()
        }
    }
    pub fn empty_byte_array(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>) -> Result<JavaValue<'gc_life>, WasException> {
        Ok(JavaValue::Object(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            vec![],
            PTypeView::ByteType,
            jvm.thread_state.new_monitor("".to_string()),
        )?)))))
    }

    fn new_object_impl(runtime_class: &Arc<RuntimeClass<'gc_life>>) -> ObjectFieldsAndClass<'gc_life> {
        // let fields = runtime_class.view().fields().flat_map(|field| {
        //     if field.is_static() {
        //         return None;
        //     }
        //     return Some((field.field_name(), UnsafeCell::new(JavaValue::Top)));
        // }).collect::<HashMap<_, _>>();
        let class_class = runtime_class.unwrap_class_class();
        let fields = repeat_n(JavaValue::Top, runtime_class.unwrap_class_class().num_vars()).map(|jv| UnsafeCell::new(jv)).collect_vec();
        ObjectFieldsAndClass {
            fields,
            class_pointer: runtime_class.clone(),
        }
    }

    pub fn new_object(jvm: &'_ JVMState<'gc_life>, runtime_class: Arc<RuntimeClass<'gc_life>>) -> Option<GcManagedObject<'gc_life>> {
        assert!(!runtime_class.view().is_abstract());

        jvm.allocate_object(Object::Object(NormalObject {
            monitor: jvm.thread_state.new_monitor("".to_string()),
            objinfo: Self::new_object_impl(&runtime_class),
        })).into()
    }

    pub fn new_vec(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, len: usize, val: JavaValue<'gc_life>, elem_type: PTypeView) -> Result<Option<GcManagedObject<'gc_life>>, WasException> {
        let mut buf: Vec<JavaValue<'gc_life>> = Vec::with_capacity(len);
        for _ in 0..len {
            buf.push(val.clone());
        }
        Ok(Some(jvm.allocate_object(Object::Array(ArrayObject::new_array(
            jvm,
            int_state,
            buf,
            elem_type,
            jvm.thread_state.new_monitor("array object monitor".to_string()),
        )?))))
    }

    pub fn new_vec_from_vec(jvm: &'_ JVMState<'gc_life>, vals: Vec<JavaValue<'gc_life>>, elem_type: PTypeView) -> JavaValue<'gc_life> {
        JavaValue::Object(todo!()/*Some(Arc::new(Object::Array(ArrayObject {
            elems: UnsafeCell::new(vals),
            elem_type,
            monitor: jvm.thread_state.new_monitor("".to_string()),
        })))*/)
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc_life> {
        //todo these are longer than ideal
        self.try_unwrap_normal_object().unwrap()
    }


    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject<'gc_life>> {
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
                todo!()
                /*                match obj {
                    None => PTypeView::NullType,
                    Some(not_null) => PTypeView::Ref(match not_null.deref() {
                        Object::Array(array) => {
                            ReferenceTypeView::Array(array.elem_type.clone().into())
                        }
                        Object::Object(obj) => {
                            ReferenceTypeView::Class(obj.objinfo.class_pointer.ptypeview().unwrap_class_type())
                        }
                    })
                }
*/
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

impl<'gc_life> Clone for JavaValue<'gc_life> {
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
            JavaValue::Object(o) => JavaValue::Object(o.clone()),
            JavaValue::Top => JavaValue::Top,
        }
    }
}

impl<'gc_life> PartialEq for JavaValue<'gc_life> {
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
                        todo!()
                        /*                        match x {
                            None => x1.is_none(),
                            Some(o) => match x1 {
                                None => false,
                                Some(o1) => Arc::ptr_eq(o, o1),
                            },
                        }
*/
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
pub enum Object<'gc_life> {
    Array(ArrayObject<'gc_life>),
    Object(NormalObject<'gc_life>),
}

//todo should really fix this
unsafe impl<'gc_life> Send for Object<'gc_life> {}

unsafe impl<'gc_life> Sync for Object<'gc_life> {}

impl<'gc_life> Object<'gc_life> {
    pub fn lookup_field(&self, s: &str) -> JavaValue<'gc_life> {
        let class_pointer = self.unwrap_normal_object().objinfo.class_pointer.clone();
        let field_number = class_pointer.unwrap_class_class().field_numbers[s];
        unsafe { self.unwrap_normal_object().objinfo.fields[field_number].get().as_ref() }.unwrap().clone()
    }

    pub fn unwrap_normal_object(&self) -> &NormalObject<'gc_life> {
        match self {
            Object::Array(_) => panic!(),
            Object::Object(o) => o,
        }
    }
    pub fn try_unwrap_normal_object(&self) -> Option<&NormalObject<'gc_life>> {
        match self {
            Object::Array(_) => None,
            Object::Object(o) => Some(o),
        }
    }


    pub fn unwrap_array(&self) -> &ArrayObject<'gc_life> {
        match self {
            Object::Array(a) => a,
            Object::Object(_) => panic!(),
        }
    }

    pub fn deep_clone(&self, jvm: &'_ JVMState<'gc_life>) -> Self {
        match &self {
            Object::Array(a) => {
                let sub_array = unsafe { a.elems.get().as_ref() }.unwrap().iter().map(|x| x.deep_clone(jvm)).collect();
                Object::Array(ArrayObject { elems: UnsafeCell::new(sub_array), elem_type: a.elem_type.clone(), monitor: jvm.thread_state.new_monitor("".to_string()) })
            }
            Object::Object(o) => {
                todo!()
                // let new_fields = UnsafeCell::new(o.fields_mut().iter().map(|(s, jv)| { (s.clone(), jv.deep_clone(jvm)) }).collect());
                // Object::Object(NormalObject {
                //     monitor: jvm.thread_state.new_monitor("".to_string()),
                //     fields: new_fields,
                //     class_pointer: o.class_pointer.clone(),
                // })
            }
        }
    }

    pub fn is_array(&self) -> bool {
        match self {
            Object::Array(_) => true,
            Object::Object(_) => false,
        }
    }

    pub fn object_array(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, object_array: Vec<JavaValue<'gc_life>>, class_type: PTypeView) -> Result<Object<'gc_life>, WasException> {
        Ok(Object::Array(ArrayObject::new_array(jvm, int_state, object_array, class_type, jvm.thread_state.new_monitor("".to_string()))?))
    }

    pub fn monitor(&self) -> &Monitor {
        match self {
            Object::Array(a) => &a.monitor,
            Object::Object(o) => &o.monitor,
        }
    }

    pub fn monitor_unlock(&self, jvm: &'_ JVMState<'gc_life>) {
        self.monitor().unlock(jvm);
    }

    pub fn monitor_lock(&self, jvm: &'_ JVMState<'gc_life>) {
        self.monitor().lock(jvm);
    }
}

#[derive(Debug)]
pub struct ArrayObject<'gc_life> {
    pub elems: UnsafeCell<Vec<JavaValue<'gc_life>>>,
    pub elem_type: PTypeView,
    pub monitor: Arc<Monitor>,
}

impl<'gc_life> ArrayObject<'gc_life> {
    pub fn mut_array(&self) -> &mut Vec<JavaValue<'gc_life>> {
        unsafe { self.elems.get().as_mut().unwrap() }
    }

    pub fn new_array(jvm: &'_ JVMState<'gc_life>, int_state: &'_ mut InterpreterStateGuard<'gc_life, '_>, elems: Vec<JavaValue<'gc_life>>, type_: PTypeView, monitor: Arc<Monitor>) -> Result<Self, WasException> {
        check_resolved_class(jvm, int_state, PTypeView::Ref(ReferenceTypeView::Array(box type_.clone())))?;
        Ok(Self {
            elems: UnsafeCell::new(elems),
            elem_type: type_,
            monitor,
        })
    }
}

pub struct ObjectFieldsAndClass<'gc_life> {
    //ordered by alphabetical and super first
    pub fields: Vec<UnsafeCell<JavaValue<'gc_life>>>,
    pub class_pointer: Arc<RuntimeClass<'gc_life>>,
}

pub struct NormalObject<'gc_life> {
    pub monitor: Arc<Monitor>,
    pub objinfo: ObjectFieldsAndClass<'gc_life>,
}

impl<'gc_life> NormalObject<'gc_life> {
    pub fn set_var_top_level(&self, name: impl Into<String>, jv: JavaValue<'gc_life>) {
        let name = name.into();
        let field_index = self.objinfo.class_pointer.unwrap_class_class().field_numbers.get(&name).unwrap();
        *unsafe {
            self.objinfo.fields[*field_index].get().as_mut()
        }.unwrap() = jv;
    }

    pub fn set_var(&self, class_pointer: Arc<RuntimeClass<'gc_life>>, name: impl Into<String>, jv: JavaValue<'gc_life>, expected_type: PTypeView) {
        let name = name.into();
        // self.expected_type_check(class_pointer.clone(), expected_type.clone(), name.clone(), &jv);
        unsafe { self.set_var_impl(&self.objinfo.class_pointer.unwrap_class_class(), class_pointer, name, jv, true) }
    }

    unsafe fn set_var_impl(&self, current_class_pointer: &RuntimeClassClass, class_pointer: Arc<RuntimeClass<'gc_life>>, name: impl Into<String>, jv: JavaValue<'gc_life>, mut do_class_check: bool) {
        let name = name.into();
        if current_class_pointer.class_view.name() == class_pointer.view().name() || !do_class_check {
            let field_index = match current_class_pointer.field_numbers.get(&name) {
                None => {
                    do_class_check = false;
                }
                Some(field_index) => {
                    self.objinfo.fields.get(*field_index).map(|set| *set.get().as_mut().unwrap() = jv.clone());
                    return;
                }
            };
        }
        if let Some(parent_class) = current_class_pointer.parent.as_ref() {
            self.set_var_impl(parent_class.unwrap_class_class(), class_pointer, name, jv, do_class_check);
        } else {
            panic!()
        }
    }


    pub fn get_var_top_level(&self, name: impl Into<String>) -> JavaValue<'gc_life> {
        let name = name.into();
        let field_index = self.objinfo.class_pointer.unwrap_class_class().field_numbers.get(&name).unwrap();
        unsafe {
            self.objinfo.fields[*field_index].get().as_ref()
        }.unwrap().clone()
    }


    // pub fn type_check(&self, class_pointer: Arc<RuntimeClass>) -> bool {
    //     Self::type_check_impl(&self.objinfo, class_pointer)
    // }

    /*fn type_check_impl(objinfo: &ObjectFieldsAndClass, class_pointer: Arc<RuntimeClass>) -> bool {
        if objinfo.class_pointer.view().name() == class_pointer.view().name() {
            return true;
        }
        return Self::type_check_impl(match objinfo.parent.as_ref() {
            Some(x) => x,
            None => return false,
        }, class_pointer);
    }*/

    /*fn find_matching_cname(objinfo: &ObjectFieldsAndClass, c_name: ReferenceTypeView) -> bool {
        if objinfo.class_pointer.view().name() == ReferenceTypeView::Class(ClassName::class()) && c_name == ReferenceTypeView::Class(ClassName::object()) {
            return true;
        }
        if objinfo.class_pointer.view().name() == c_name {
            return true;
        }
        let super_ = objinfo.parent.as_ref().map(|parent| Self::find_matching_cname(parent.deref(), c_name.clone())).unwrap_or(false);
        let interfaces = objinfo.class_pointer.unwrap_class_class().interfaces.iter().any(|interface| interface.view().name() == c_name.clone());
        return interfaces || super_;
    }*/


    pub fn get_var(&self, class_pointer: Arc<RuntimeClass<'gc_life>>, name: impl Into<String>, expected_type: PTypeView) -> JavaValue<'gc_life> {
        let name = name.into();
        // if !self.type_check(class_pointer.clone()) {
        //     dbg!(name);
        //     dbg!(class_pointer.view().name());
        //     dbg!(self.objinfo.class_pointer.view().name());
        //     panic!()
        // }
        let res = unsafe { Self::get_var_impl(self, self.objinfo.class_pointer.unwrap_class_class(), class_pointer.clone(), &name, true) };
        // self.expected_type_check(class_pointer, expected_type, name, &res);
        res
    }

    /*fn expected_type_check(&self, class_pointer: Arc<RuntimeClass>, expected_type: PTypeView, name: String, res: &JavaValue) {
        match expected_type {
            PTypeView::ByteType => {}
            PTypeView::CharType => {}
            PTypeView::DoubleType => {}
            PTypeView::FloatType => {}
            PTypeView::IntType => {}
            PTypeView::LongType => {}
            PTypeView::Ref(ref_) => match ref_ {
                ReferenceTypeView::Class(c_name) => {
                    if !Self::find_matching_cname(&match match res.unwrap_object() {
                        Some(x) => x,
                        None => return,
                    }.try_unwrap_normal_object() {
                        Some(x) => x,
                        None => return,
                    }.objinfo, ReferenceTypeView::Class(c_name.clone())) {
                        dbg!(name);
                        dbg!(class_pointer.view().name());
                        dbg!(self.objinfo.class_pointer.view().name());
                        dbg!(self.objinfo.fields.keys().collect_vec());
                        dbg!(res.unwrap_normal_object().objinfo.class_pointer.view().name());
                        dbg!(c_name);
                        panic!()
                    }
                }
                ReferenceTypeView::Array(_) => {}
            }
            PTypeView::ShortType => {}
            PTypeView::BooleanType => {}
            PTypeView::VoidType => {}
            PTypeView::TopType => {}
            PTypeView::NullType => {}
            PTypeView::Uninitialized(_) => {}
            PTypeView::UninitializedThis => {}
            PTypeView::UninitializedThisOrClass(_) => {}
        };
    }*/

    unsafe fn get_var_impl(&self, current_class_pointer: &RuntimeClassClass, class_pointer: Arc<RuntimeClass<'gc_life>>, name: impl Into<String>, mut do_class_check: bool) -> JavaValue<'gc_life> {
        let name = name.into();
        if current_class_pointer.class_view.name() == class_pointer.view().name() || !do_class_check {
            match current_class_pointer.field_numbers.get(&name) {
                Some(field_number) => {
                    return self.objinfo.fields[*field_number].get().as_ref().unwrap().clone();
                }
                None => {
                    do_class_check = false;
                }
            }
        }
        if let Some(parent_class) = current_class_pointer.parent.as_ref() {
            return self.get_var_impl(parent_class.unwrap_class_class(), class_pointer, name, do_class_check);
        } else {
            panic!()
        }
    }
}

impl<'gc_life> Debug for NormalObject<'gc_life> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        /*self.cycle_fmt(&vec![], f)*/
        todo!()
    }
}

pub fn default_value<'gc_life>(type_: PTypeView) -> JavaValue<'gc_life> {
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
        PTypeView::NullType => JavaValue::Object(todo!()/*None*/),
        PTypeView::Uninitialized(_) => unimplemented!(),
        PTypeView::UninitializedThis => unimplemented!(),
        PTypeView::UninitializedThisOrClass(_) => panic!(),
    }
}

impl<'gc_life> ArrayObject<'gc_life> {
    pub fn unwrap_object_array(&self) -> Vec<Option<GcManagedObject<'gc_life>>> {
        unsafe { self.elems.get().as_ref() }.unwrap().iter().map(|x| { x.unwrap_object() }).collect()
    }

    pub fn unwrap_mut(&self) -> &mut Vec<JavaValue<'gc_life>> {
        unsafe { self.elems.get().as_mut() }.unwrap()
    }
    pub fn unwrap_object_array_nonnull(&self) -> Vec<GcManagedObject<'gc_life>> {
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

impl<'gc_life> std::convert::From<Option<GcManagedObject<'gc_life>>> for JavaValue<'gc_life> {
    fn from(f: Option<GcManagedObject<'gc_life>>) -> Self {
        JavaValue::Object(todo!()/*f*/)
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

impl ExceptionReturn for *const u16 {
    fn invalid_default() -> Self {
        null()
    }
}


impl ExceptionReturn for *mut c_void {
    fn invalid_default() -> Self {
        null_mut()
    }
}

impl<'gc_life> ExceptionReturn for JavaValue<'gc_life> {
    fn invalid_default() -> Self {
        JavaValue::Top
    }
}

impl ExceptionReturn for () {
    fn invalid_default() -> Self {
        ()
    }
}

impl ExceptionReturn for jfieldID {
    fn invalid_default() -> Self {
        null_mut()
    }
}

impl ExceptionReturn for jmethodID {
    fn invalid_default() -> Self {
        null_mut()
    }
}