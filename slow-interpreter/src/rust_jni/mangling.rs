use std::sync::Arc;
use runtime_common::runtime_class::RuntimeClass;
use rust_jvm_common::utils::extract_string_from_utf8;
use rust_jvm_common::classnames::class_name;


pub fn mangle(classfile: Arc<RuntimeClass>, method_i: usize) -> String {
    let method = &classfile.classfile.methods[method_i];
    let name_index = method.name_index as usize;
    let method_name = extract_string_from_utf8(&classfile.classfile.constant_pool[name_index]);
//    let descriptor_index = method.descriptor_index as usize;
//    let descriptor_str = extract_string_from_utf8(&classfile.classfile.constant_pool[descriptor_index]);
    let class_name = class_name(&classfile.classfile).get_referred_name();
    //todo in the case of overloaded names this is incorrect
    format!("Java_{}_{}", escape(class_name),escape(method_name))
}


pub fn escape(s:String)-> String{
    let initial_replace = s.replace("_","_1").replace("/","_").replace(";","_2").replace("[","_3");
    //todo need to handle non-unicode but shouldn't be an issue for now.
    initial_replace
}