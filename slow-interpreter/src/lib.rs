#![feature(c_variadic)]
#![feature(thread_local)]
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
use classfile_view::loading::{LoaderArc, LivePoolGetter, LoaderName};
use descriptor_parser::MethodDescriptor;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use crate::jvmti::SharedLibJVMTI;
use crate::java::lang::thread::JThread;
use crate::loading::{Classpath, BootstrapLoader};
use std::borrow::Borrow;
use crate::stack_entry::StackEntry;
use std::thread::LocalKey;


pub mod java_values;
pub mod runtime_class;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
pub mod utils;

type ThreadId = u64;

#[derive(Debug)]
pub struct JavaThread {
    // pub jvm_pointer : &'vmlife JVMState<'vmlife>,
    java_tid: ThreadId,
    name: String,
    call_stack: RefCell<Vec<StackEntry>>,
    thread_object: RefCell<Option<JThread>>,
    //for the main thread the object may not exist for a bit,b/c the code to create that object needs to run on a thread
    //todo maybe this shouldn't be private?
    pub interpreter_state: InterpreterState,
}

impl JavaThread/*<'_>*/ {
    fn raw_thread(&self) -> *const Self {
        self as *const Self
    }
}

#[derive(Debug)]
pub struct InterpreterState {
    pub terminate: RefCell<bool>,
    pub throw: RefCell<Option<Arc<Object>>>,
    pub function_return: RefCell<bool>,
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


pub struct JVMState {
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
    pub anon_class_counter: AtomicUsize,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,

    pub built_in_jdwp: Arc<SharedLibJVMTI>,

    main_thread: RwLock<Option<Arc<JavaThread/*<'vmlife>*/>>>,
    pub all_threads: RwLock<Vec<Arc<JavaThread/*<'vmlife>*/>>>,
    pub main_class_name: ClassName,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread>>>>,
}


thread_local! {
        static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread>>> = RefCell::new(None);
    }

impl JVMState {
    pub fn set_current_thread(&self, java_thread: Arc<JavaThread>) {
        self.current_java_thread.with(|x| x.replace(java_thread.into()));
    }

    pub fn get_current_thread(&self) -> Arc<JavaThread> {
        self.current_java_thread.with(|thread_refcell| thread_refcell.borrow().as_ref().unwrap().clone())
    }

    pub fn get_current_frame(&self) -> &StackEntry{
        self.get_current_thread().call_stack.borrow().last().unwrap()
    }

    pub fn new(jvm_options: JVMOptions) -> Self {
        let JVMOptions { main_class_name, classpath, args, shared_libs } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let bootstrap_loader = Arc::new(BootstrapLoader {
            loaded: RwLock::new(HashMap::new()),
            parsed: RwLock::new(HashMap::new()),
            name: RwLock::new(LoaderName::BootstrapLoader),
            classpath,
        });


        Self {
            bootstrap_loader,
            initialized_classes: RwLock::new(HashMap::new()),
            class_object_pool: RwLock::new(HashMap::new()),
            system_domain_loader: true,
            jni: LibJavaLoading::new_java_loading(libjava),
            string_pool: StringPool {
                entries: HashSet::new()
            },
            start_instant: Instant::now(),
            anon_class_live_object_ldc_pool: Arc::new(RwLock::new(vec![])),
            built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(libjdwp.as_str())),
            anon_class_counter: AtomicUsize::new(0),
            all_threads: vec![].into(),
            main_thread: RwLock::new(None),
            main_class_name,
            current_java_thread: &CURRENT_JAVA_THREAD,
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


impl /*<'jvmlife>*/ JVMState/*<'jvmlife>*/ {
    pub fn get_live_object_pool_getter(&self) -> Arc<dyn LivePoolGetter> {
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.anon_class_live_object_ldc_pool.clone() })
    }

    pub fn register_main_thread(&self, main_thread: Arc<JavaThread>) {
        //todo perhaps there should be a single rw lock for this
        self.all_threads.write().unwrap().push(main_thread.clone());
        let mut main_thread_writer = self.main_thread.write().unwrap();
        main_thread_writer.replace(main_thread.clone().into());
        self.set_current_thread(main_thread);
    }

    pub fn main_thread(&self) -> Arc<JavaThread> {
        let read_guard = self.main_thread.read().unwrap();
        read_guard.as_ref().unwrap().clone()
    }
}

pub fn run(opts: JVMOptions) -> Result<(), Box<dyn Error>> {
    let mut jvm = JVMState::new(opts);
    jvm_run_system_init(&mut jvm);
    let main_view = jvm.bootstrap_loader.load_class(jvm.bootstrap_loader.clone(), &jvm.main_class_name, jvm.bootstrap_loader.clone(), jvm.get_live_object_pool_getter())?;
    let main_class = prepare_class(main_view.clone().backing_class(), jvm.bootstrap_loader.clone());
    let main_i = locate_main_method(&jvm.bootstrap_loader, &main_view.backing_class());
    let main_thread = jvm.main_thread();


    /*let main_stack = Rc::new(StackEntry {
        last_call_stack: None,
        class_pointer: Arc::new(main_class),
        method_i: main_i as u16,
        local_vars: vec![].into(),//todo handle parameters, todo handle non-zero size locals
        operand_stack: vec![].into(),
        pc: RefCell::new(0),
        pc_offset: 0.into(),
    });*/


    jvm.built_in_jdwp.vm_inited(&jvm);
    run_function(&jvm);
    if main_thread.interpreter_state.throw.borrow().is_some() || *main_thread.interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    Result::Ok(())
}

fn jvm_run_system_init(jvm: &JVMState) {
    let bl = &jvm.bootstrap_loader;
    let bootstrap_system_class_view = bl.load_class(bl.clone(), &ClassName::system(), bl.clone(), jvm.get_live_object_pool_getter()).unwrap();
    let bootstrap_system_class = Arc::new(prepare_class(bootstrap_system_class_view.backing_class(), bl.clone()));
    let bootstrap_frame = StackEntry {
        class_pointer: bootstrap_system_class,
        method_i: 0,
        local_vars: RefCell::new(vec![]),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    };
    jvm.register_main_thread(Arc::new(JavaThread {
        java_tid: 0,
        name: "Main".to_string(),
        call_stack: RefCell::new(vec![bootstrap_frame]),
        thread_object: RefCell::new(None),
        interpreter_state: InterpreterState {
            terminate: RefCell::new(false),
            throw: RefCell::new(None),
            function_return: RefCell::new(false),
        },
    }));
    let system_class = check_inited_class(jvm, &ClassName::system(), bl.clone());
    let (init_system_class_i, method_info) = locate_init_system_class(&system_class.classfile);
    let mut locals = vec![];
    for _ in 0..method_info.code_attribute().unwrap().max_locals {
        locals.push(JavaValue::Top);
    }
    let initialize_system_frame = StackEntry {
        class_pointer: system_class.clone(),
        method_i: init_system_class_i as u16,
        local_vars: RefCell::new(locals),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(-1),
    };
    jvm.get_current_thread().call_stack.replace(vec![initialize_system_frame]);
    jvm.built_in_jdwp.agent_load(jvm, &jvm.main_thread());
//todo technically this needs to before any bytecode is run.
    run_function(&jvm);
    if *jvm.main_thread().interpreter_state.function_return.borrow() {
        jvm.main_thread().interpreter_state.function_return.replace(false);
    }
    if jvm.main_thread().interpreter_state.throw.borrow().is_some() || *jvm.main_thread().interpreter_state.terminate.borrow() {
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