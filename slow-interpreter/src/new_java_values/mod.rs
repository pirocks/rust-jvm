use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::{NonNull, null_mut};
use std::sync::Arc;

use iced_x86::CC_b::c;
use iced_x86::ConditionCode::p;
use iced_x86::OpCodeOperandKind::al;
use libc::c_void;

use gc_memory_layout_common::AllocatedObjectType;
use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};
use rust_jvm_common::compressed_classfile::CPDType;
use rust_jvm_common::compressed_classfile::names::FieldName;
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::{JavaValue, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java_values::{GcManagedObject, NativeJavaValue};
use crate::jvm_state::Native;
use crate::new_java_values::array_wrapper::ArrayWrapper;
use crate::runtime_class::{FieldNumber, RuntimeClass, RuntimeClassClass};

pub mod array_wrapper;

#[derive(Debug)]
pub enum NewJavaValueHandle<'gc_life> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),
    Float(f32),
    Double(f64),
    Null,
    Object(AllocatedObjectHandle<'gc_life>),
    Top,
}

impl Eq for NewJavaValueHandle<'_> {

}

impl PartialEq for NewJavaValueHandle<'_>{
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'gc_life> NewJavaValueHandle<'gc_life> {
    pub fn to_jv(&self) -> JavaValue<'gc_life> {
        todo!()
    }

    pub fn as_njv(&self) -> NewJavaValue<'gc_life, '_> {
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
                NewJavaValue::AllocObject(AllocatedObject { handle: obj })
            }
            NewJavaValueHandle::Top => {
                todo!()
            }
        }
    }

    pub fn unwrap_object(self) -> Option<AllocatedObjectHandle<'gc_life>> {
        match self {
            NewJavaValueHandle::Object(obj) => { Some(obj) }
            NewJavaValueHandle::Null => { None }
            _ => { panic!() }
        }
    }

    pub fn unwrap_object_nonnull(self) -> AllocatedObjectHandle<'gc_life> {
        self.unwrap_object().unwrap()
    }

    pub fn from_optional_object(obj: Option<AllocatedObjectHandle<'gc_life>>) -> Self{
        match obj {
            None => {
                Self::Null
            }
            Some(obj) => {
                Self::Object(obj)
            }
        }
    }
}


#[derive(Clone)]
pub enum NewJavaValue<'gc_life, 'l> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),
    Float(f32),
    Double(f64),
    Null,
    UnAllocObject(UnAllocatedObject<'gc_life, 'l>),
    AllocObject(AllocatedObject<'gc_life, 'l>),
    Top,
}

impl<'gc_life, 'l> Debug for NewJavaValue<'gc_life, 'l> {
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
                write!(f, "obj:{:?}", obj.handle.ptr)
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

impl<'gc_life, 'l> NewJavaValue<'gc_life, 'l> {
    pub fn to_jv(&self) -> JavaValue<'gc_life> {
        todo!()
    }

