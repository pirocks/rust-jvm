use std::collections::HashMap;
use std::sync::Arc;

use runtime_class_stuff::field_numbers::FieldNumber;
use runtime_class_stuff::layout::ObjectLayout;
use runtime_class_stuff::{FieldNameAndFieldType, RuntimeClass};
use runtime_class_stuff::hidden_fields::HiddenJVMFieldAndFieldType;
use crate::java_values::default_value_njv;

use crate::{NewJavaValue};

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
pub struct ObjectFields<'gc, 'l> {
    pub fields: HashMap<FieldNumber, NewJavaValue<'gc, 'l>>,
    pub hidden_fields: HashMap<FieldNumber, NewJavaValue<'gc, 'l>>,
}

impl<'gc, 'l> ObjectFields<'gc, 'l> {
    pub fn new_default_init_fields(object_layout: &ObjectLayout) -> Self {
        let hidden_fields = object_layout.hidden_field_numbers_reverse.iter().map(|(i, HiddenJVMFieldAndFieldType { cpdtype, .. })| {
            (*i, default_value_njv(cpdtype))
        }).collect::<HashMap<_, _>>();
        let fields = object_layout.field_numbers_reverse.iter().map(|(i, FieldNameAndFieldType { cpdtype, .. })| {
            (*i, default_value_njv(cpdtype))
        }).collect::<HashMap<_, _>>();
        Self{
            fields,
            hidden_fields
        }
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
