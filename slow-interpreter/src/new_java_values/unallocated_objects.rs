use std::collections::HashMap;
use std::sync::Arc;
use runtime_class_stuff::{RuntimeClass};
use runtime_class_stuff::field_numbers::FieldNumber;
use crate::NewJavaValue;

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
