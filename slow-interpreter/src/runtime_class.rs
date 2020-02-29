use std::sync::Arc;
use rust_jvm_common::classfile::{Classfile, ACC_FINAL};

use rust_jvm_common::classfile::ACC_STATIC;
use std::collections::HashMap;
use crate::InterpreterState;
use crate::run_function;
use std::rc::Rc;
use runtime_common::java_values::{default_value, JavaValue};
use runtime_common::runtime_class::RuntimeClass;
use std::cell::RefCell;
use runtime_common::StackEntry;
use crate::instructions::ldc::from_constant_pool_entry;
use classfile_view::view::ClassView;
use classfile_view::loading::LoaderArc;
use classfile_view::view::descriptor_parser::parse_field_descriptor;


pub fn prepare_class(classfile: Arc<Classfile>, loader: LoaderArc) -> RuntimeClass {
    let mut res = HashMap::new();
    for field in &classfile.fields {
        if (field.access_flags & ACC_STATIC) > 0 {
            let name = classfile.constant_pool[field.name_index as usize].extract_string_from_utf8();
            let field_descriptor_string = classfile.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
            let parsed = parse_field_descriptor(field_descriptor_string.as_str()).unwrap();//todo we should really have two pass parsing
            let val = default_value(parsed.field_type);
            res.insert(name, val);
        }
    }
    RuntimeClass {
        class_view: ClassView::from(classfile.clone()),
        classfile,
        loader,
        static_vars: RefCell::new(res),
    }
}

pub fn initialize_class(runtime_class: Arc<RuntimeClass>, state: &mut InterpreterState, stack: Option<Rc<StackEntry>>) -> Arc<RuntimeClass> {
    //todo make sure all superclasses are iniited first
    //todo make sure all interfaces are initted first
    //todo create a extract string which takes index. same for classname
    {
        let classfile = &runtime_class.classfile;
        for field in &classfile.fields {
            if (field.access_flags & ACC_STATIC > 0) && (field.access_flags & ACC_FINAL > 0) {
                //todo do I do this for non-static? Should I?
                let value_i = match field.constant_value_attribute_i() {
                    None => continue,
                    Some(i) => i,
                };
                let constant_pool = &classfile.constant_pool;
                let x = &constant_pool[value_i as usize];
                let constant_value = from_constant_pool_entry(constant_pool,x,state,stack.clone());
                let name = constant_pool[field.name_index as usize].extract_string_from_utf8();
                runtime_class.static_vars.borrow_mut().insert(name, constant_value);
            }
        }
    }
    //todo detecting if assertions are enabled?
    let class_arc = runtime_class;
    let classfile = &class_arc.classfile;
    let lookup_res = classfile.lookup_method_name(&"<clinit>".to_string());
    assert!(lookup_res.len() <= 1);
    let (clinit_i, clinit) = match lookup_res.iter().nth(0){
        None => return class_arc,
        Some(x) => x,
    };
    //todo should I really be manipulating the interpreter state like this

    let mut locals = vec![];
    let locals_n = clinit.code_attribute().unwrap().max_locals;
    for _ in 0..locals_n{
        locals.push(JavaValue::Top);
    }

    let new_stack = StackEntry {
        last_call_stack: stack,
        class_pointer: class_arc.clone(),
        method_i: *clinit_i as u16,
        local_vars: locals.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    };
    run_function(state, Rc::new(new_stack));
    if state.throw.is_some() || state.terminate {
        unimplemented!()
        //need to clear status after
    }
    if state.function_return {
        state.function_return = false;
        return class_arc;
    }
    panic!()
}

