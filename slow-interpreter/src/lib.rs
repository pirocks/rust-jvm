#![feature(c_variadic)]
#![feature(thread_local)]
#![feature(vec_leak)]
#![feature(box_syntax)]
extern crate log;
extern crate simple_logger;
extern crate libloading;
extern crate libc;
extern crate regex;
extern crate va_list;
extern crate lock_api;
extern crate parking_lot;
extern crate futures_intrusive;
extern crate nix;
extern crate errno;

use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;
use rust_jvm_common::ptype::PType;
use rust_jvm_common::classfile::Classfile;
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue, NormalObject, ArrayObject};
use crate::runtime_class::prepare_class;
use crate::interpreter_util::check_inited_class;
use libloading::Library;
use classfile_view::view::ptype_view::{ReferenceTypeView, PTypeView};
use classfile_view::loading::{LoaderArc, LivePoolGetter, LoaderName};
use descriptor_parser::MethodDescriptor;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicUsize;
use std::error::Error;
use std::collections::{HashMap, HashSet};
use std::cell::RefCell;
use std::time::{Instant};
use crate::java::lang::thread::JThread;
use crate::loading::{Classpath, BootstrapLoader};
use crate::stack_entry::StackEntry;
use std::thread::LocalKey;
use std::rc::Rc;
use crate::monitor::Monitor;
use jvmti_jni_bindings::JNIInvokeInterface_;
use std::ffi::c_void;
use jvmti_jni_bindings::{jrawMonitorID, jlong};
use lock_api::Mutex;
use parking_lot::RawMutex;
use crate::tracing::TracingSettings;
use crate::interpreter::run_function;
use classfile_view::view::method_view::MethodView;
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use nix::unistd::{Pid, gettid};
use crate::method_table::{MethodTable, MethodId};
use crate::field_table::FieldTable;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use std::ops::Deref;
use crate::java_values::Object::Array;


pub mod java_values;
pub mod runtime_class;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
pub mod utils;

type ThreadId = i64;

#[derive(Debug)]
pub struct JavaThread {
    pub java_tid: ThreadId,
    pub call_stack: RefCell<Vec<Rc<StackEntry>>>,
    pub thread_object: RefCell<Option<JThread>>,
    //for the main thread the object may not exist for a bit,b/c the code to create that object needs to run on a thread
    //todo maybe this shouldn't be private?
    pub interpreter_state: InterpreterState,
    pub unix_tid: Pid,
}

//todo is this correct?
unsafe impl Send for JavaThread {}

unsafe impl Sync for JavaThread {}

impl JavaThread {
    pub fn get_current_frame(&self) -> Rc<StackEntry> {
        self.call_stack.borrow().last().unwrap().clone()
    }
    pub fn print_stack_trace(&self) {
        self.call_stack.borrow().iter().rev().enumerate().for_each(|(i, stack_entry)| {
            let name = stack_entry.class_pointer.view().name();
            let meth_name = stack_entry.class_pointer.view().method_view_i(stack_entry.method_i as usize).name();
            println!("{}.{} {} pc: {}", name.get_referred_name(), meth_name, i, stack_entry.pc.borrow())
        });
    }
}

#[derive(Debug)]
pub struct InterpreterState {
    pub terminate: RefCell<bool>,
    pub throw: RefCell<Option<Arc<Object>>>,
    pub function_return: RefCell<bool>,
    //todo find some way of clarifying these can only be acessed from one thread
    pub suspended: std::sync::RwLock<SuspendedStatus>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        InterpreterState {
            terminate: RefCell::new(false),
            throw: RefCell::new(None),
            function_return: RefCell::new(false),
            suspended: RwLock::new(SuspendedStatus {
                suspended: false,
                suspended_lock: Arc::new(Mutex::new(())),
            }),
        }
    }
}

#[derive(Debug)]
pub struct SuspendedStatus {
    pub suspended: bool,
    pub suspended_lock: Arc<Mutex<RawMutex, ()>>,
    // pub suspend_critical_section_lock: Mutex<RawMutex,()>
}

pub struct SharedLibraryPaths {
    libjava: String,
    libjdwp: String,
}

