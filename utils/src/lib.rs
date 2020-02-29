
use std::sync::Arc;
use runtime_common::InterpreterState;
use runtime_common::runtime_class::RuntimeClass;
use classfile_view::view::descriptor_parser::MethodDescriptor;
use classfile_view::loading::LoaderArc;
use runtime_common::java_values::Object;
use std::ops::Deref;


//todo the fact that I need a loader for this is dumb
pub fn lookup_method_parsed(state: &mut InterpreterState, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor, loader: &LoaderArc) -> Option<(usize, Arc<RuntimeClass>)> {
    let res = lookup_method_parsed_impl(state, class, name, descriptor, loader);
    match res {
        None => None,
        Some((i, c)) => {
            Some((i, c))
        }
    }
}

pub fn lookup_method_parsed_impl(state: &mut InterpreterState, class: Arc<RuntimeClass>, name: String, descriptor: &MethodDescriptor, loader: &LoaderArc) -> Option<(usize, Arc<RuntimeClass>)> {
    for (i, m) in &class.classfile.lookup_method_name(&name) {
        let current: MethodDescriptor = MethodDescriptor::from_legacy(&m, &class.classfile);
        if current.parameter_types.iter().zip(descriptor.parameter_types.iter()).all(|(l, r)| l == r) &&
            current.return_type == descriptor.return_type && current.parameter_types.len() == descriptor.parameter_types.len() {
            return Some((*i, class.clone()));
        }
    }
    let super_class = state.initialized_classes.read().unwrap().get(&class.classfile.super_class_name().unwrap()).unwrap().clone();
    lookup_method_parsed_impl(state, super_class, name, descriptor, loader)
}


pub fn string_obj_to_string(str_obj: Option<Arc<Object>>) -> String {
    let temp = str_obj.unwrap();
    let str_fields = temp.unwrap_normal_object().fields.borrow();
    let char_object = str_fields.get("value").unwrap().unwrap_object().unwrap();
    let chars = char_object.unwrap_array();
    let borrowed_elems = chars.elems.borrow();
    let mut res = String::new();
    for char_ in borrowed_elems.deref() {
        res.push(char_.unwrap_char());
    }
    res
}