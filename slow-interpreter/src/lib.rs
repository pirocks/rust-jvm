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

use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::string_pool::StringPool;
use rust_jvm_common::ptype::PType;
use rust_jvm_common::classfile::Classfile;
use crate::runtime_class::RuntimeClass;
use crate::java_values::{Object, JavaValue};
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
use std::time::{Instant, Duration};
use crate::jvmti::SharedLibJVMTI;
use crate::java::lang::thread::JThread;
use crate::loading::{Classpath, BootstrapLoader};
use crate::stack_entry::StackEntry;
use std::thread::LocalKey;
use std::rc::Rc;
use crate::monitor::Monitor;
use jni_bindings::JNIInvokeInterface_;
use std::ffi::c_void;
use jvmti_bindings::{jrawMonitorID, jlong};
use crate::rust_jni::MethodId;
use lock_api::Mutex;
use parking_lot::RawMutex;
use crate::tracing::TracingSettings;
use crate::interpreter::run_function;
use classfile_view::view::method_view::MethodView;


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
}

//todo is this correct?
unsafe impl Send for JavaThread {}

unsafe impl Sync for JavaThread {}

impl JavaThread {
    pub fn get_current_frame(&self) -> Rc<StackEntry> {
        self.call_stack.borrow().last().unwrap().clone()
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

#[derive(Debug)]
pub struct SuspendedStatus {
    pub suspended: bool,
    pub suspended_lock: Mutex<RawMutex, ()>,
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
    pub system_domain_loader: bool,
    pub string_pool: StringPool,
    pub start_instant: Instant,
    //todo needs to be used for all instances of getClass
    pub jni: LibJavaLoading,//todo rename to libjava


    pub initialized_classes: RwLock<HashMap<ClassName, Arc<RuntimeClass>>>,
    pub class_object_pool: RwLock<HashMap<PTypeView, Arc<Object>>>,
    //anon classes
    pub anon_class_counter: AtomicUsize,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,

    pub main_class_name: ClassName,

    pub  classpath: Arc<Classpath>,
    invoke_interface: RwLock<Option<JNIInvokeInterface_>>,

    pub jvmti_state: JVMTIState,
    pub thread_state: ThreadState,
    pub tracing: TracingSettings,
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

    pub fn new(jvm_options: JVMOptions) -> Self {
        let JVMOptions { main_class_name, classpath, args: _, shared_libs } = jvm_options;
        let SharedLibraryPaths { libjava, libjdwp } = shared_libs;
        let classpath_arc = Arc::new(classpath);
        let bootstrap_loader = Arc::new(BootstrapLoader {
            loaded: RwLock::new(HashMap::new()),
            parsed: RwLock::new(HashMap::new()),
            name: RwLock::new(LoaderName::BootstrapLoader),
            classpath: classpath_arc.clone(),
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
            anon_class_counter: AtomicUsize::new(0),
            main_class_name,
            classpath: classpath_arc,
            invoke_interface: RwLock::new(None),
            jvmti_state: JVMTIState {
                built_in_jdwp: Arc::new(SharedLibJVMTI::load_libjdwp(libjdwp.as_str())),
                jvmti_thread_local_storage: &JVMTI_TLS,
                break_points: RwLock::new(HashMap::new()),
                tags: RwLock::new(HashMap::new()),
            },
            thread_state: ThreadState {
                alive_threads: RwLock::new(HashMap::new()),
                main_thread: RwLock::new(None),
                current_java_thread: &CURRENT_JAVA_THREAD,
                system_thread_group: RwLock::new(None),
                monitors: RwLock::new(vec![]),
            },
            tracing: TracingSettings::new(),
        }
    }

    pub fn new_monitor(&self, name: String) -> Arc<Monitor> {
        let mut monitor_guard = self.thread_state.monitors.write().unwrap();
        let index = monitor_guard.len();
        let res = Arc::new(Monitor::new(name, index));
        monitor_guard.push(res.clone());
        res
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
    pub lib: Library,
    pub registered_natives: RwLock<HashMap<Arc<RuntimeClass>, RwLock<HashMap<u16, unsafe extern fn()>>>>,
}

impl LivePoolGetter for LivePoolGetterImpl {
    fn elem_type(&self, idx: usize) -> ReferenceTypeView {
        let object = &self.anon_class_live_object_ldc_pool.read().unwrap()[idx];
        ReferenceTypeView::Class(object.unwrap_normal_object().class_pointer.class_view.name())//todo handle arrays
    }
}

pub struct NoopLivePoolGetter {}

impl LivePoolGetter for NoopLivePoolGetter {
    fn elem_type(&self, _idx: usize) -> ReferenceTypeView {
        panic!()
    }
}


impl JVMState {
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
    let mut jvm = JVMState::new(opts);
    jvm_run_system_init(&mut jvm);
    let main_view = jvm.bootstrap_loader.load_class(jvm.bootstrap_loader.clone(), &jvm.main_class_name, jvm.bootstrap_loader.clone(), jvm.get_live_object_pool_getter())?;
    let main_class = prepare_class(main_view.clone().backing_class(), jvm.bootstrap_loader.clone());
    let main_i = locate_main_method(&jvm.bootstrap_loader, &main_view.backing_class());
    let main_thread = jvm.main_thread();
    assert!(Arc::ptr_eq(&jvm.get_current_thread(), &main_thread));
    // dbg!(main_thread.thread_object.borrow().as_ref().unwrap().name().to_rust_string());
    // dbg!(main_thread.thread_object.borrow().as_ref().unwrap().tid());

    jvm.jvmti_state.built_in_jdwp.vm_inited(&jvm);
    let main_stack = Rc::new(StackEntry {
        class_pointer: Arc::new(main_class),
        method_i: main_i as u16,
        local_vars: vec![].into(),//todo handle parameters, todo handle non-zero size locals
        operand_stack: vec![].into(),
        pc: RefCell::new(0),
        pc_offset: 0.into(),
    });
    jvm.main_thread().call_stack.replace(vec![main_stack]);
    jvm.jvmti_state.built_in_jdwp.thread_start(&jvm, jvm.main_thread().thread_object.borrow().clone().unwrap());
    //trigger breakpoint on thread.resume for debuggers that rely on that:

    let thread_class = check_inited_class(&jvm, &ClassName::thread(), jvm.bootstrap_loader.clone());
    let method_i = thread_class.classfile.lookup_method("resume".to_string(), "()V".to_string()).unwrap().0;
    let thread_resume_id = MethodId { class: thread_class, method_i };
    let breakpoints = jvm.jvmti_state.break_points.read().unwrap();
    let breakpoint_offsets = breakpoints.get(&thread_resume_id);
    std::mem::drop(breakpoint_offsets
        .map(|x| x.iter().for_each(|i| {
            //todo handle this breakpoint the usual way
            jvm.jvmti_state.built_in_jdwp.breakpoint(&jvm, thread_resume_id.clone(), *i);
        })));
    std::mem::drop(breakpoints);
    //todo so this DUMB AND VERY TEMPORARY UNTIL I CAN FIGURE OUT A RACE CONDITION LIKELY NOT IN MY CODE.
    std::thread::sleep(Duration::new(2, 0));
    jvm.jvmti_state.built_in_jdwp.breakpoint(&jvm, thread_resume_id.clone(), 0);
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
        call_stack: RefCell::new(vec![Rc::new(bootstrap_frame)]),
        thread_object: RefCell::new(None),
        interpreter_state: InterpreterState {
            terminate: RefCell::new(false),
            throw: RefCell::new(None),
            function_return: RefCell::new(false),
            suspended: RwLock::new(SuspendedStatus {
                suspended: false,
                suspended_lock: Mutex::new(()),
            }),
        },
    }));
    let system_class = check_inited_class(jvm, &ClassName::system(), bl.clone());
    let init_method_view = locate_init_system_class(&system_class);
    let mut locals = vec![];
    for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
        locals.push(JavaValue::Top);
    }
    let initialize_system_frame = StackEntry {
        class_pointer: system_class.clone(),
        method_i: init_method_view.method_i() as u16,
        local_vars: RefCell::new(locals),
        operand_stack: RefCell::new(vec![]),
        pc: RefCell::new(0),
        pc_offset: RefCell::new(-1),
    };
    jvm.get_current_thread().call_stack.replace(vec![Rc::new(initialize_system_frame)]);
    jvm.jvmti_state.built_in_jdwp.agent_load(jvm, &jvm.main_thread());
//todo technically this needs to before any bytecode is run.
    run_function(&jvm);
    if *jvm.main_thread().interpreter_state.function_return.borrow() {
        jvm.main_thread().interpreter_state.function_return.replace(false);
    }
    if jvm.main_thread().interpreter_state.throw.borrow().is_some() || *jvm.main_thread().interpreter_state.terminate.borrow() {
        unimplemented!()
    }
}

fn locate_init_system_class(system: &Arc<RuntimeClass>) -> MethodView {
    let method_views = system.class_view.method_index().lookup_method_name(&"initializeSystemClass".to_string()).unwrap();
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