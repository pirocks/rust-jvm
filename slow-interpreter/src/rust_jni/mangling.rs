use std::sync::Arc;
use rust_jvm_common::classnames::class_name;
use regex::Regex;
use rust_jvm_common::classfile::ACC_NATIVE;
use crate::runtime_class::RuntimeClass;

pub fn mangle(classfile: Arc<RuntimeClass>, method_i: usize) -> String {
    let method = &classfile.classfile.methods[method_i];
    let method_name = method.method_name(&classfile.classfile);
    let class_name_ = class_name(&classfile.classfile);
    let class_name = class_name_.get_referred_name();
    if classfile.classfile.lookup_method_name(&method_name).iter().filter(|(_i,m)|{
        m.access_flags & ACC_NATIVE > 0
    }).count() > 1 {
        let descriptor_str = method.descriptor_str(&classfile.classfile);
        let rg = Regex::new(r"\(([A-Za-z/;]*)\)").unwrap();
        let extracted_descriptor = rg.captures(descriptor_str.as_str()).unwrap().get(1).unwrap().as_str().to_string();
        format!("Java_{}_{}__{}", escape(class_name), escape(&method_name), escape(&extracted_descriptor))
    } else {
        format!("Java_{}_{}", escape(class_name), escape(&method_name))
    }
}


pub fn escape(s: &String) -> String {
    let initial_replace = s
        .replace("_", "_1")
        .replace(";", "_2")
        .replace("[", "_3")
        .replace("(","")
        .replace(")","")
        .replace("$","_00024")
        .replace("/", "_");
    //todo need to handle unicode but shouldn't be an issue for now.
    initial_replace
}