    pub fn unwrap_object(&self) -> Option<NewJVObject<'gc_life, 'l>> {
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

    pub fn try_unwrap_object_alloc(&self) -> Option<Option<AllocatedObject<'gc_life, 'l>>> {
        match self {
            NewJavaValue::Null => Some(None),
            NewJavaValue::AllocObject(alloc) => {
                Some(Some(alloc.clone()))
            }
            _ => None,
        }
    }

    pub fn unwrap_object_alloc(&self) -> Option<AllocatedObject<'gc_life, 'l>> {
        self.try_unwrap_object_alloc().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> NewJVObject<'gc_life, 'l> {
        todo!()
    }

    pub fn unwrap_bool_strict(&self) -> jboolean {
        todo!()
    }

    pub fn unwrap_byte_strict(&self) -> jbyte {
        todo!()
    }

    pub fn unwrap_char_strict(&self) -> jchar {
        match self {
            NewJavaValue::Char(char) => {
                *char
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_short_strict(&self) -> jshort {
        todo!()
    }

    pub fn unwrap_int_strict(&self) -> jint {
        todo!()
    }

    pub fn unwrap_int(&self) -> jint {
        match self {
            NewJavaValue::Int(int) => {
                *int
            }
            _ => panic!()
        }
    }

    pub fn unwrap_long_strict(&self) -> jlong {
        match self {
            NewJavaValue::Long(long) => {
                *long
            }
            _ => panic!()
        }
    }

    pub fn unwrap_float_strict(&self) -> jfloat {
        match self {
            NewJavaValue::Float(float) => {
                *float
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_double_strict(&self) -> jdouble {
        match self {
            NewJavaValue::Double(double) => {
                *double
            }
            _ => {
                panic!()
            }
        }
    }

    pub fn to_native(&self) -> NativeJavaValue<'gc_life> {
        let mut all_zero = NativeJavaValue { as_u64: 0 };
        match self {
            NewJavaValue::Long(long) => {
                all_zero.long = *long;
            }
            NewJavaValue::Int(int) => {
                all_zero.int = *int;
            }
            NewJavaValue::Short(short) => {
                all_zero.short = *short;
            }
            NewJavaValue::Byte(byte) => {
                all_zero.byte = *byte;
            }
            NewJavaValue::Boolean(bool) => {
                all_zero.boolean = *bool;
            }
            NewJavaValue::Char(char) => {
                all_zero.char = *char;
            }
            NewJavaValue::Float(float) => {
                all_zero.float = *float;
            }
            NewJavaValue::Double(double) => {
                all_zero.double = *double;
            }
            NewJavaValue::Null => {
                all_zero.object = null_mut();
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!()
            }
            NewJavaValue::AllocObject(obj) => {
                all_zero.object = obj.handle.ptr.as_ptr();
            }
            NewJavaValue::Top => {
                all_zero.as_u64 = 0xdddd_dddd_dddd_dddd;
            }
        }
        all_zero
    }

    pub fn to_handle_discouraged(&self) -> NewJavaValueHandle<'gc_life> {
        match self {
            NewJavaValue::Long(_) => {
                todo!()
            }
            NewJavaValue::Int(_) => {
                todo!()
            }
            NewJavaValue::Short(_) => {
                todo!()
            }
            NewJavaValue::Byte(_) => {
                todo!()
            }
            NewJavaValue::Boolean(_) => {
                todo!()
            }
            NewJavaValue::Char(_) => {
                todo!()
            }
            NewJavaValue::Float(_) => {
                todo!()
            }
            NewJavaValue::Double(_) => {
                todo!()
            }
            NewJavaValue::Null => {
                todo!()
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!("wtf do I do here")
            }
            NewJavaValue::AllocObject(obj) => {
                NewJavaValueHandle::Object(obj.handle.duplicate_discouraged())
            }
            NewJavaValue::Top => {
                todo!()
            }
        }
    }

    pub fn rtype(&self, jvm: &'gc_life JVMState<'gc_life>) -> RuntimeType{
        match self {
            NewJavaValue::Long(_) => {
                RuntimeType::LongType
            }
            NewJavaValue::Int(_) => {
                todo!()
            }
            NewJavaValue::Short(_) => {
                todo!()
            }
            NewJavaValue::Byte(_) => {
                todo!()
            }
            NewJavaValue::Boolean(_) => {
                RuntimeType::IntType
            }
            NewJavaValue::Char(_) => {
                todo!()
            }
            NewJavaValue::Float(_) => {
                todo!()
            }
            NewJavaValue::Double(_) => {
                todo!()
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
                todo!()
            }
        }
    }
}

pub enum NewJVObject<'gc_life, 'l> {
    UnAllocObject(UnAllocatedObject<'gc_life, 'l>),
    AllocObject(AllocatedObject<'gc_life, 'l>),
}

impl<'gc_life, 'l> NewJVObject<'gc_life, 'l> {
    pub fn unwrap_alloc(&self) -> AllocatedObject<'gc_life, 'l> {
        match self {
            NewJVObject::UnAllocObject(_) => panic!(),
            NewJVObject::AllocObject(alloc_obj) => {
                alloc_obj.clone()
            }
        }
    }

    pub fn to_jv(&self) -> JavaValue<'gc_life> {
        todo!()
    }
}

#[derive(Clone)]
pub enum UnAllocatedObject<'gc_life, 'l> {
    Object(UnAllocatedObjectObject<'gc_life, 'l>),
    Array(UnAllocatedObjectArray<'gc_life, 'l>),
}

impl<'gc_life, 'l> UnAllocatedObject<'gc_life, 'l> {
    pub fn new_array(whole_array_runtime_class: Arc<RuntimeClass<'gc_life>>, elems: Vec<NewJavaValue<'gc_life, 'l>>) -> Self {
        Self::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems })
    }
}

#[derive(Clone)]
pub struct UnAllocatedObjectObject<'gc_life, 'l> {
    pub(crate) object_rc: Arc<RuntimeClass<'gc_life>>,
    pub(crate) fields: HashMap<FieldNumber, NewJavaValue<'gc_life, 'l>>,
}

#[derive(Clone)]
pub struct UnAllocatedObjectArray<'gc_life, 'l> {
    pub(crate) whole_array_runtime_class: Arc<RuntimeClass<'gc_life>>,
    pub(crate) elems: Vec<NewJavaValue<'gc_life, 'l>>,
}

pub struct AllocatedObject<'gc_life, 'l> {
    pub(crate) handle: &'l AllocatedObjectHandle<'gc_life>,//todo put in same module as gc
}

impl<'gc_life, 'any> AllocatedObject<'gc_life, 'any> {
    pub fn to_gc_managed(&self) -> GcManagedObject<'gc_life> {
        todo!()
    }

    pub fn raw_ptr_usize(&self) -> usize {
        self.handle.ptr.as_ptr() as usize
    }

    pub fn set_var(&self, current_class_pointer: &Arc<RuntimeClass<'gc_life>>, field_name: FieldName, val: NewJavaValue<'gc_life, 'any>) {
        let field_number = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().0;
        unsafe {
            self.handle.ptr.cast::<NativeJavaValue<'gc_life>>().as_ptr().offset(field_number.0 as isize).write(val.to_native());
        }
    }

    pub fn lookup_field(&self, current_class_pointer: &Arc<RuntimeClass<'gc_life>>, field_name: FieldName) -> NewJavaValueHandle<'gc_life> {
        let field_number = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().0;
        let cpdtype = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().1;
        let jvm = self.handle.jvm;
        let native_jv = unsafe { self.handle.ptr.cast::<NativeJavaValue>().as_ptr().offset(field_number.0 as isize).read() };
        native_jv.to_new_java_value(cpdtype, jvm)
    }

    pub fn runtime_class(&self, jvm: &'gc_life JVMState<'gc_life>) -> Arc<RuntimeClass<'gc_life>> {
        let guard = jvm.gc.memory_region.lock().unwrap();
        let allocated_obj_type = guard.find_object_allocated_type(self.handle.ptr).clone();
        drop(guard);
        assert_inited_or_initing_class(jvm, allocated_obj_type.as_cpdtype())
    }
}

impl<'gc_life> Clone for AllocatedObject<'gc_life, '_> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle
        }
    }
}