pub struct JVMOptions {
    main_class_name: ClassName,
    classpath: Classpath,
    args: Vec<String>,
    //todo args not implemented yet
    shared_libs: SharedLibraryPaths,
    enable_tracing: bool,
    enable_jvmti: bool,
    properties: Vec<String>,
}

impl JVMOptions {
    pub fn new(main_class_name: ClassName,
               classpath: Classpath,
               args: Vec<String>,
               libjava: String,
               libjdwp: String,
               enable_tracing: bool,
               enable_jvmti: bool,
               properties: Vec<String>,
    ) -> Self {
        Self {
            main_class_name,
            classpath,
            args,
            shared_libs: SharedLibraryPaths { libjava, libjdwp },
            enable_tracing,
            enable_jvmti,
            properties,
        }
    }
}

pub struct JVMState {
    properties: Vec<String>,
    loaders: RwLock<HashMap<LoaderName, Arc<Object>>>,
    pub bootstrap_loader: LoaderArc,
    pub system_domain_loader: bool,
    pub string_pool: StringPool,
    pub start_instant: Instant,
    //todo needs to be used for all instances of getClass
    pub libjava: LibJavaLoading,

    pub initialized_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub class_object_pool: RwLock<HashMap<Arc<RuntimeClass>, Arc<Object>>>,
    //anon classes
    pub anon_class_counter: AtomicUsize,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,

    pub main_class_name: ClassName,

    pub classpath: Arc<Classpath>,
    invoke_interface: RwLock<Option<JNIInvokeInterface_>>,

    pub jvmti_state: Option<JVMTIState>,
    pub thread_state: ThreadState,
    pub tracing: TracingSettings,
    pub method_table: RwLock<MethodTable>,
    pub field_table: RwLock<FieldTable>,
    live: RwLock<bool>,
}


pub struct ThreadState {
    main_thread: RwLock<Option<Arc<JavaThread>>>,
    pub alive_threads: RwLock<HashMap<ThreadId, Arc<JavaThread>>>,
    current_java_thread: &'static LocalKey<RefCell<Option<Arc<JavaThread>>>>,
    pub system_thread_group: RwLock<Option<Arc<Object>>>,
    monitors: RwLock<Vec<Arc<Monitor>>>,
}

impl ThreadState {
    pub fn get_monitor(&self, monitor: jrawMonitorID) -> Arc<Monitor> {
        let monitors_read_guard = self.monitors.read().unwrap();
        let monitor = monitors_read_guard[monitor as usize].clone();
        std::mem::drop(monitors_read_guard);
        monitor
    }
}

thread_local! {
        static JVMTI_TLS: RefCell<*mut c_void> = RefCell::new(std::ptr::null_mut());
    }

thread_local! {
        static CURRENT_JAVA_THREAD: RefCell<Option<Arc<JavaThread>>> = RefCell::new(None);
    }

impl JVMState {
    pub fn set_current_thread(&self, java_thread: Arc<JavaThread>) {
        self.thread_state.current_java_thread.with(|x| x.replace(java_thread.into()));
    }

    pub fn get_current_thread(&self) -> Arc<JavaThread> {
        self.thread_state.current_java_thread.with(|thread_refcell| {
            thread_refcell.borrow().as_ref().unwrap().clone()
        })
    }

    pub fn get_current_frame(&self) -> Rc<StackEntry> {
        let current_thread = self.get_current_thread();
        let temp = current_thread.call_stack.borrow();
        temp.last().unwrap().clone()
    }

    pub fn get_previous_frame(&self) -> Rc<StackEntry> {
        let thread = self.get_current_thread();
        let call_stack = thread.call_stack.borrow();
        call_stack.get(call_stack.len() - 2).unwrap().clone()
    }

