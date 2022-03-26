use std::collections::HashMap;
use std::ffi::c_void;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ptr::{NonNull, null_mut};
use std::sync::Arc;
use gc_memory_layout_common::NativeJavaValue;


use jvmti_jni_bindings::{jboolean, jbyte, jchar, jdouble, jfloat, jint, jlong, jshort};
use rust_jvm_common::compressed_classfile::{CompressedParsedRefType, CPDType};
use rust_jvm_common::compressed_classfile::names::{CClassName, FieldName};
use rust_jvm_common::runtime_type::{RuntimeRefType, RuntimeType};

use crate::{JavaValue, JVMState};
use crate::class_loading::assert_inited_or_initing_class;
use crate::java_values::{GcManagedObject, native_to_new_java_value};
use crate::new_java_values::array_wrapper::ArrayWrapper;
use crate::runtime_class::{FieldNumber, RuntimeClass};

pub mod array_wrapper;

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
    Object(AllocatedObjectHandle<'gc>),
    Top,
}

impl Eq for NewJavaValueHandle<'_> {}

impl PartialEq for NewJavaValueHandle<'_> {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl<'gc> NewJavaValueHandle<'gc> {
    pub fn to_jv(&self) -> JavaValue<'gc> {
        todo!()
    }

    pub fn as_njv(&self) -> NewJavaValue<'gc, '_> {
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

    pub fn null() -> NewJavaValueHandle<'gc> {
        NewJavaValueHandle::Null
    }

    pub fn unwrap_object(self) -> Option<AllocatedObjectHandle<'gc>> {
        match self {
            NewJavaValueHandle::Object(obj) => { Some(obj) }
            NewJavaValueHandle::Null => { None }
            _ => { panic!() }
        }
    }

    pub fn unwrap_object_nonnull(self) -> AllocatedObjectHandle<'gc> {
        self.unwrap_object().unwrap()
    }

    pub fn from_optional_object(obj: Option<AllocatedObjectHandle<'gc>>) -> Self {
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

    pub fn try_unwrap_object_alloc(self) -> Option<Option<AllocatedObjectHandle<'gc>>> {
        match self {
            NewJavaValueHandle::Null => Some(None),
            NewJavaValueHandle::Object(obj) => Some(Some(obj)),
            _ => None
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

    pub fn try_unwrap_object_alloc(&self) -> Option<Option<AllocatedObject<'gc, 'l>>> {
        match self {
            NewJavaValue::Null => Some(None),
            NewJavaValue::AllocObject(alloc) => {
                Some(Some(alloc.clone()))
            }
            _ => None,
        }
    }

    pub fn unwrap_object_alloc(&self) -> Option<AllocatedObject<'gc, 'l>> {
        self.try_unwrap_object_alloc().unwrap()
    }

    pub fn unwrap_object_nonnull(&self) -> NewJVObject<'gc, 'l> {
        todo!()
    }

    pub fn unwrap_bool_strict(&self) -> jboolean {
        match self {
            NewJavaValue::Boolean(bool) => *bool,
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_byte_strict(&self) -> jbyte {
        match self {
            NewJavaValue::Byte(byte) => {
                *byte
            }
            _ => panic!()
        }
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
        match self {
            NewJavaValue::Int(res) => *res,
            _ => {
                panic!()
            }
        }
    }

    pub fn unwrap_int(&self) -> jint {
        match self {
            NewJavaValue::Int(int) => {
                *int
            }
            NewJavaValue::Short(short) => {
                *short as jint
            }
            NewJavaValue::Byte(byte) => {
                *byte as jint
            }
            NewJavaValue::Boolean(bool) => {
                *bool as jint
            }
            NewJavaValue::Char(char) => {
                *char as jint
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

    pub fn to_native(&self) -> NativeJavaValue<'gc> {
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

    pub fn to_handle_discouraged(&self) -> NewJavaValueHandle<'gc> {
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

    pub fn rtype(&self, jvm: &'gc JVMState<'gc>) -> RuntimeType {
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
            NewJavaValue::Null => { CPDType::Ref(CompressedParsedRefType::Class(CClassName::object())) }
            NewJavaValue::UnAllocObject(_) => { todo!() }
            NewJavaValue::AllocObject(obj) => { obj.runtime_class(jvm).cpdtype() }
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
    AllocObject(AllocatedObject<'gc, 'l>),
}

impl<'gc, 'l> NewJVObject<'gc, 'l> {
    pub fn unwrap_alloc(&self) -> AllocatedObject<'gc, 'l> {
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

#[derive(Clone)]
pub enum UnAllocatedObject<'gc, 'l> {
    Object(UnAllocatedObjectObject<'gc, 'l>),
    Array(UnAllocatedObjectArray<'gc, 'l>),
}

impl<'gc, 'l> UnAllocatedObject<'gc, 'l> {
    pub fn new_array(whole_array_runtime_class: Arc<RuntimeClass<'gc>>, elems: Vec<NewJavaValue<'gc, 'l>>) -> Self {
        Self::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems })
    }
}

#[derive(Clone)]
pub struct UnAllocatedObjectObject<'gc, 'l> {
    pub object_rc: Arc<RuntimeClass<'gc>>,
    pub fields: HashMap<FieldNumber, NewJavaValue<'gc, 'l>>,
}

#[derive(Clone)]
pub struct UnAllocatedObjectArray<'gc, 'l> {
    pub whole_array_runtime_class: Arc<RuntimeClass<'gc>>,
    pub elems: Vec<NewJavaValue<'gc, 'l>>,
}

pub struct AllocatedObject<'gc, 'l> {
    pub handle: &'l AllocatedObjectHandle<'gc>,//todo put in same module as gc
}

impl<'gc, 'any> AllocatedObject<'gc, 'any> {
    pub fn to_gc_managed(&self) -> GcManagedObject<'gc> {
        todo!()
    }

    pub fn raw_ptr_usize(&self) -> usize {
        self.handle.ptr.as_ptr() as usize
    }

    pub fn set_var(&self, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: FieldName, val: NewJavaValue<'gc, 'any>) {
        let field_number = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().0;
        unsafe {
            self.handle.ptr.cast::<NativeJavaValue<'gc>>().as_ptr().offset(field_number.0 as isize).write(val.to_native());
        }
    }

    pub fn set_var_top_level(&self, jvm: &'gc JVMState<'gc>, field_name: FieldName, val: NewJavaValue<'gc, 'any>) {
        let current_class_pointer = self.runtime_class(jvm);
        self.set_var(&current_class_pointer, field_name, val)
    }

    pub fn get_var(&self, jvm: &'gc JVMState<'gc>, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: FieldName) -> NewJavaValueHandle<'gc> {
        let (field_number, desc_type) = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap();
        self.raw_get_var(jvm, *field_number, *desc_type)
    }

    pub fn raw_get_var(&self, jvm: &'gc JVMState<'gc>, number: FieldNumber, cpdtype: CPDType) -> NewJavaValueHandle<'gc> {
        unsafe {
            let native_jv = self.handle.ptr.cast::<NativeJavaValue<'gc>>().as_ptr().offset(number.0 as isize).read();
            native_to_new_java_value(native_jv,&cpdtype, jvm)
        }
    }

    pub fn get_var_top_level(&self, jvm: &'gc JVMState<'gc>, field_name: FieldName) -> NewJavaValueHandle<'gc> {
        let current_class_pointer = self.runtime_class(jvm);
        self.get_var(jvm, &current_class_pointer, field_name)
    }

    pub fn lookup_field(&self, current_class_pointer: &Arc<RuntimeClass<'gc>>, field_name: FieldName) -> NewJavaValueHandle<'gc> {
        let field_number = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().0;
        let cpdtype = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().1;
        let jvm = self.handle.jvm;
        let native_jv = unsafe { self.handle.ptr.cast::<NativeJavaValue>().as_ptr().offset(field_number.0 as isize).read() };
        native_to_new_java_value(native_jv,cpdtype, jvm)
    }

    pub fn runtime_class(&self, jvm: &'gc JVMState<'gc>) -> Arc<RuntimeClass<'gc>> {
        let allocated_obj_type = jvm.gc.memory_region.lock().unwrap().find_object_allocated_type(self.handle.ptr).clone();
        assert_inited_or_initing_class(jvm, allocated_obj_type.as_cpdtype())
    }
}

impl<'gc> Clone for AllocatedObject<'gc, '_> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle
        }
    }
}

