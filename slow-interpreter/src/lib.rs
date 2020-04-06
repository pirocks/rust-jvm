#![feature(c_variadic)]
extern crate log;
extern crate simple_logger;
extern crate libloading;
extern crate libc;
extern crate regex;
extern crate va_list;

use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;
use rust_jvm_common::ptype::PType;
use rust_jvm_common::classfile::{Classfile, MethodInfo};
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue};
use crate::runtime_class::prepare_class;
use crate::interpreter_util::{run_function, check_inited_class};
use libloading::Library;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use classfile_view::loading::{LoaderArc, LivePoolGetter, Loader, LoaderName};
use descriptor_parser::MethodDescriptor;
use std::sync::{Arc, RwLock};
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use crate::jvmti::SharedLibJVMTI;
use crate::java::lang::thread::JThread;
use crate::stack_entry::StackEntry;
use crate::loading::{Classpath, BootstrapLoader};


pub mod java_values;
pub mod runtime_class;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
pub mod utils;

type ThreadId = u64;

pub struct JavaThread {
    java_tid: ThreadId,
    name: String,
    call_stack: Rc<StackEntry>,
    thread_object: RefCell<Option<JThread>>,
    //for the main thread the object may not exist for a bit,b/c the code to create that object needs to run on a thread
    interpreter_state: InterpreterState,
}

pub struct InterpreterState {
    pub terminate: bool,
    pub throw: Option<Arc<Object>>,
    pub function_return: bool,
}

pub struct SharedLibraryPaths {
    libjava: String,
    libjdwp: String,
}

pub struct JVMOptions {
    main_class_name: ClassName,
    classpath: Classpath,
    args: Vec<String>,
    shared_libs: SharedLibraryPaths,

}

impl JVMOptions {
    pub fn new(main_class_name: ClassName,
               classpath: Classpath,
               args: Vec<String>,
               libjava: String,
               libjdwp: String,
    ) -> Self {
        Self {
            main_class_name,
            classpath,
            args,
            shared_libs: SharedLibraryPaths { libjava, libjdwp },
        }
    }
}

// let jni = new_java_loading(libjava);
// let jdwp = load_libjdwp(libjdwp.as_str());


pub struct JVMState<'vmlifetime> {
    pub bootstrap_loader: LoaderArc,
    pub initialized_classes: RwLock<HashMap<ClassName, Arc<RuntimeClass>>>,
    pub string_pool: StringPool,
    //todo this should really be in some sort of parser/jvm state
    pub start_instant: Instant,

    pub class_object_pool: RwLock<HashMap<PTypeView, Arc<Object>>>,
    pub system_domain_loader: bool,

    //todo needs to be used for all instances of getClass
    pub jni: LibJavaLoading,//todo rename to libjava


    //anon classes
    pub anon_class_counter: RwLock<usize>,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,

    pub built_in_jdwp: Arc<SharedLibJVMTI>,

    pub main_thread: RwLock<Option<JavaThread>>,
    pub all_threads: RwLock<Vec<JavaThread>>,

    pub options: JVMOptions,
}

impl JVMState {
    pub fn new(jvm_options: JVMOptions, bl: LoaderArc) -> Self {
        Self {
            bootstrap_loader: bl,
            initialized_classes: RwLock::new(HashMap::new()),
            class_object_pool: RwLock::new(HashMap::new()),
            system_domain_loader: true,
            jni,
            string_pool: StringPool {
                entries: HashSet::new()
            },
            start_instant: Instant::now(),
            anon_class_live_object_ldc_pool: Arc::new(RwLock::new(vec![])),
            built_in_jdwp: Arc::new(jdwp),
            anon_class_counter: RwLock::new(0),
            all_threads: vec![].into(),
            options: jvm_options,
            main_thread: RwLock::new(None)
        }
    }
}

struct LivePoolGetterImpl {
    anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>
}