pub enum NewJVArray<'gc_life, 'l> {
    UnAlloc(UnAllocatedObjectArray<'gc_life, 'l>),
    Alloc(AllocatedObject<'gc_life, 'l>),
}

impl<'gc_life, 'l> From<AllocatedObject<'gc_life, 'l>> for NewJVObject<'gc_life, 'l> {
    fn from(_: AllocatedObject<'gc_life, 'l>) -> Self {
        todo!()
    }
}


pub struct AllocatedObjectHandle<'gc_life> {
    /*pub(in crate::java_values)*/
    pub(crate) jvm: &'gc_life JVMState<'gc_life>,
    //todo move gc to same crate
    /*pub(in crate::java_values)*/
    pub(crate) ptr: NonNull<c_void>,
}

impl Eq for AllocatedObjectHandle<'_> {

}

impl <'gc_life> PartialEq for AllocatedObjectHandle<'gc_life> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<'gc_life> AllocatedObjectHandle<'gc_life> {
    pub fn new_java_value(&self) -> NewJavaValue<'gc_life, '_> {
        NewJavaValue::AllocObject(self.as_allocated_obj())
    }

    pub fn as_allocated_obj(&self) -> AllocatedObject<'gc_life, '_> {
        AllocatedObject { handle: self }
    }

    pub fn to_jv(&self) -> JavaValue<'gc_life> {
        todo!()
    }

    pub fn duplicate_discouraged(&self) -> Self {
        self.jvm.gc.register_root_reentrant(self.jvm, self.ptr)
    }

    pub fn unwrap_array(&self) -> ArrayWrapper<'gc_life, '_> {
        ArrayWrapper {
            allocated_object: self.as_allocated_obj()
        }
    }
}

impl Debug for AllocatedObjectHandle<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}


impl Drop for AllocatedObjectHandle<'_> {
    fn drop(&mut self) {
        self.jvm.gc.deregister_root_reentrant(self.ptr)
    }
}


pub enum AllocatedObjectCOW<'gc_life, 'k> {
    Handle(AllocatedObjectHandle<'gc_life>),
    Ref(AllocatedObject<'gc_life, 'k>),
}

impl<'gc_life, 'k> AllocatedObjectCOW<'gc_life, 'k> {
    pub fn as_allocated_object(&'k self) -> AllocatedObject<'gc_life, 'k> {
        match self {
            AllocatedObjectCOW::Handle(handle) => {
                handle.as_allocated_obj()
            }
            AllocatedObjectCOW::Ref(allocated_object) => {
                allocated_object.clone()
            }
        }
    }
}