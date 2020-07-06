use std::sync::Arc;

use regex::Regex;

use classfile_view::view::HasAccessFlags;

use crate::runtime_class::RuntimeClass;

pub fn mangle(classfile: Arc<RuntimeClass>, method_i: usize) -> String {
    let method = &classfile.view().method_view_i(method_i);
    let method_name = method.name();
    let class_name_ = classfile.view().name();
    let class_name = class_name_.get_referred_name();
    if classfile.view().lookup_method_name(&method_name).iter().filter(|m| {
        m.is_native()
    }).count() > 1 {
        let descriptor_str = method.desc_str();
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
        .replace("(", "")
        .replace(")", "")
        .replace("$", "_00024")
        .replace("/", "_");
    //todo need to handle unicode but shouldn't be an issue for now.
    initial_replace
}