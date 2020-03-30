#![feature(c_variadic)]
extern crate log;
extern crate simple_logger;
extern crate libloading;
extern crate libc;
extern crate regex;
extern crate va_list;

use rust_jvm_common::classnames::{ClassName, class_name};
use rust_jvm_common::string_pool::StringPool;
use rust_jvm_common::ptype::PType;
use rust_jvm_common::classfile::{Classfile, MethodInfo, CPIndex};
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue, NormalObject};
use crate::runtime_class::prepare_class;
use crate::interpreter_util::{run_function, check_inited_class};
use crate::interpreter_util::push_new_object;
use crate::instructions::ldc::create_string_on_stack;
use libloading::Library;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use classfile_view::loading::{LoaderArc, LivePoolGetter};
use descriptor_parser::MethodDescriptor;
use std::sync::{Arc, RwLock};
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Instant;


pub mod java_values;
pub mod runtime_class;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
pub mod utils;

pub struct InterpreterState {
    pub terminate: bool,
    pub throw: Option<Arc<Object>>,
    pub function_return: bool,
    pub bootstrap_loader: LoaderArc,
    pub initialized_classes: RwLock<HashMap<ClassName, Arc<RuntimeClass>>>,
    pub string_internment: RefCell<HashMap<String, Arc<Object>>>,


    pub class_object_pool: RefCell<HashMap<PTypeView, Arc<Object>>>,
    // pub class_loader : Arc<Object>,
    pub system_domain_loader : bool,

    //todo needs to be used for all instances of getClass
    pub jni: LibJavaLoading,
    pub string_pool: StringPool,
    //todo this should really be in some sort of parser/jvm state
    pub start_instant: Instant,

    //anon classes
    pub anon_class_counter: usize,
    pub anon_class_live_object_ldc_pool : Arc<RefCell<Vec<Arc<Object>>>>,

    pub debug_exclude: bool
}

struct LivePoolGetterImpl{
    anon_class_live_object_ldc_pool : Arc<RefCell<Vec<Arc<Object>>>>
}

impl LivePoolGetter for LivePoolGetterImpl{
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.borrow()[idx];
        ReferenceTypeView::Class(object.unwrap_normal_object().class_pointer.class_view.name())//todo handle arrays
    }
}

pub struct NoopLivePoolGetter{}

impl LivePoolGetter for NoopLivePoolGetter{
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        panic!()
    }
}


impl InterpreterState{
    pub fn get_live_object_pool_getter(&self) -> Arc<dyn LivePoolGetter>{
        Arc::new(LivePoolGetterImpl{ anon_class_live_object_ldc_pool: self.anon_class_live_object_ldc_pool.clone() })
    }
}

#[derive(Debug)]
pub struct StackEntry {
    pub last_call_stack: Option<Rc<StackEntry>>,
    pub class_pointer: Arc<RuntimeClass>,
    pub method_i: CPIndex,

    pub local_vars: RefCell<Vec<JavaValue>>,
    pub operand_stack: RefCell<Vec<JavaValue>>,
    pub pc: RefCell<usize>,
    //the pc_offset is set by every instruction. branch instructions and others may us it to jump
    pub pc_offset: RefCell<isize>,
}

impl StackEntry {
    pub fn pop(&self) -> JavaValue {
        self.operand_stack.borrow_mut().pop().unwrap_or_else(|| {
            let classfile = &self.class_pointer.classfile;
            let method = &classfile.methods[self.method_i as usize];
            dbg!(&method.method_name(&classfile));
            dbg!(&method.code_attribute().unwrap().code);
            dbg!(&self.pc);
            panic!()
        })
    }
    pub fn push(&self, j: JavaValue) {
        self.operand_stack.borrow_mut().push(j)
    }

    pub fn depth(&self) -> usize {
        match &self.last_call_stack {
            None => 0,
            Some(last) => last.depth() + 1,
        }
    }

    pub fn print_stack_trace(&self) {
        let class_file = &self.class_pointer.classfile;
        let method = &class_file.methods[self.method_i as usize];
        println!("{} {} {} {}",
                 &class_name(class_file).get_referred_name(),
                 method.method_name(class_file),
                 method.descriptor_str(class_file),
                 self.depth());
        match &self.last_call_stack {
            None => {}
            Some(last) => last.print_stack_trace(),
        }
    }
}


#[derive(Debug)]
pub struct LibJavaLoading {
    pub lib: Library,
    pub registered_natives: RefCell<HashMap<Arc<RuntimeClass>, RefCell<HashMap<u16, unsafe extern fn()>>>>,
}