pub enum NewJVArray<'gc, 'l> {
    UnAlloc(UnAllocatedObjectArray<'gc, 'l>),
    Alloc(AllocatedObject<'gc, 'l>),
}

impl<'gc, 'l> From<AllocatedObject<'gc, 'l>> for NewJVObject<'gc, 'l> {
    fn from(_: AllocatedObject<'gc, 'l>) -> Self {
        todo!()
    }
}


pub struct AllocatedObjectHandle<'gc> {
    /*pub(in crate::java_values)*/
    pub(crate) jvm: &'gc JVMState<'gc>,
    //todo move gc to same crate
    /*pub(in crate::java_values)*/
    pub ptr: NonNull<c_void>,
}

impl Eq for AllocatedObjectHandle<'_> {}


impl<'gc> PartialEq for AllocatedObjectHandle<'gc> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<'gc> AllocatedObjectHandle<'gc> {
    pub fn new_java_value(&self) -> NewJavaValue<'gc, '_> {
        NewJavaValue::AllocObject(self.as_allocated_obj())
    }

    pub fn as_allocated_obj(&self) -> AllocatedObject<'gc, '_> {
        AllocatedObject { handle: self }
    }

    pub fn to_jv<'any>(&'any self) -> JavaValue<'gc> {
        todo!()
    }

    pub fn duplicate_discouraged(&self) -> Self {
        self.jvm.gc.register_root_reentrant(self.jvm, self.ptr)
    }

    pub fn is_array(&self, jvm: &'gc JVMState<'gc>) -> bool {
        let rc = self.as_allocated_obj().runtime_class(jvm);
        rc.cpdtype().is_array()
    }

    pub fn unwrap_array(&self, jvm: &'gc JVMState<'gc>) -> ArrayWrapper<'gc, '_> {
        assert!(self.is_array(jvm));
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


pub struct AllocatedObjectHandleByAddress<'gc>(pub AllocatedObjectHandle<'gc>);

impl Hash for AllocatedObjectHandleByAddress<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.ptr.as_ptr() as usize);
    }
}

impl PartialEq for AllocatedObjectHandleByAddress<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.ptr == other.0.ptr
    }
}

impl Eq for AllocatedObjectHandleByAddress<'_> {}

pub enum AllocatedObjectCOW<'gc, 'k> {
    Handle(AllocatedObjectHandle<'gc>),
    Ref(AllocatedObject<'gc, 'k>),
}

impl<'gc, 'k> AllocatedObjectCOW<'gc, 'k> {
    pub fn as_allocated_object(&'k self) -> AllocatedObject<'gc, 'k> {
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