use regex::{Regex};

use classfile_view::view::{ClassView, HasAccessFlags};
use classfile_view::view::method_view::MethodView;
use rust_jvm_common::compressed_classfile::compressed_descriptors::mangling_escape;
use rust_jvm_common::compressed_classfile::string_pool::CompressedClassfileStringPool;


pub struct ManglingRegex {
    rg: Regex,
}

impl ManglingRegex {
    pub fn new() -> Self {
        Self {
            rg: Regex::new(r"\(([A-Za-z/;]*)\)").unwrap()
        }
    }
}

pub fn mangle(mangling_regex: &ManglingRegex, pool: &CompressedClassfileStringPool, method: &MethodView) -> String {
    let method_name = method.name();
    let class_view = method.classview();
    let class_name = class_view
        .type_()
        .unwrap_class_type()
        .0
        .to_str(pool) /*.class_name_representation()*/
        .replace(".", "/");
    let multiple_same_name_methods = class_view.lookup_method_name(method_name).iter().filter(|m| m.is_native()).count() > 1;
    let res = if multiple_same_name_methods {
        let descriptor_str = method.desc_str();
        let desc = method.desc();
        //todo replace regex with using parsed desc
        let rg = &mangling_regex.rg;
        let extracted_descriptor = rg.captures(descriptor_str.to_str(pool).as_str()).unwrap().get(1).unwrap().as_str().to_string();
        format!("Java_{}_{}__{}", mangling_escape(&class_name), mangling_escape(&method_name.0.to_str(pool)), /*desc.mangled_representation(pool)*/mangling_escape(&extracted_descriptor))
    } else {
        format!("Java_{}_{}", mangling_escape(&class_name), mangling_escape(&method_name.0.to_str(pool)))
    };

    res
}