    pub fn new(jvm_options: JVMOptions) -> (Vec<String>, Self) {
        let JVMOptions { main_class_name, classpath, args, shared_libs, enable_tracing, enable_jvmti, properties } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let classpath_arc = Arc::new(classpath);
        let bootstrap_loader = Arc::new(BootstrapLoader {
            loaded: RwLock::new(HashMap::new()),
            parsed: RwLock::new(HashMap::new()),
            name: RwLock::new(LoaderName::BootstrapLoader),
            classpath: classpath_arc.clone(),
        });


        let tracing = if enable_tracing { TracingSettings::new() } else { TracingSettings::disabled() };

        let jvmti_state = if enable_jvmti {
            JVMTIState {
                built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(libjdwp.as_str())),
                jvmti_thread_local_storage: &JVMTI_TLS,
                break_points: RwLock::new(HashMap::new()),
                tags: RwLock::new(HashMap::new()),
            }.into()
        } else { None };
        let thread_state = ThreadState {
            alive_threads: RwLock::new(HashMap::new()),
            main_thread: RwLock::new(None),
            current_java_thread: &CURRENT_JAVA_THREAD,
            system_thread_group: RwLock::new(None),
            monitors: RwLock::new(vec![]),
        };
        (args, Self {
            properties,
            loaders: RwLock::new(HashMap::new()),
            bootstrap_loader,
            initialized_classes: RwLock::new(HashMap::new()),
            class_object_pool: RwLock::new(HashMap::new()),
            system_domain_loader: true,
            libjava: LibJavaLoading::new_java_loading(libjava),
            string_pool: StringPool {
                entries: HashSet::new()
            },
            start_instant: Instant::now(),
            anon_class_live_object_ldc_pool: Arc::new(RwLock::new(vec![])),
            anon_class_counter: AtomicUsize::new(0),
            main_class_name,
            classpath: classpath_arc,
            invoke_interface: RwLock::new(None),
            jvmti_state,
            thread_state,
            tracing,
            method_table: RwLock::new(MethodTable::new()),
            field_table: RwLock::new(FieldTable::new()),
            live: RwLock::new(false),
        })
    }

    pub fn new_monitor(&self, name: String) -> Arc<Monitor> {
        let mut monitor_guard = self.thread_state.monitors.write().unwrap();
        let index = monitor_guard.len();
        let res = Arc::new(Monitor::new(name, index));
        monitor_guard.push(res.clone());
        res
    }

    pub fn get_current_thread_name(&self) -> String {
        let current_thread = self.get_current_thread();
        let thread_object = current_thread.thread_object.borrow();
        thread_object.as_ref().map(|jthread| jthread.name().to_rust_string())
            .unwrap_or(std::thread::current().name().unwrap_or("unknown").to_string())
    }

    pub fn get_or_create_bootstrap_object_loader(&self) -> JavaValue {
        if !self.vm_live() {
            return JavaValue::Object(None);
        }
        let mut loader_guard = self.loaders.write().unwrap();
        match loader_guard.get(&self.bootstrap_loader.name()) {
            None => {
                let java_lang_class_loader = ClassName::new("java/lang/ClassLoader");
                let current_loader = self.get_current_frame().class_pointer.loader(self).clone();
                let class_loader_class = check_inited_class(self, &java_lang_class_loader.into(), current_loader.clone());
                let res = Arc::new(Object::Object(NormalObject {
                    monitor: self.new_monitor("bootstrap loader object monitor".to_string()),
                    fields: RefCell::new(HashMap::new()),
                    class_pointer: class_loader_class.clone(),
                    class_object_type: None,
                }));
                loader_guard.insert(self.bootstrap_loader.name(), res.clone());
                JavaValue::Object(res.into())
            }
            Some(res) => { JavaValue::Object(res.clone().into()) }
        }
    }
}


type CodeIndex = isize;
type TransmutedObjectPointer = usize;

pub struct JVMTIState {
    pub built_in_jdwp: Arc<SharedLibJVMTI>,
    jvmti_thread_local_storage: &'static LocalKey<RefCell<*mut c_void>>,
    pub break_points: RwLock<HashMap<MethodId, HashSet<CodeIndex>>>,
    pub tags: RwLock<HashMap<TransmutedObjectPointer, jlong>>,
}

struct LivePoolGetterImpl {
    anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>
}

#[derive(Debug)]
pub struct LibJavaLoading {
    pub libjava: Library,
    pub libnio: Library,
    pub registered_natives: RwLock<HashMap<Arc<RuntimeClass>, RwLock<HashMap<u16, unsafe extern fn()>>>>,
}

