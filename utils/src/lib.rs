use rust_jvm_common::classfile::{MethodInfo, Classfile};
use classfile_parser::types::MethodDescriptor;
use rust_jvm_common::loading::LoaderArc;

//todo the fact that I need a loader for this is dumb
pub fn lookup_method_parsed<'l>(class: &'l Classfile, name: String, descriptor: &MethodDescriptor, loader: &LoaderArc) -> Option<(usize, &'l MethodInfo)> {
    let res = lookup_method_parsed_impl(class,name,descriptor,loader);
    match res {
        None => None,
        Some(i) => Some((i,&class.methods[i])),
    }
}

pub fn lookup_method_parsed_impl(class: &Classfile, name: String, descriptor: &MethodDescriptor, loader: &LoaderArc) -> Option<usize> {
    for (i, m) in &class.lookup_method_name(name.clone()) {
        let current: MethodDescriptor = MethodDescriptor::from(&m, &class, loader);
        if current.parameter_types.iter().zip(descriptor.parameter_types.iter()).all(|(l, r)| l == r) &&
            current.return_type == descriptor.return_type && current.parameter_types.len() == descriptor.parameter_types.len() {
            return Some(*i);
        }
    }
    let super_class = loader.load_class(loader.clone(), &class.super_class_name(), loader.clone()).unwrap();//todo what am I going to do about bl
    lookup_method_parsed_impl(&super_class, name, descriptor, loader)
}

