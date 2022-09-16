use std::collections::HashMap;
use std::sync::Arc;

use runtime_class_stuff::field_numbers::FieldNumber;
use runtime_class_stuff::layout::ObjectLayout;
use runtime_class_stuff::RuntimeClass;

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

pub struct ObjectFields<'gc, 'l> {
    pub fields: HashMap<FieldNumber, NewJavaValue<'gc, 'l>>,
    pub hidden_fields: HashMap<FieldNumber, NewJavaValue<'gc, 'l>>,
}

impl<'gc, 'l> ObjectFields<'gc, 'l> {
    pub fn new(object_layout: &ObjectLayout) -> Self {
        todo!()
    }
}

#[derive(Clone)]
pub struct UnAllocatedObjectObject<'gc, 'l> {
    pub object_rc: Arc<RuntimeClass<'gc>>,
    pub object_fields: ObjectFields<'gc, 'l>,
}

#[derive(Clone)]
pub struct UnAllocatedObjectArray<'gc, 'l> {
    pub whole_array_runtime_class: Arc<RuntimeClass<'gc>>,
    pub elems: Vec<NewJavaValue<'gc, 'l>>,
}
