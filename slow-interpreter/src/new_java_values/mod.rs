use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::ptr::null_mut;
use std::sync::Arc;
use itertools::Itertools;

use runtime_class_stuff::RuntimeClass;
use rust_jvm_common::compressed_classfile::{CPDType};
use rust_jvm_common::compressed_classfile::names::CClassName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::{JavaValue, JVMState};
use crate::interpreter::real_interpreter_state::InterpreterJavaValue;
use crate::java_values::{default_value_njv};
use crate::new_java_values::allocated_objects::{AllocatedHandle, AllocatedNormalObjectHandle, AllocatedObject};
use crate::new_java_values::java_value_common::JavaValueCommon;
use crate::new_java_values::unallocated_objects::{UnAllocatedObject, UnAllocatedObjectArray};

pub mod unallocated_objects;
pub mod allocated_objects;
pub mod java_value_common;
pub mod owned_casts;

#[derive(Debug)]
pub enum NewJavaValueHandle<'gc> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),
    Float(f32),
    Double(f64),
    Null,
    Object(AllocatedHandle<'gc>),
    Top,
}

impl Eq for NewJavaValueHandle<'_> {}

impl PartialEq for NewJavaValueHandle<'_> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'gc> JavaValueCommon<'gc> for NewJavaValueHandle<'gc> {
    fn as_njv(&self) -> NewJavaValue<'gc, '_> {
        match self {
            NewJavaValueHandle::Long(long) => {
                NewJavaValue::Long(*long)
            }
            NewJavaValueHandle::Int(int) => {
                NewJavaValue::Int(*int)
            }
            NewJavaValueHandle::Short(short) => {
                NewJavaValue::Short(*short)
            }
            NewJavaValueHandle::Byte(byte) => {
                NewJavaValue::Byte(*byte)
            }
            NewJavaValueHandle::Boolean(bool) => {
                NewJavaValue::Boolean(*bool)
            }
            NewJavaValueHandle::Char(char) => {
                NewJavaValue::Char(*char)
            }
            NewJavaValueHandle::Float(float) => {
                NewJavaValue::Float(*float)
            }
            NewJavaValueHandle::Double(double) => {
                NewJavaValue::Double(*double)
            }
            NewJavaValueHandle::Null => {
                NewJavaValue::Null
            }
            NewJavaValueHandle::Object(obj) => {
                NewJavaValue::AllocObject(AllocatedObject::Handle(obj))
            }
            NewJavaValueHandle::Top => {
                NewJavaValue::Top
            }
        }
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn null() -> NewJavaValueHandle<'gc> {
        NewJavaValueHandle::Null
    }

    pub fn unwrap_object(self) -> Option<AllocatedHandle<'gc>> {
        match self {
            NewJavaValueHandle::Object(obj) => { Some(obj) }
            NewJavaValueHandle::Null => { None }
            _ => { panic!() }
        }
    }

    pub fn unwrap_object_nonnull(self) -> AllocatedHandle<'gc> {
        self.unwrap_object().unwrap()
    }

    pub fn from_optional_object(obj: Option<AllocatedHandle<'gc>>) -> Self {
        match obj {
            None => {
                Self::Null
            }
            Some(obj) => {
                Self::Object(obj)
            }
        }
    }

    pub fn empty_byte_array(jvm: &'gc JVMState<'gc>, empty_byte_array: Arc<RuntimeClass<'gc>>) -> Self {
        Self::Object(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class: empty_byte_array, elems: vec![] })))
    }

    pub fn new_default_array(jvm: &'gc JVMState<'gc>, len: i32, whole_array_runtime_class: Arc<RuntimeClass<'gc>>, elem_type: CPDType) -> Self{
        let elems = (0..len).map(|_|default_value_njv(&elem_type)).collect_vec();
        Self::Object(jvm.allocate_object(UnAllocatedObject::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems })))
    }

    pub fn try_unwrap_object_alloc(self) -> Option<Option<AllocatedHandle<'gc>>> {
        match self {
            NewJavaValueHandle::Null => Some(None),
            NewJavaValueHandle::Object(obj) => Some(Some(obj)),
            _ => None
        }
    }

    pub fn to_interpreter_jv(&self) -> InterpreterJavaValue{
        match self {
            NewJavaValueHandle::Long(long) => {
                InterpreterJavaValue::Long(*long)
            }
            NewJavaValueHandle::Int(int) => {
                InterpreterJavaValue::Int(*int)
            }
            NewJavaValueHandle::Short(short) => {
                InterpreterJavaValue::Int(*short as i32)
            }
            NewJavaValueHandle::Byte(byte) => {
                InterpreterJavaValue::Int(*byte as i32)
            }
            NewJavaValueHandle::Boolean(bool) => {
                InterpreterJavaValue::Int(*bool as i32)
            }
            NewJavaValueHandle::Char(char) => {
                InterpreterJavaValue::Int(*char as i32)
            }
            NewJavaValueHandle::Float(float) => {
                InterpreterJavaValue::Float(*float)
            }
            NewJavaValueHandle::Double(double) => {
                InterpreterJavaValue::Double(*double)
            }
            NewJavaValueHandle::Null => {
                InterpreterJavaValue::Object(None)
            }
            NewJavaValueHandle::Object(obj) => {
                InterpreterJavaValue::Object(Some(obj.ptr()))
            }
            NewJavaValueHandle::Top => {
                panic!()
            }
        }
    }
}


