use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr::NonNull;
use std::sync::Arc;

use libc::c_void;

use jvmti_jni_bindings::{jbyte, jchar, jdouble, jfloat, jint, jlong};

use crate::{JavaValue, JVMState};
use crate::java_values::GcManagedObject;
use crate::runtime_class::{FieldNumber, RuntimeClass, RuntimeClassClass};

pub enum NewJavaValue<'gc_life> {
    Long(i64),
    Int(i32),
    Short(i16),
    Byte(i8),
    Boolean(u8),
    Char(u16),
    Float(f32),
    Double(f64),
    Null,
    UnAllocObject(UnAllocatedObject<'gc_life>),
    AllocObject(AllocatedObject<'gc_life>),
    Top,
}

impl<'gc_life> NewJavaValue<'gc_life> {
    pub fn to_jv(&self) -> JavaValue<'gc_life> {
        todo!()
    }
    pub fn unwrap_object(&self) -> Option<NewJVObject<'gc_life>> {
        todo!()
    }

    pub fn unwrap_object_nonnull(&self) -> NewJVObject<'gc_life> {
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
}

pub enum NewJVObject<'gc_life> {
    UnAllocObject(UnAllocatedObject<'gc_life>),
    AllocObject(AllocatedObject<'gc_life>),
}

pub enum UnAllocatedObject<'gc_life> {
    Object(UnAllocatedObjectObject<'gc_life>),
    Array(UnAllocatedObjectArray<'gc_life>),
}

impl<'gc_life> UnAllocatedObject<'gc_life> {
    pub fn new_array(whole_array_runtime_class: Arc<RuntimeClass<'gc_life>>, elems: Vec<NewJavaValue<'gc_life>>) -> Self {
        Self::Array(UnAllocatedObjectArray { whole_array_runtime_class, elems })
    }
}

pub struct UnAllocatedObjectObject<'gc_life> {
    pub(crate) object_rc: Arc<RuntimeClass<'gc_life>>,
    fields: HashMap<FieldNumber, NewJavaValue<'gc_life>>,
}

pub struct UnAllocatedObjectArray<'gc_life> {
    pub(crate) whole_array_runtime_class: Arc<RuntimeClass<'gc_life>>,
    pub(crate) elems: Vec<NewJavaValue<'gc_life>>,
}

pub struct AllocatedObject<'gc_life> {
    jvm: &'gc_life JVMState<'gc_life>,
    ptr: NonNull<c_void>,
    phantom: PhantomData<&'gc_life ()>,
}

impl<'gc_life> AllocatedObject<'gc_life> {
    pub fn to_gc_managed(&self) -> GcManagedObject<'gc_life> {
        todo!()
    }

    pub fn raw_ptr_usize(&self) -> usize{
        todo!()
    }
}

impl<'gc_life> Clone for AllocatedObject<'gc_life> {
    fn clone(&self) -> Self {
        todo!()
    }
}

pub enum NewJVArray<'gc_life> {
    UnAlloc(UnAllocatedObjectArray<'gc_life>),
    Alloc(AllocatedObject<'gc_life>),
}

impl <'gc_life> From<AllocatedObject<'gc_life>> for NewJVObject<'gc_life>{
    fn from(_: AllocatedObject<'gc_life>) -> Self {
        todo!()
    }
}

