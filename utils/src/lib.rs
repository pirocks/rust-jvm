use rust_jvm_common::classfile::{MethodInfo, Classfile};
use classfile_parser::types::MethodDescriptor;
use rust_jvm_common::loading::LoaderArc;

//todo the fact that I need a loader for this is dumb
pub fn lookup_method_parsed<'l>(class : &'l Classfile, name : String, descriptor : &MethodDescriptor,loader: &LoaderArc) -> Option<(usize, &'l MethodInfo)>{
    class.lookup_method_name(name).iter().filter(|(_i,m)|{
        let current: MethodDescriptor= MethodDescriptor::from(m,class,loader);
        current.parameter_types.iter().zip(descriptor.parameter_types.iter()).all(|(l,r)| l == r) &&
            current.return_type ==descriptor.return_type && current.parameter_types.len() == descriptor.parameter_types.len()
    }).nth(0).map(|x|x.clone())
}