#[derive(Clone)]
pub enum NewJavaValue<'gc, 'l> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),
    Float(f32),
    Double(f64),
    Null,
    UnAllocObject(UnAllocatedObject<'gc, 'l>),
    AllocObject(AllocatedObject<'gc, 'l>),
    Top,
}

impl<'gc> JavaValueCommon<'gc> for NewJavaValue<'gc, '_> {
    fn as_njv(&self) -> NewJavaValue<'gc, '_> {
        self.clone()
    }
}

impl<'gc, 'l> Debug for NewJavaValue<'gc, 'l> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NewJavaValue::Long(elem) => {
                write!(f, "Long:{}", elem)
            }
            NewJavaValue::Int(elem) => {
                write!(f, "Int:{}", elem)
            }
            NewJavaValue::Short(elem) => {
                write!(f, "Short:{}", elem)
            }
            NewJavaValue::Byte(elem) => {
                write!(f, "Byte:{}", elem)
            }
            NewJavaValue::Boolean(elem) => {
                write!(f, "Boolean:{}", elem)
            }
            NewJavaValue::Char(elem) => {
                write!(f, "Char:{}", elem)
            }
            NewJavaValue::Float(elem) => {
                write!(f, "Float:{}", elem)
            }
            NewJavaValue::Double(elem) => {
                write!(f, "Double:{}", elem)
            }
            NewJavaValue::AllocObject(obj) => {
                write!(f, "obj:{:?}", obj.ptr())
            }
            NewJavaValue::Top => {
                write!(f, "top")
            }
            NewJavaValue::Null => {
                write!(f, "obj:{:?}", null_mut::<c_void>())
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!()
            }
        }
    }
}