impl LivePoolGetter for LivePoolGetterImpl {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.read().unwrap()[idx];
        ReferenceTypeView::Class(object.unwrap_normal_object().class_pointer.view().name())//todo handle arrays
    }
}

pub struct NoopLivePoolGetter {}

impl LivePoolGetter for NoopLivePoolGetter {
    fn elem_type(&self, _idx: usize) -> ReferenceTypeView {
        panic!()
    }
}


impl JVMState {
    pub fn vm_live(&self) -> bool {
        *self.live.read().unwrap()
    }

    pub fn get_live_object_pool_getter(&self) -> Arc<dyn LivePoolGetter> {
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.anon_class_live_object_ldc_pool.clone() })
    }

    pub fn register_main_thread(&self, main_thread: Arc<JavaThread>) {
        //todo perhaps there should be a single rw lock for this
        self.thread_state.alive_threads.write().unwrap().insert(1, main_thread.clone());
        let mut main_thread_writer = self.thread_state.main_thread.write().unwrap();
        main_thread_writer.replace(main_thread.clone().into());
        self.set_current_thread(main_thread);
    }

    pub fn main_thread(&self) -> Arc<JavaThread> {
        let read_guard = self.thread_state.main_thread.read().unwrap();
        read_guard.as_ref().unwrap().clone()
    }
}

pub fn run(opts: JVMOptions) -> Result<(), Box<dyn Error>> {
    let (args, mut jvm) = JVMState::new(opts);
    jvm_run_system_init(&mut jvm);
    jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.vm_inited(&jvm));
    let main_view = jvm.bootstrap_loader.load_class(jvm.bootstrap_loader.clone(), &jvm.main_class_name, jvm.bootstrap_loader.clone(), jvm.get_live_object_pool_getter())?;
    let main_class = prepare_class(&jvm, main_view.clone().backing_class(), jvm.bootstrap_loader.clone());
    jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.class_prepare(&jvm, &main_view.name()));
    let main_i = locate_main_method(&jvm.bootstrap_loader, &main_view.backing_class());
    let main_thread = jvm.main_thread();
    assert!(Arc::ptr_eq(&jvm.get_current_thread(), &main_thread));
    let num_vars = main_view.method_view_i(main_i).code_attribute().unwrap().max_locals;
    // jvm.jvmti_state.built_in_jdwp.vm_start(&jvm);
    let stack_entry = StackEntry {
        class_pointer: Arc::new(main_class),
        method_i: main_i as u16,
        local_vars: vec![JavaValue::Top; num_vars as usize].into(),//todo handle parameters, todo handle non-zero size locals
        operand_stack: vec![].into(),
        pc: RefCell::new(0),
        pc_offset: 0.into(),
    };
    let main_stack = Rc::new(stack_entry);
    jvm.main_thread().call_stack.replace(vec![main_stack]);
    jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.thread_start(&jvm, jvm.main_thread().thread_object.borrow().clone().unwrap()));
    //trigger breakpoint on thread.resume for debuggers that rely on that:

    let thread_class = check_inited_class(
        &jvm,
        &ClassName::thread().into(),
        jvm.bootstrap_loader.clone(),
    );
    //todo is firing a resume breakpoint event really needed?
    let method_i = thread_class
        .view()
        .lookup_method(&"resume".to_string(),
                       &MethodDescriptor {
                           parameter_types: vec![],
                           return_type: PType::VoidType,
                       })
        .unwrap()
        .method_i();
    let thread_resume_id = jvm.method_table.write().unwrap().get_method_id(thread_class, method_i as u16);
    match &jvm.jvmti_state {
        None => {}
        Some(jvmti) => {
            let breakpoints = jvmti.break_points.read().unwrap();
            let breakpoint_offsets = breakpoints.get(&thread_resume_id);
            std::mem::drop(breakpoint_offsets
                .map(|x| x.iter().for_each(|i| {
                    //todo handle this breakpoint the usual way
                    jvmti.built_in_jdwp.breakpoint(&jvm, thread_resume_id.clone(), *i as i64);
                })));
            std::mem::drop(breakpoints);
            jvmti.built_in_jdwp.breakpoint(&jvm, thread_resume_id.clone(), 0);
        }
    }

    setup_program_args(&jvm, args);
    run_function(&jvm);
    if main_thread.interpreter_state.throw.borrow().is_some() || *main_thread.interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    Result::Ok(())
}


