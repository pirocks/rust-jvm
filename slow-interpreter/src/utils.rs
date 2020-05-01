use std::sync::Arc;
use classfile_view::loading::LoaderArc;
use crate::JVMState;
use crate::runtime_class::RuntimeClass;
use crate::java_values::Object;
use std::ops::Deref;
use descriptor_parser::MethodDescriptor;


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
    match class.view().method_index().lookup(&name, &descriptor) {
        None => {
            let super_class = state.initialized_classes.read().unwrap().get(&class.view().super_name().unwrap()).unwrap().clone();
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
        res.push(char_.unwrap_char());
    }
    res
}