impl<'gc, 'l> NewJavaValue<'gc, 'l> {
    pub fn to_jv(&self) -> JavaValue<'gc> {
        todo!()
    }

    pub fn unwrap_object(&self) -> Option<NewJVObject<'gc, 'l>> {
        match self {
            NewJavaValue::Null => None,
            NewJavaValue::UnAllocObject(obj) => {
                Some(NewJVObject::UnAllocObject(obj.clone()))//todo maybe this shouldn't clone
            }
            NewJavaValue::AllocObject(obj) => {
                Some(NewJVObject::AllocObject(obj.clone()))
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn try_unwrap_object_alloc(&self) -> Option<Option<AllocatedObject<'gc,'l>>> {
        match self {
            NewJavaValue::Null => Some(None),
            NewJavaValue::AllocObject(alloc) => {
                Some(Some(alloc.clone()))
            }
            _ => None,
        }
    }

    pub fn unwrap_normal_object(&self) -> Option<&'l AllocatedNormalObjectHandle<'gc>>{
        if let NewJavaValue::AllocObject(obj) = self {
            match obj {
                AllocatedObject::Handle(handle) => {
                    if let AllocatedHandle::NormalObject(normal_object) = handle {
                        return Some(normal_object)
                    }
                }
                AllocatedObject::NormalObject(normal_object) => {
                    return Some(normal_object)
                }
                AllocatedObject::ArrayObject(_) => {}
            }
        };
        return None
    }

    pub fn unwrap_object_alloc(&self) -> Option<AllocatedObject<'gc ,'l>> {
        self.try_unwrap_object_alloc().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> NewJVObject<'gc, 'l> {
        todo!()
    }


    pub fn to_handle_discouraged(&self) -> NewJavaValueHandle<'gc> {
        match self {
            NewJavaValue::Long(long) => {
                NewJavaValueHandle::Long(*long)
            }
            NewJavaValue::Int(int) => {
                NewJavaValueHandle::Int(*int)
            }
            NewJavaValue::Short(short) => {
                NewJavaValueHandle::Short(*short)
            }
            NewJavaValue::Byte(byte) => {
                NewJavaValueHandle::Byte(*byte)
            }
            NewJavaValue::Boolean(bool) => {
                NewJavaValueHandle::Boolean(*bool)
            }
            NewJavaValue::Char(char) => {
                NewJavaValueHandle::Char(*char)
            }
            NewJavaValue::Float(float) => {
                NewJavaValueHandle::Float(*float)
            }
            NewJavaValue::Double(double) => {
                NewJavaValueHandle::Double(*double)
            }
            NewJavaValue::Null => {
                NewJavaValueHandle::Null
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!("wtf do I do here")
            }
            NewJavaValue::AllocObject(obj) => {
                obj.duplicate_discouraged().new_java_value_handle()
            }
            NewJavaValue::Top => {
                todo!()
            }
        }
    }

    pub fn rtype(&self, jvm: &'gc JVMState<'gc>) -> RuntimeType {
        match self {
            NewJavaValue::Long(_) => {
                RuntimeType::LongType
            }
            NewJavaValue::Int(_) => {
                RuntimeType::IntType
            }
            NewJavaValue::Short(_) => {
                RuntimeType::IntType
            }
            NewJavaValue::Byte(_) => {
                RuntimeType::IntType
            }
            NewJavaValue::Boolean(_) => {
                RuntimeType::IntType
            }
            NewJavaValue::Char(_) => {
                RuntimeType::IntType
            }
            NewJavaValue::Float(_) => {
                RuntimeType::FloatType
            }
            NewJavaValue::Double(_) => {
                RuntimeType::DoubleType
            }
            NewJavaValue::Null => {
                RuntimeType::Ref(RuntimeRefType::NullType)
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!()
            }
            NewJavaValue::AllocObject(obj) => {
                RuntimeType::Ref(obj.runtime_class(jvm).view().name().to_runtime_type())
            }
            NewJavaValue::Top => {
                RuntimeType::TopType
            }
        }
    }

    pub fn to_type(&self, jvm: &'gc JVMState<'gc>) -> CPDType {
        match self {
            NewJavaValue::Long(_) => { CPDType::LongType }
            NewJavaValue::Int(_) => { CPDType::IntType }
            NewJavaValue::Short(_) => { CPDType::ShortType }
            NewJavaValue::Byte(_) => { CPDType::ByteType }
            NewJavaValue::Boolean(_) => { CPDType::BooleanType }
            NewJavaValue::Char(_) => { CPDType::CharType }
            NewJavaValue::Float(_) => { CPDType::FloatType }
            NewJavaValue::Double(_) => { CPDType::DoubleType }
            NewJavaValue::Null => { CClassName::object().into() }
            NewJavaValue::UnAllocObject(_) => { todo!() }
            NewJavaValue::AllocObject(obj) => { obj.unwrap_normal_object().runtime_class(jvm).cpdtype() }
            NewJavaValue::Top => panic!()
        }
    }

    pub fn to_type_basic(&self) -> CPDType {
        match self {
            NewJavaValue::Long(_) => { CPDType::LongType }
            NewJavaValue::Int(_) => { CPDType::IntType }
            NewJavaValue::Short(_) => { CPDType::ShortType }
            NewJavaValue::Byte(_) => { CPDType::ByteType }
            NewJavaValue::Boolean(_) => { CPDType::BooleanType }
            NewJavaValue::Char(_) => { CPDType::CharType }
            NewJavaValue::Float(_) => { CPDType::FloatType }
            NewJavaValue::Double(_) => { CPDType::DoubleType }
            NewJavaValue::Null => { CClassName::object().into() }
            NewJavaValue::UnAllocObject(_) => { CPDType::object().into() }
            NewJavaValue::AllocObject(_) => { CPDType::object().into() }
            NewJavaValue::Top => panic!()
        }
    }
}

pub enum NewJVObject<'gc, 'l> {
    UnAllocObject(UnAllocatedObject<'gc, 'l>),
    AllocObject(AllocatedObject<'gc,'l>),
}

impl<'gc, 'l> NewJVObject<'gc, 'l> {
    pub fn unwrap_alloc(&self) -> AllocatedObject<'gc,'l> {
        match self {
            NewJVObject::UnAllocObject(_) => panic!(),
            NewJVObject::AllocObject(alloc_obj) => {
                alloc_obj.clone()
            }
        }
    }

    pub fn to_jv(&self) -> JavaValue<'gc> {
        todo!()
    }
}


