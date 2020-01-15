use std::sync::Arc;
use rust_jvm_common::classfile::Classfile;
use rust_jvm_common::loading::Loader;
use crate::java_values::JavaValue;
use rust_jvm_common::classfile::ACC_STATIC;
use rust_jvm_common::utils::extract_string_from_utf8;
use std::collections::HashMap;
use crate::java_values::default_value;
use classfile_parser::types::parse_field_descriptor;
use rust_jvm_common::classfile::AttributeType;
use rust_jvm_common::classfile::FieldInfo;
use rust_jvm_common::classfile::CPIndex;
use crate::InterpreterState;
use crate::CallStackEntry;
use crate::run_function;

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

pub fn initialize_class(mut runtime_class: RuntimeClass, state: &mut InterpreterState) -> Arc<RuntimeClass>{
    //todo make sure all superclasses are iniited first
    //todo make sure all interfaces are initted first
    //todo create a extract string which takes index. same for classname
    let classfile = &runtime_class.classfile;
    for field in &classfile.fields {
        let value_i = constant_value_attribute_i(field);
        let x = &classfile.constant_pool[value_i as usize];
        let constant_value = JavaValue::from_constant_pool_entry(x);
        let name = extract_string_from_utf8(&classfile.constant_pool[field.name_index as usize]);
        runtime_class.static_vars.insert(name,constant_value);
    }
    //todo detecting if assertions are enabled?
    let (clinit_i,_) = runtime_class.classfile.methods.iter().enumerate().find(|(_,m)|{
        let name = extract_string_from_utf8(&classfile.constant_pool[m.name_index as usize]);
        name == "<clinit>"
    }).unwrap();
    //todo should I really be manipulating the interpreter state like this
    let class_arc = Arc::new(runtime_class);
    state.call_stack.push(CallStackEntry {
        class_pointer: class_arc.clone(),
        method_i: clinit_i as u16,
        local_vars: vec![],
        operand_stack: vec![],
        pc: 0,
        pc_offset: 0
    });
    run_function(state);
    if state.throw || state.terminate {
        unimplemented!()
        //need to clear status after
    }
    if state.function_return {
        state.function_return = false;
        return class_arc
    }
    panic!()
}

fn constant_value_attribute_i(field: &FieldInfo) -> CPIndex {
    for attr in &field.attributes {
        match &attr.attribute_type {
            AttributeType::ConstantValue(c) => {
                return c.constant_value_index
            },
            _ => {}
        }
    }
    panic!()
}
