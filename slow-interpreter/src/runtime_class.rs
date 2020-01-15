use std::sync::Arc;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::Loader;
use crate::java_values::JavaValue;
use rust_jvm_common::classfile::ACC_STATIC;
use rust_jvm_common::utils::extract_string_from_utf8;
use std::collections::HashMap;
use crate::java_values::default_value;
use classfile_parser::types::parse_field_descriptor;

pub struct RuntimeClass{
    pub classfile: Arc<Classfile>,
    pub loader: Arc<dyn Loader + Send + Sync>,
    pub static_vars: HashMap<String,JavaValue>
}


pub fn prepare_class(classfile: Arc<Classfile>, loader: Arc<dyn Loader + Send + Sync>) -> RuntimeClass{
    let mut res = HashMap::new();
    for field in &classfile.fields {
        if (field.access_flags & ACC_STATIC) > 0{
            let name = extract_string_from_utf8(&classfile.constant_pool[field.name_index as usize]);
            let field_descriptor_string = extract_string_from_utf8(&classfile.constant_pool[field.descriptor_index as usize]);
            let parsed = parse_field_descriptor(&loader,field_descriptor_string.as_str()).unwrap();//todo we should really have two pass parsing
            let val = default_value(parsed.field_type);
            res.insert(name,val);
        }
    }
    RuntimeClass {
        classfile,
        loader,
        static_vars:res,

    }

}
