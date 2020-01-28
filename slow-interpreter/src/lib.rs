#![feature(c_variadic)]

extern crate log;
extern crate simple_logger;
extern crate libc;
//extern crate va_list;

use std::sync::{Arc, RwLock};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::loading::Loader;
use std::error::Error;
use rust_jvm_common::utils::method_name;
use rust_jvm_common::utils::extract_string_from_utf8;
use classfile_parser::types::parse_method_descriptor;
use rust_jvm_common::unified_types::ParsedType;
use rust_jvm_common::unified_types::ArrayType;
use rust_jvm_common::unified_types::ClassWithLoader;
use crate::runtime_class::prepare_class;
use crate::interpreter_util::{run_function, check_inited_class};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use runtime_common::java_values::{JavaValue, Object};
use crate::interpreter_util::push_new_object;
use runtime_common::{InterpreterState, LibJavaLoading, StackEntry};
use rust_jvm_common::classfile::{Classfile, MethodInfo};


pub fn get_or_create_class_object(state: &mut InterpreterState,
                                  class_name: &ClassName,
                                  current_frame: Rc<StackEntry>,
                                  loader_arc: Arc<dyn Loader + Sync + Send>,
) -> Arc<Object> {
    //todo in future this may introduce new and exciting concurrency bugs

    let class_for_object = check_inited_class(state, class_name, current_frame.clone().into(), loader_arc);
    let res = state.class_object_pool.borrow().get(&class_for_object).cloned();
    match res {
        None => {
            let java_lang_class = ClassName::class();
            let java_lang_class_loader = ClassName::new("java/lang/ClassLoader");
            let current_loader = current_frame.class_pointer.loader.clone();
            let class_class = check_inited_class(state, &java_lang_class, current_frame.clone().into(), current_loader.clone());
            let class_loader_class = check_inited_class(state, &java_lang_class_loader, current_frame.clone().into(), current_loader.clone());
            //the above would only be required for higher jdks where a class loader obect is part of Class.
            //as it stands we can just push to operand stack
            push_new_object(current_frame.clone(), &class_class);
            let object = current_frame.pop();
            match object.clone() {
                JavaValue::Object(o) => {
                    let boostrap_loader_object = Object {
                        gc_reachable: true,
                        fields: RefCell::new(HashMap::new()),
                        class_pointer: class_loader_class.clone(),
                        bootstrap_loader: true,
                        object_class_object_pointer: RefCell::new(None),
                    };
                    let bootstrap_arc = Arc::new(boostrap_loader_object);
                    let bootstrap_class_loader = JavaValue::Object(bootstrap_arc.clone().into());
                    {
                        bootstrap_arc.fields.borrow_mut().insert("assertionLock".to_string(), bootstrap_class_loader.clone());//itself...
                        bootstrap_arc.fields.borrow_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
                        o.unwrap().fields.borrow_mut().insert("classLoader".to_string(), bootstrap_class_loader);
                    }
                }
                _ => panic!(),
            }
            let r = object.unwrap_object().unwrap();
            r.object_class_object_pointer.replace(Some(class_for_object.clone()));
            state.class_object_pool.borrow_mut().insert(class_for_object, r.clone());
            r
        }
        Some(r) => r.clone(),
    }
}

pub fn run(
    main_class_name: &ClassName,
    bl: Arc<dyn Loader + Send + Sync>,
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
        jni,
    };
    let system_class = check_inited_class(&mut state, &ClassName::new("java/lang/System"), None, bl.clone());
    let (init_system_class_i, _method_info) = locate_init_system_class(&system_class.classfile);
    let initialize_system_frame = StackEntry {
        last_call_stack: None,
        class_pointer: system_class.clone(),
        method_i: init_system_class_i as u16,
        local_vars: RefCell::new(vec![]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(-1),
    };

    run_function(&mut state, initialize_system_frame.into());
    let main_stack = StackEntry {
        last_call_stack: None,
        class_pointer: Arc::new(main_class),
        method_i: main_i as u16,
//            todo is that vec access safe, or does it not heap allocate?
        local_vars: vec![JavaValue::Array(Some(Arc::new(vec![].into())))].into(),//todo handle parameters
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
    system.methods.iter().enumerate().find(|(_, method)| {
        let name = method_name(system, method);
        name == "initializeSystemClass".to_string()
    }).unwrap()
}

fn locate_main_method(bl: &Arc<dyn Loader + Send + Sync>, main: &Arc<Classfile>) -> usize {
    main.methods.iter().enumerate().find(|(_, method)| {
        let name = method_name(main, method);
        if name == "main".to_string() {
            let descriptor_string = extract_string_from_utf8(&main.constant_pool[method.descriptor_index as usize]);
            let descriptor = parse_method_descriptor(&bl, descriptor_string.as_str()).unwrap();
            let string_name = ClassName::string();
            let string_class = ParsedType::Class(ClassWithLoader { class_name: string_name, loader: bl.clone() });
            let string_array = ParsedType::ArrayReferenceType(ArrayType { sub_type: Box::new(string_class) });
            descriptor.parameter_types.len() == 1 &&
                descriptor.return_type == ParsedType::VoidType &&
                descriptor.parameter_types.iter().zip(vec![string_array]).all(|(a, b)| a == &b)
        } else {
            false
        }
    }).unwrap().0
}

pub mod instructions;
pub mod interpreter_util;
pub mod runtime_class;
pub mod native;
pub mod rust_jni;