pub fn get_or_create_class_object(state: &mut InterpreterState,
                                  type_: &PTypeView,
                                  current_frame: Rc<StackEntry>,
                                  loader_arc: LoaderArc,
) -> Arc<Object> {
    match type_ {
        PTypeView::Ref(t) => match t {
            ReferenceTypeView::Array(c) => {
                return array_object(state, c.deref(), current_frame);
            }
            _ => {}
        },
        _ => {}
    }

    regular_object(state, type_, current_frame, loader_arc)
}

fn array_object(state: &mut InterpreterState, array_sub_type: &PTypeView, current_frame: Rc<StackEntry>) -> Arc<Object> {
    let type_for_object: PType = array_sub_type.to_ptype();
    array_of_type_class(state, current_frame, &type_for_object)
}

pub fn array_of_type_class(state: &mut InterpreterState, current_frame: Rc<StackEntry>, type_for_object: &PType) -> Arc<Object> {
    //todo wrap in array and convert
    let array_type = PTypeView::Ref(ReferenceTypeView::Array(PTypeView::from_ptype(type_for_object).into()));
    let res = state.class_object_pool.borrow().get(&array_type).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, current_frame);
            let array_ptype_view = array_type.clone().into();
            r.unwrap_normal_object().class_object_ptype.replace(array_ptype_view);
            state.class_object_pool.borrow_mut().insert(array_type, r.clone());
            r
        }
        Some(r) => r.clone(),
    }
}

fn regular_object(state: &mut InterpreterState, class_type: &PTypeView, current_frame: Rc<StackEntry>, loader_arc: LoaderArc) -> Arc<Object> {
    check_inited_class(state, class_type.unwrap_type_to_name().as_ref().unwrap(), current_frame.clone().into(), loader_arc);
    let res = state.class_object_pool.borrow().get(&class_type).cloned();
    match res {
        None => {
            let r = create_a_class_object(state, current_frame.clone());
            r.unwrap_normal_object().class_object_ptype.replace(Some(class_type.clone()));
            state.class_object_pool.borrow_mut().insert(class_type.clone(), r.clone());
            if class_type.is_primitive() {
                //handles edge case of classes whose names do not correspond to the name of the class they represent
                //normally names are obtained with getName0 which gets handled in libjvm.so
                create_string_on_stack(state, &current_frame, class_type.primitive_name().to_string());
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
    let boostrap_loader_object = Arc::new(Object::Object(NormalObject {
        gc_reachable: true,
        fields: RefCell::new(HashMap::new()),
        class_pointer: class_loader_class.clone(),
        bootstrap_loader: true,
        // object_class_object_pointer: RefCell::new(None),
        // array_class_object_pointer: RefCell::new(None),
        class_object_ptype: RefCell::new(None),
    }));
    // state.class_loader = boostrap_loader_object;
    //the above would only be required for higher jdks where a class loader object is part of Class.
    //as it stands we can just push to operand stack
    push_new_object(state,current_frame.clone(), &class_class);
    let object = current_frame.pop();
    match object.clone() {
        JavaValue::Object(o) => {
            let bootstrap_arc = boostrap_loader_object;
            let bootstrap_class_loader = JavaValue::Object(bootstrap_arc.clone().into());
            {
                bootstrap_arc.unwrap_normal_object().fields.borrow_mut().insert("assertionLock".to_string(), bootstrap_class_loader.clone());//itself...
                bootstrap_arc.unwrap_normal_object().fields.borrow_mut().insert("classAssertionStatus".to_string(), JavaValue::Object(None));
                o.as_ref().unwrap().unwrap_normal_object().fields.borrow_mut().insert("classLoader".to_string(), JavaValue::Object(None));
            }
            if !state.system_domain_loader{
                o.as_ref().unwrap().unwrap_normal_object().fields.borrow_mut().insert("classLoader".to_string(),bootstrap_class_loader);
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
    let mut state = InterpreterState {
        terminate: false,
        throw: None,
        function_return: false,
        bootstrap_loader: bl.clone(),
        initialized_classes: RwLock::new(HashMap::new()),
        string_internment: RefCell::new(HashMap::new()),
        class_object_pool: RefCell::new(HashMap::new()),
        system_domain_loader: true,
        jni,
        string_pool: StringPool {
            entries: HashSet::new()
        },
        start_instant: Instant::now(),
        anon_class_counter: 0,
        anon_class_live_object_ldc_pool: Arc::new(RefCell::new(vec![])),
        debug_exclude: true
    };
    let main = bl.clone().load_class(bl.clone(), main_class_name, bl.clone(),state.get_live_object_pool_getter())?;
    let main_class = prepare_class(main.clone().backing_class(), bl.clone());
    let main_i = locate_main_method(&bl, &main.backing_class());
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
        if m.is_static() && desc.parameter_types == vec![string_array.to_ptype()] && desc.return_type == PType::VoidType {
            return i;
        }
    }
    panic!();
}

pub mod instructions;
pub mod interpreter_util;
pub mod rust_jni;
pub mod loading;