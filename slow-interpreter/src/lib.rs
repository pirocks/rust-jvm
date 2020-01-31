#![feature(c_variadic)]

extern crate log;
extern crate simple_logger;
extern crate libc;
extern crate regex;
//extern crate va_list;

use std::sync::{Arc, RwLock};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::LoaderArc;
use std::error::Error;
use classfile_parser::types::{MethodDescriptor, parse_field_type};
use rust_jvm_common::unified_types::ParsedType;
use rust_jvm_common::unified_types::ArrayType;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::runtime_class::prepare_class;
use crate::interpreter_util::{run_function, check_inited_class};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use runtime_common::java_values::{JavaValue, Object, NormalObject};
use crate::interpreter_util::push_new_object;
use runtime_common::{InterpreterState, LibJavaLoading, StackEntry};
use rust_jvm_common::classfile::{Classfile, MethodInfo};


pub fn get_or_create_class_object(state: &mut InterpreterState,
                                  class_name: &ClassName,
                                  current_frame: Rc<StackEntry>,
                                  loader_arc: LoaderArc,
) -> Arc<Object> {
    //todo in future this may introduce new and exciting concurrency bugs
    if class_name.get_referred_name().starts_with('[') {
        array_object(state, class_name, current_frame, loader_arc)
    } else {
        regular_object(state, class_name, current_frame, loader_arc)
    }
}

fn array_object(state: &mut InterpreterState, class_name: &ClassName, current_frame: Rc<StackEntry>, loader_arc: LoaderArc) -> Arc<Object> {
    let referred_class_name = class_name.get_referred_name();
    let after_parse = parse_field_type(&loader_arc, referred_class_name.as_str()).unwrap();
    assert!(after_parse.0.is_empty());
    let type_for_object : ParsedType = after_parse.1.unwrap_array_type();
    let res = state.array_object_pool.borrow().get(&type_for_object).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, current_frame);
            r.unwrap_normal_object().array_class_object_pointer.replace(type_for_object.clone().into());
            state.array_object_pool.borrow_mut().insert(type_for_object.clone(), r.clone());
            r
        },
        Some(r) => r.clone(),
    }
}

fn regular_object(state: &mut InterpreterState, class_name: &ClassName, current_frame: Rc<StackEntry>, loader_arc: LoaderArc) -> Arc<Object> {
    let class_for_object = check_inited_class(state, class_name, current_frame.clone().into(), loader_arc);
    let res = state.class_object_pool.borrow().get(&class_for_object).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, current_frame);
            r.unwrap_normal_object().object_class_object_pointer.replace(Some(class_for_object.clone()));
            state.class_object_pool.borrow_mut().insert(class_for_object, r.clone());
            r
        },
        Some(r) => r.clone(),
    }
}

fn create_a_class_object(state: &mut InterpreterState, current_frame: Rc<StackEntry>) -> Arc<Object> {
    let java_lang_class = ClassName::class();
    let java_lang_class_loader = ClassName::new("java/lang/ClassLoader");
    let current_loader = current_frame.class_pointer.loader.clone();
    let class_class = check_inited_class(state, &java_lang_class, current_frame.clone().into(), current_loader.clone());
    let class_loader_class = check_inited_class(state, &java_lang_class_loader, current_frame.clone().into(), current_loader.clone());
    //the above would only be required for higher jdks where a class loader object is part of Class.
    //as it stands we can just push to operand stack
    push_new_object(current_frame.clone(), &class_class);
    let object = current_frame.pop();
    match object.clone() {
        JavaValue::Object(o) => {
            let boostrap_loader_object = NormalObject {
                gc_reachable: true,
                fields: RefCell::new(HashMap::new()),
                class_pointer: class_loader_class.clone(),
                bootstrap_loader: true,
                object_class_object_pointer: RefCell::new(None),
                array_class_object_pointer: RefCell::new(None)
            };
            let bootstrap_arc = Arc::new(Object::Object(boostrap_loader_object));
            let bootstrap_class_loader = JavaValue::Object(bootstrap_arc.clone().into());
            {
                bootstrap_arc.unwrap_normal_object().fields.borrow_mut().insert("assertionLock".to_string(), bootstrap_class_loader.clone());//itself...
                bootstrap_arc.unwrap_normal_object().fields.borrow_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
//                o.unwrap().unwrap_object().fields.borrow_mut().insert("classLoader".to_string(), bootstrap_class_loader);
                o.unwrap().unwrap_normal_object().fields.borrow_mut().insert("classLoader".to_string(), JavaValue::Object(None));
            }
        }
        _ => panic!(),
    }
    let r = object.unwrap_object().unwrap();
    r
}

pub fn run(
    main_class_name: &ClassName,
    bl: LoaderArc,
    _args: Vec<String>,
    jni: LibJavaLoading,
) -> Result<(), Box<dyn Error>> {
    let main = bl.clone().load_class(bl.clone(), main_class_name, bl.clone())?;
    let main_class = prepare_class(main.clone(), bl.clone());
    let main_i = locate_main_method(&bl, &main);
    let mut state = InterpreterState {
        terminate: false,
        throw: false,
        function_return: false,
        bootstrap_loader: bl.clone(),
        initialized_classes: RwLock::new(HashMap::new()),
        string_internment: RefCell::new(HashMap::new()),
        class_object_pool: RefCell::new(HashMap::new()),
        array_object_pool: RefCell::new(HashMap::new()),
        jni,
    };
    let system_class = check_inited_class(&mut state, &ClassName::new("java/lang/System"), None, bl.clone());
    let (init_system_class_i, method_info) = locate_init_system_class(&system_class.classfile);
    let mut locals = vec![];
    for _ in 0..method_info.code_attribute().unwrap().max_locals{
        locals.push(JavaValue::Top);
    }
    let initialize_system_frame = StackEntry {
        last_call_stack: None,
        class_pointer: system_class.clone(),
        method_i: init_system_class_i as u16,
        local_vars: RefCell::new(locals),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(-1),
    };

    run_function(&mut state, initialize_system_frame.into());
//    let array_elem_type = ParsedType::Class(ClassWithLoader { class_name: ClassName::string(), loader: bl.clone() });
    //todo use array_elem_type
    let main_stack = StackEntry {
        last_call_stack: None,
        class_pointer: Arc::new(main_class),
        method_i: main_i as u16,
//            todo is that vec access safe, or does it not heap allocate?
        local_vars: vec![].into(),//todo handle parameters, todo handle non-zero size locals
        operand_stack: vec![].into(),
        pc: RefCell::new(0),
        pc_offset: 0.into(),
    };
    run_function(&mut state, Rc::new(main_stack));
    if state.throw || state.terminate {
        unimplemented!()
    }
    Result::Ok(())
}

fn locate_init_system_class(system: &Arc<Classfile>) -> (usize, &MethodInfo) {
    system.lookup_method_name(&"initializeSystemClass".to_string()).iter().nth(0).unwrap().clone()
}

fn locate_main_method(bl: &LoaderArc, main: &Arc<Classfile>) -> usize {
    let string_name = ClassName::string();
    let string_class = ParsedType::Class(ClassWithLoader { class_name: string_name, loader: bl.clone() });
    let string_array = ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(string_class) });
    let psvms = main.lookup_method_name(&"main".to_string());
    for (i, m) in psvms {
        let desc = MethodDescriptor::from(m, main, bl);
        if m.is_static() && desc.parameter_types == vec![string_array.clone()] && desc.return_type == ParsedType::VoidType {
            return i;
        }
    }
    panic!();
}

pub mod instructions;
pub mod interpreter_util;
pub mod runtime_class;
pub mod rust_jni;