fn setup_program_args(jvm: &JVMState, args: Vec<String>) {
    let current_frame = jvm.get_current_frame();

    let arg_strings:Vec<JavaValue> = args.iter().map(|arg_str| {
        JString::from(jvm, current_frame.deref(), arg_str.clone()).java_value()
    }).collect();
    let arg_array = JavaValue::Object(Some(Arc::new(Array(ArrayObject {
        elems: RefCell::new(arg_strings),
        elem_type: PTypeView::Ref(ReferenceTypeView::Class(ClassName::string())),
        monitor: jvm.new_monitor( "arg array monitor".to_string())
    }))));
    let mut local_vars = current_frame.local_vars.borrow_mut();
    local_vars[0] = arg_array;
}

fn jvm_run_system_init(jvm: &JVMState) {
    let bl = &jvm.bootstrap_loader;
    let bootstrap_system_class_view = bl.load_class(bl.clone(), &ClassName::system(), bl.clone(), jvm.get_live_object_pool_getter()).unwrap();
    let bootstrap_system_class = Arc::new(prepare_class(jvm, bootstrap_system_class_view.backing_class(), bl.clone()));
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
        call_stack: RefCell::new(vec![Rc::new(bootstrap_frame)]),
        thread_object: RefCell::new(None),
        interpreter_state: InterpreterState::default(),
        unix_tid: gettid(),
    }));
    let system_class = check_inited_class(jvm, &ClassName::system().into(), bl.clone());
    let init_method_view = locate_init_system_class(&system_class);
    let mut locals = vec![];
    for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
        locals.push(JavaValue::Top);
    }
    let initialize_system_frame = Rc::new(StackEntry {
        class_pointer: system_class.clone(),
        method_i: init_method_view.method_i() as u16,
        local_vars: RefCell::new(locals),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(0),
    });
    jvm.get_current_thread().call_stack.replace(vec![initialize_system_frame.clone()]);
    jvm.jvmti_state.as_ref().map(|jvmti| jvmti.built_in_jdwp.agent_load(jvm, &jvm.main_thread()));
//todo technically this needs to before any bytecode is run.
    run_function(&jvm);
    if *jvm.main_thread().interpreter_state.function_return.borrow() {
        jvm.main_thread().interpreter_state.function_return.replace(false);
    }
    if jvm.main_thread().interpreter_state.throw.borrow().is_some() || *jvm.main_thread().interpreter_state.terminate.borrow() {
        unimplemented!()
    }
    *jvm.live.write().unwrap() = true;
    set_properties(jvm);
}

fn set_properties(jvm: &JVMState) {
    let properties = &jvm.properties;
    let prop_obj = System::props(jvm);
    assert_eq!(properties.len() % 2, 0);
    for i in 0..properties.len() / 2 {
        let key_i = 2 * i;
        let value_i = 2 * i + 1;
        let current_frame = &jvm.get_current_frame();
        let key = JString::from(jvm, current_frame, properties[key_i].clone());
        let value = JString::from(jvm, current_frame, properties[value_i].clone());
        prop_obj.set_property(jvm, current_frame, key, value);
    }
}


fn locate_init_system_class(system: &Arc<RuntimeClass>) -> MethodView {
    let system_class = system.view();
    let method_views = system_class.lookup_method_name(&"initializeSystemClass".to_string());
    method_views.first().unwrap().clone()
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


pub mod thread_signalling;
pub mod instructions;
pub mod interpreter_util;
pub mod rust_jni;
pub mod loading;
pub mod jvmti;
pub mod invoke_interface;
pub mod stack_entry;
pub mod class_objects;
pub mod monitor;
pub mod tracing;
pub mod interpreter;
pub mod signal;
pub mod method_table;
pub mod field_table;