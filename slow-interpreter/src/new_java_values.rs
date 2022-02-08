use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ptr::{NonNull, null_mut};
use std::sync::Arc;

use libc::c_void;

use jvmti_jni_bindings::{jbyte, jchar, jdouble, jfloat, jint, jlong};
use rust_jvm_common::compressed_classfile::names::FieldName;

use crate::{JavaValue, JVMState};
use crate::java_values::{GcManagedObject, NativeJavaValue};
use crate::jvm_state::Native;
use crate::runtime_class::{FieldNumber, RuntimeClass, RuntimeClassClass};

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
            NewJavaValueHandle::Short(_) => {
                todo!()
            }
            NewJavaValueHandle::Byte(_) => {
                todo!()
            }
            NewJavaValueHandle::Boolean(bool) => {
                NewJavaValue::Boolean(*bool)
            }
            NewJavaValueHandle::Char(_) => {
                todo!()
            }
            NewJavaValueHandle::Float(_) => {
                todo!()
            }
            NewJavaValueHandle::Double(_) => {
                todo!()
            }
            NewJavaValueHandle::Null => {
                NewJavaValue::Null
            }
            NewJavaValueHandle::Object(_) => {
                todo!()
            }
            NewJavaValueHandle::Top => {
                todo!()
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

    pub fn unwrap_object_alloc(&self) -> Option<AllocatedObject<'gc_life, 'l>> {
        match self {
            NewJavaValue::Null => None,
            NewJavaValue::AllocObject(alloc) => {
                Some(alloc.clone())
            }
            _ => panic!(),
        }
    }

    pub fn unwrap_object_nonnull(&self) -> NewJVObject<'gc_life, 'l> {
        todo!()
    }

    pub fn unwrap_bool_strict(&self) -> jbyte {
        todo!()
    }

    pub fn unwrap_byte_strict(&self) -> jbyte {
        todo!()
    }

    pub fn unwrap_char_strict(&self) -> jchar {
        todo!()
    }

    pub fn unwrap_short_strict(&self) -> jchar {
        todo!()
    }

    pub fn unwrap_int_strict(&self) -> jint {
        todo!()
    }

    pub fn unwrap_int(&self) -> jint {
        todo!()
    }

    pub fn unwrap_long_strict(&self) -> jlong {
        todo!()
    }

    pub fn unwrap_float_strict(&self) -> jfloat {
        todo!()
    }

    pub fn unwrap_double_strict(&self) -> jdouble {
        todo!()
    }

    pub fn to_native(&self) -> NativeJavaValue<'gc_life> {
        match self {
            NewJavaValue::Long(long) => {
                NativeJavaValue { long: *long }
            }
            NewJavaValue::Int(int) => {
                NativeJavaValue { int: *int }
            }
            NewJavaValue::Short(_) => {
                todo!()
            }
            NewJavaValue::Byte(_) => {
                todo!()
            }
            NewJavaValue::Boolean(bool) => {
                NativeJavaValue { boolean: *bool }
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
                NativeJavaValue { object: null_mut() }
            }
            NewJavaValue::UnAllocObject(_) => {
                todo!()
            }
            NewJavaValue::AllocObject(_) => {
                todo!()
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

    pub fn set_var(&self, current_class_pointer: &Arc<RuntimeClass<'gc_life>>, field_name: FieldName, val: NewJavaValueHandle<'gc_life>) {
        let field_number = &current_class_pointer.unwrap_class_class().field_numbers.get(&field_name).unwrap().0;
        unsafe {
            self.handle.ptr.cast::<NativeJavaValue<'gc_life>>().as_ptr().offset(field_number.0 as isize).write(val.as_njv().to_native());
        }
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