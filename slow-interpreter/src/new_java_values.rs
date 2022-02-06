use std::collections::HashMap;
use std::ptr::NonNull;
use std::sync::Arc;
use libc::c_void;
use crate::JVMState;
use crate::runtime_class::{FieldNumber, RuntimeClass, RuntimeClassClass};

pub enum NewJavaValue<'gc_life>{
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

pub enum NewJVObject<'gc_life>{
    UnAllocObject(UnAllocatedObject<'gc_life>),
    AllocObject(AllocatedObject<'gc_life>)
}

pub struct UnAllocatedObjectObject<'gc_life>{
    r#ype: Arc<RuntimeClass<'gc_life>>,
    fields: HashMap<FieldNumber, NewJavaValue<'gc_life>>
}

pub struct UnAllocatedObjectArray<'gc_life>{
    members: Vec<NewJavaValue<'gc_life>>
}

pub enum UnAllocatedObject<'gc_life>{
    Object(UnAllocatedObjectObject<'gc_life>),
    Array(UnAllocatedObjectArray<'gc_life>)
}

pub struct AllocatedObject<'gc_life>{
    jvm: &'gc_life JVMState<'gc_life>,
    ptr: NonNull<c_void>
}