#[derive(Debug)]
pub struct LibJavaLoading {
    pub lib: Library,
    pub registered_natives: RefCell<HashMap<Arc<RuntimeClass>, RefCell<HashMap<u16, unsafe extern fn()>>>>,
}

impl LivePoolGetter for LivePoolGetterImpl {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.read().unwrap()[idx];
        ReferenceTypeView::Class(object.unwrap_normal_object().class_pointer.class_view.name())//todo handle arrays
    }
}

pub struct NoopLivePoolGetter {}

impl LivePoolGetter for NoopLivePoolGetter {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        panic!()
    }
}


impl JVMState {
    pub fn get_live_object_pool_getter(&self) -> Arc<dyn LivePoolGetter> {
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.anon_class_live_object_ldc_pool.clone() })
    }

    pub fn register_main_thread(&self, main_thread: JavaThread) {
        //todo perhaps there should be a single rw lock for this
        self.all_threads.write().unwrap().push(main_thread);
        let mut main_thread_writer = self.main_thread.write().unwrap();
        main_thread_writer.replace(main_thread.into());
    }

    pub fn main_thread(self) -> &JavaThread{
        self.main_thread.read().unwrap().as_ref().unwrap()
    }
}

pub fn run(opts: JVMOptions) -> Result<(), Box<dyn Error>> {
    let bootstrap_loader = Arc::new(BootstrapLoader {
        loaded: RwLock::new(HashMap::new()),
        parsed: RwLock::new(HashMap::new()),
        name: RwLock::new(LoaderName::BootstrapLoader),
        classpath,
    });

    let mut state = JVMState::new(opts, bootstrap_loader.clone());
    jvm_run_system_init(&bl, &mut state);
    let main_view = bootstrap_loader.clone().load_class(bootstrap_loader.clone(), main_class_name, bootstrap_loader.clone(), state.get_live_object_pool_getter())?;
    let main_class = prepare_class(main_view.clone().backing_class(), bl.clone());
    let main_i = locate_main_method(&bootstrap_loader, &main_view.backing_class());
    let main_stack = Rc::new(StackEntry {
        last_call_stack: None,
        class_pointer: Arc::new(main_class),
        method_i: main_i as u16,
        local_vars: vec![].into(),//todo handle parameters, todo handle non-zero size locals
        operand_stack: vec![].into(),
        pc: RefCell::new(0),
        pc_offset: 0.into(),
    });



    jdwp_copy.vm_inited(&mut state, main_stack.clone());
        run_function(&mut state, main_stack);
    if state.throw.is_some() || state.terminate {
        unimplemented!()
    }
    Result::Ok(())
}

fn jvm_run_system_init(bl: &LoaderArc, state: &JVMState) {
    let system_class = check_inited_class(state, &ClassName::new("java/lang/System"), None, bl.clone());
    let (init_system_class_i, method_info) = locate_init_system_class(&system_class.classfile);
    let mut locals = vec![];
    for _ in 0..method_info.code_attribute().unwrap().max_locals {
        locals.push(JavaValue::Top);
    }
    let initialize_system_frame = Rc::new(StackEntry {
        last_call_stack: None,
        class_pointer: system_class.clone(),
        method_i: init_system_class_i as u16,
        local_vars: RefCell::new(locals),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(-1),
    });


    state.register_main_thread(JavaThread {
        java_tid: 0,
        name: "Main".to_string(),
        call_stack: main_stack,
        thread_object: RefCell::new(None),
        interpreter_state: InterpreterState{
            terminate: false,
            throw: None,
            function_return: false
        }
    });


    state.built_in_jdwp.agent_load(&mut state, initialize_system_frame.clone());
//todo technically this needs to before any bytecode is run.
    run_function(&mut state, initialize_system_frame);
    if state.function_return {
        state.function_return = false;
    }
    if state.throw.is_some() || state.terminate {
        unimplemented!()
    }
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
pub mod jvmti;
pub mod invoke_interface;
pub mod stack_entry;
pub mod class_objects;