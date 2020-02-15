use rust_jvm_common::loading::LoaderArc;
use std::sync::Arc;
use runtime_common::InterpreterState;
use runtime_common::runtime_class::RuntimeClass;
use descriptor_parser::MethodDescriptor;

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
        let current: MethodDescriptor = MethodDescriptor::from(&m, &class.classfile);
        if current.parameter_types.iter().zip(descriptor.parameter_types.iter()).all(|(l, r)| l == r) &&
            current.return_type == descriptor.return_type && current.parameter_types.len() == descriptor.parameter_types.len() {
            return Some((*i, class.clone()));
        }
    }
    let super_class = state.initialized_classes.read().unwrap().get(&class.classfile.super_class_name().unwrap()).unwrap().clone();
    lookup_method_parsed_impl(state, super_class, name, descriptor, loader)
}

