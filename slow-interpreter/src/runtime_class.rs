use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::fmt::{Formatter, Debug, Error};
use crate::java_values::{JavaValue, default_value};
use rust_jvm_common::classfile::{Classfile, ACC_FINAL, ACC_STATIC};
use std::hash::{Hash, Hasher};
use classfile_view::view::ClassView;
use classfile_view::loading::LoaderArc;

use crate::{StackEntry, JVMState};
use crate::instructions::ldc::from_constant_pool_entry;
use descriptor_parser::parse_field_descriptor;
use classfile_view::view::ptype_view::PTypeView;
use crate::interpreter::run_function;

#[derive(Debug, PartialEq, Hash)]
pub enum RuntimeClass{
    Int,
    Array(RuntimeClassArray),
    Object(RuntimeClassObject)
}

#[derive(Debug)]
pub struct RuntimeClassArray{

}


pub struct RuntimeClassObject {
    pub classfile: Arc<Classfile>,
    pub class_view: ClassView,
    pub loader: LoaderArc,
    pub static_vars: RwLock<HashMap<String, JavaValue>>,
}

impl Debug for RuntimeClassObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}:{:?}", self.class_view.name(), self.static_vars)
    }
}

impl Hash for RuntimeClassObject {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.classfile.hash(state);
        self.loader.name().to_string().hash(state)
    }
}


impl PartialEq for RuntimeClassObject {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.loader, &other.loader) && self.classfile == other.classfile && *self.static_vars.read().unwrap() == *other.static_vars.read().unwrap()
    }
}

impl Eq for RuntimeClass {}


pub fn prepare_class(classfile: Arc<Classfile>, loader: LoaderArc) -> RuntimeClass {
    let mut res = HashMap::new();
    for field in &classfile.fields {
        if (field.access_flags & ACC_STATIC) > 0 {
            let name = classfile.constant_pool[field.name_index as usize].extract_string_from_utf8();
            let field_descriptor_string = classfile.constant_pool[field.descriptor_index as usize].extract_string_from_utf8();
            let parsed = parse_field_descriptor(field_descriptor_string.as_str()).unwrap();//todo we should really have two pass parsing
            let val = default_value(PTypeView::from_ptype(&parsed.field_type));
            res.insert(name, val);
        }
    }
    RuntimeClassObject {
        class_view: ClassView::from(classfile.clone()),
        classfile,
        loader,
        static_vars: RwLock::new(res),
    }.into()
}

pub fn initialize_class(
    runtime_class: Arc<RuntimeClass>,
    jvm: &JVMState,
) -> Arc<RuntimeClass> {
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
                let constant_value = from_constant_pool_entry(constant_pool, x, jvm);
                let name = constant_pool[field.name_index as usize].extract_string_from_utf8();
                runtime_class.static_vars.write().unwrap().insert(name, constant_value);
            }
        }
    }
    //todo detecting if assertions are enabled?
    let class_arc = runtime_class;
    let classfile = &class_arc.classfile;
    let lookup_res = classfile.lookup_method_name(&"<clinit>".to_string());
    assert!(lookup_res.len() <= 1);
    let (clinit_i, clinit) = match lookup_res.iter().nth(0) {
        None => return class_arc,
        Some(x) => x,
    };
    //todo should I really be manipulating the interpreter state like this

    let mut locals = vec![];
    let locals_n = clinit.code_attribute().unwrap().max_locals;
    for _ in 0..locals_n {
        locals.push(JavaValue::Top);
    }

    let new_stack = StackEntry {
        class_pointer: class_arc.clone(),
        method_i: *clinit_i as u16,
        local_vars: locals.into(),
        operand_stack: vec![].into(),
        pc: 0.into(),
        pc_offset: 0.into(),
    }.into();
    jvm.get_current_thread().call_stack.borrow_mut().push(new_stack);
    run_function(jvm);
    jvm.get_current_thread().call_stack.borrow_mut().pop();
    if jvm.get_current_thread().interpreter_state.throw.borrow().is_some() || *jvm.get_current_thread().interpreter_state.terminate.borrow() {
        unimplemented!()
        //need to clear status after
    }
    if *jvm.get_current_thread().interpreter_state.function_return.borrow() {
        *jvm.get_current_thread().interpreter_state.function_return.borrow_mut() = false;
        return class_arc;
    }
    panic!()
}

