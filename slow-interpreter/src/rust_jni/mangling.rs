use regex::Regex;

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::compressed_classfile::CompressedClassfileStringPool;

pub fn mangle(pool: &CompressedClassfileStringPool, method: &MethodView) -> String {
    let method_name = method.name();
    let class_view = method.classview();
    let class_name = class_view.type_().class_name_representation().replace(".", "/");
    let multiple_same_name_methods = class_view.lookup_method_name(method_name).iter().filter(|m| {
        m.is_native()
    }).count() > 1;
    if multiple_same_name_methods {
        let descriptor_str = method.desc_str();
        let rg = Regex::new(r"\(([A-Za-z/;]*)\)").unwrap();
        let extracted_descriptor = rg.captures(descriptor_str.as_str()).unwrap().get(1).unwrap().as_str().to_string();
        format!("Java_{}_{}__{}", escape(&class_name), escape(&method_name.0.to_str(pool)), escape(&extracted_descriptor))
    } else {
        format!("Java_{}_{}", escape(&class_name), escape(&method_name.0.to_str(pool)))
    }
}


pub fn escape(s: &String) -> String {
    //todo need to handle unicode but shouldn't be an issue for now.
    s
        .replace("_", "_1")
        .replace(";", "_2")
        .replace("[", "_3")
        .replace("(", "")
        .replace(")", "")
        .replace("$", "_00024")
        .replace("/", "_")
}