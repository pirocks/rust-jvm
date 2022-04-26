use std::collections::HashMap;
use std::sync::Arc;
use itertools::Itertools;


use classfile_view::view::{ClassView};
use rust_jvm_common::method_shape::{MethodShape, ShapeOrderWrapperOwned};

use crate::RuntimeClass;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct MethodNumber(pub u32);


//todo this won't handle private methods correctly but can't b/c method shape doesn't know about multiple private methods
pub struct MethodNumberMappings {
    pub current_method_number: u32,
    pub mapping: HashMap<MethodShape, MethodNumber>,
}

impl MethodNumberMappings {
    pub fn new() -> Self {
        Self {
            current_method_number: 0,
            mapping: HashMap::new(),
        }
    }

    pub fn sink_method(&mut self, shape: MethodShape) {
        if self.mapping.contains_key(&shape) {
            return;
        } else {
            let this_method_number = self.current_method_number;
            self.current_method_number += 1;
            self.mapping.insert(shape, MethodNumber(this_method_number));
        }
    }
}

// method number order
// object class vtable
// subclass:
//  interfaces in iteration order, methods sorted by shape order
//  subclass methods in method shape order
fn get_method_numbers_recurse<'gc>(class_view: &Arc<dyn ClassView>, parent: &Option<Arc<RuntimeClass<'gc>>>, interfaces: &[Arc<RuntimeClass<'gc>>], method_number_mappings: &mut MethodNumberMappings) {
    if let Some(parent) = parent.as_ref() {
        let class_class = parent.unwrap_class_class();
        get_method_numbers_recurse(&class_class.class_view, &class_class.parent, class_class.interfaces.as_slice(), method_number_mappings);
    }

    for interface in interfaces.iter() {
        let class_class = interface.unwrap_class_class();
        get_method_numbers_recurse(&class_class.class_view, &None, class_class.interfaces.as_slice(), method_number_mappings);
    }

    for method_shape in class_view.methods().map(|method|ShapeOrderWrapperOwned(method.method_shape())).sorted() {
        method_number_mappings.sink_method(method_shape.0);
    }
}

// returns all method numbers applicable for this class and super clasess and super interfaces
pub fn get_method_numbers<'gc>(class_view: &Arc<dyn ClassView>, parent: &Option<Arc<RuntimeClass<'gc>>>, interfaces: &[Arc<RuntimeClass<'gc>>]) -> (u32, HashMap<MethodShape, MethodNumber>) {
    let mut method_number_mappings = MethodNumberMappings::new();
    get_method_numbers_recurse(&class_view, parent, interfaces, &mut method_number_mappings);
    (method_number_mappings.current_method_number, method_number_mappings.mapping)
}
