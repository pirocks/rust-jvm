#![feature(c_variadic)]

extern crate log;
extern crate simple_logger;
extern crate libc;
extern crate regex;
extern crate va_list;

use std::sync::{Arc, RwLock};
use rust_jvm_common::classnames::ClassName;

use std::error::Error;
use rust_jvm_common::ptype::PType;
use crate::runtime_class::prepare_class;
use crate::interpreter_util::{run_function, check_inited_class};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;
use runtime_common::java_values::{JavaValue, Object, NormalObject};
use crate::interpreter_util::push_new_object;
use runtime_common::{InterpreterState, LibJavaLoading, StackEntry};
use rust_jvm_common::classfile::{Classfile, MethodInfo};
use rust_jvm_common::string_pool::StringPool;

use std::ops::Deref;
use std::time::Instant;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use classfile_view::loading::LoaderArc;
use classfile_view::view::descriptor_parser::MethodDescriptor;
use crate::instructions::ldc::create_string_on_stack;

pub fn get_or_create_class_object(state: &mut InterpreterState,
                                  type_: &ReferenceTypeView,
                                  current_frame: Rc<StackEntry>,
                                  loader_arc: LoaderArc,
                                  primitive: Option<String>,
) -> Arc<Object> {
    match type_ {
        ReferenceTypeView::Class(class_name) => {
            regular_object(state, class_name, current_frame, loader_arc, primitive)
        }
        ReferenceTypeView::Array(c) => {
            assert!(primitive.is_none());
            array_object(state, c.deref(), current_frame)
        }
    }
}

fn array_object(state: &mut InterpreterState, array_sub_type: &PTypeView, current_frame: Rc<StackEntry>) -> Arc<Object> {
    let type_for_object: PType = array_sub_type.to_ptype();
    array_of_type_class(state, current_frame, &type_for_object)
}

pub fn array_of_type_class(state: &mut InterpreterState, current_frame: Rc<StackEntry>, type_for_object: &PType) -> Arc<Object> {
    let res = state.array_object_pool.borrow().get(&type_for_object).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, current_frame);
            let array_ptype_view = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::from_ptype(type_for_object).into())).into();
            r.unwrap_normal_object().class_object_ptype.replace(array_ptype_view);
            state.array_object_pool.borrow_mut().insert(type_for_object.clone(), r.clone());
            r
        }
        Some(r) => r.clone(),
    }
}

fn regular_object(state: &mut InterpreterState, class_name: &ClassName, current_frame: Rc<StackEntry>, loader_arc: LoaderArc, primitive: Option<String>) -> Arc<Object> {
    let class_for_object = check_inited_class(state, class_name, current_frame.clone().into(), loader_arc);
    let res = if primitive.is_some() {
        state.primitive_object_pool.borrow().get(&class_for_object).cloned()
    } else {
        state.class_object_pool.borrow().get(&class_for_object).cloned()
    };
    match res {
        None => {
            let r = create_a_class_object(state, current_frame.clone());
            r.unwrap_normal_object().class_object_ptype.replace(Some(PTypeView::Ref(ReferenceTypeView::Class(class_for_object.class_view.name()))));
            state.class_object_pool.borrow_mut().insert(class_for_object, r.clone());
            if primitive.is_some(){
                create_string_on_stack(state,&current_frame,primitive.unwrap());
                r.unwrap_normal_object().fields.borrow_mut().insert("name".to_string(), current_frame.pop());
            }
            r
        }
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
//            assert_eq!(&class_loader_class.classfile.access_flags & ACC_ABSTRACT, 0);
            let boostrap_loader_object = NormalObject {
                gc_reachable: true,
                fields: RefCell::new(HashMap::new()),
                class_pointer: class_loader_class.clone(),
                bootstrap_loader: true,
                // object_class_object_pointer: RefCell::new(None),
                // array_class_object_pointer: RefCell::new(None),
                class_object_ptype: RefCell::new(None)
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
    let r = object.unwrap_object_nonnull();
    r
}

pub fn run(
    main_class_name: &ClassName,
    bl: LoaderArc,
    _args: Vec<String>,
    jni: LibJavaLoading,
) -> Result<(), Box<dyn Error>> {
    let main = bl.clone().load_class(bl.clone(), main_class_name, bl.clone())?;
    let main_class = prepare_class(main.clone().backing_class(), bl.clone());
    let main_i = locate_main_method(&bl, &main.backing_class());
    let mut state = InterpreterState {
        terminate: false,
        throw: None,
        function_return: false,
        bootstrap_loader: bl.clone(),
        initialized_classes: RwLock::new(HashMap::new()),
        string_internment: RefCell::new(HashMap::new()),
        class_object_pool: RefCell::new(HashMap::new()),
        primitive_object_pool: RefCell::new(HashMap::new()),
        array_object_pool: RefCell::new(HashMap::new()),
        jni,
        string_pool: StringPool {
            entries: HashSet::new()
        },
        start_instant: Instant::now(),
    };
    let system_class = check_inited_class(&mut state, &ClassName::new("java/lang/System"), None, bl.clone());
    let (init_system_class_i, method_info) = locate_init_system_class(&system_class.classfile);
    let mut locals = vec![];
    for _ in 0..method_info.code_attribute().unwrap().max_locals {
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
    if state.function_return {
        state.function_return = false;
    }
    if state.throw.is_some() || state.terminate {
        unimplemented!()
    }
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
    if state.throw.is_some() || state.terminate {
        unimplemented!()
    }
    Result::Ok(())
}

fn locate_init_system_class(system: &Arc<Classfile>) -> (usize, &MethodInfo) {
    system.lookup_method_name(&"initializeSystemClass".to_string()).iter().nth(0).unwrap().clone()
}

fn locate_main_method(_bl: &LoaderArc, main: &Arc<Classfile>) -> usize {
    let string_name = ClassName::string();
    let string_class = PTypeView::Ref(ReferenceTypeView::Class(string_name));
    let string_array = PTypeView::Ref(ReferenceTypeView::Array(string_class.into()));
    let psvms = main.lookup_method_name(&"main".to_string());
    for (i, m) in psvms {
        let desc = MethodDescriptor::from_legacy(m, main);
        if m.is_static() && desc.parameter_types == vec![string_array.clone()] && desc.return_type == PTypeView::VoidType {
            return i;
        }
    }
    panic!();
}

pub mod instructions;
pub mod interpreter_util;
pub mod runtime_class;
pub mod rust_jni;