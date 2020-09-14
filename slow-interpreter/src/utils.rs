use std::ops::Deref;
use std::sync::Arc;

use classfile_view::loading::LoaderArc;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;

use crate::java_values::Object;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;

//todo the fact that I need a loader for this is dumb
pub fn lookup_method_parsed(state: &JVMState, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor, loader: &LoaderArc) -> Option<(usize, Arc<RuntimeClass>)> {
    let res = lookup_method_parsed_impl(state, class, name, descriptor, loader);
    match res {
        None => None,
        Some((i, c)) => {
            Some((i, c))
        }
    }
}

pub fn lookup_method_parsed_impl(state: &JVMState, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor, loader: &LoaderArc) -> Option<(usize, Arc<RuntimeClass>)> {
    match class.view().lookup_method(&name, &descriptor) {
        None => {
            let class_name = class.view().super_name().unwrap();
            let lookup_type = PTypeView::Ref(ReferenceTypeView::Class(class_name));
            let super_class = state.classes.initialized_classes.read().unwrap().get(&lookup_type).unwrap().clone(); //todo this unwrap could fail, and this should really be using check_inited_class
            lookup_method_parsed_impl(state, super_class, name, descriptor, loader)
        }
        Some(method_view) => {
            Some((method_view.method_i(), class.clone()))
        }
    }
}


//todo make this an impl method on Object
pub fn string_obj_to_string(str_obj: Option<Arc<Object>>) -> String {
    let temp = str_obj.unwrap().lookup_field("value");
    let chars = temp.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    let mut res = String::new();
    for char_ in borrowed_elems.deref() {
        res.push(char_.unwrap_char() as u8 as char);
    }
    res
}