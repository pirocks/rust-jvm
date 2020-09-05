#![feature(c_variadic)]
#![feature(thread_local)]
#![feature(box_syntax)]
#![feature(vec_into_raw_parts)]
extern crate errno;
extern crate futures_intrusive;
extern crate libc;
extern crate libloading;
extern crate lock_api;
extern crate nix;
extern crate parking_lot;
extern crate regex;
extern crate va_list;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::ffi::c_void;
use std::intrinsics::transmute;
use std::sync::{Arc, RwLock, RwLockWriteGuard};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::thread::LocalKey;
use std::time::Instant;

use libloading::Library;

use classfile_view::loading::{LivePoolGetter, LoaderArc, LoaderName};
use classfile_view::view::ClassView;
use classfile_view::view::method_view::MethodView;
use classfile_view::view::ptype_view::{PTypeView, ReferenceTypeView};
use descriptor_parser::MethodDescriptor;
use jvmti_jni_bindings::jlong;
use jvmti_jni_bindings::JNIInvokeInterface_;
use rust_jvm_common::classfile::{Classfile, CPIndex};
use rust_jvm_common::classnames::ClassName;
use rust_jvm_common::ptype::PType;
use rust_jvm_common::string_pool::StringPool;

use crate::field_table::FieldTable;
use crate::interpreter::run_function;
use crate::interpreter_util::check_inited_class;
use crate::java::lang::string::JString;
use crate::java::lang::system::System;
use crate::java_values::{ArrayObject, JavaValue, NormalObject, Object};
use crate::java_values::Object::Array;
use crate::jvmti::event_callbacks::SharedLibJVMTI;
use crate::loading::{BootstrapLoader, Classpath};
use crate::method_table::{MethodId, MethodTable};
use crate::native_allocation::NativeAllocator;
use crate::runtime_class::RuntimeClass;
use crate::stack_entry::StackEntry;
use crate::threading::{JavaThread, ThreadState};
use crate::tracing::TracingSettings;

// #[macro_export]
// macro_rules! get_state_thread_frame {
//     ($env: expr, $jvm: ident, $thread: ident, $frames: ident, $frame: ident) => {
//         let $jvm = get_state($env);
//         let $thread = get_thread($env);
//         let mut $frames = get_frames(&$thread);
//         let $frame = get_frame(&mut $frames);
//     };
// }

#[macro_use]
pub mod java_values;
#[macro_use]
pub mod runtime_class;
#[macro_use]
pub mod java;
#[macro_use]
pub mod sun;
#[macro_use]
pub mod utils;


#[derive(Debug)]
pub struct InterpreterState {
    pub terminate: bool,
    pub throw: Option<Arc<Object>>,
    pub function_return: bool,
    //todo find some way of clarifying these can only be acessed from one thread
    pub(crate) call_stack: Vec<StackEntry>,
}

impl Default for InterpreterState {
    fn default() -> Self {
        InterpreterState {
            terminate: false,
            throw: None,
            function_return: false,
            /*suspended: RwLock::new(SuspendedStatus {
                suspended: false,
                suspended_lock: Arc::new(Mutex::new(())),
            }),*/
            call_stack: Default::default(),
        }
    }
}

pub struct InterpreterStateGuard<'l> {
    pub int_state: Option<RwLockWriteGuard<'l, InterpreterState>>,
    pub thread: &'l Arc<JavaThread>,
}

impl<'l> InterpreterStateGuard<'l> {
    pub fn current_class_pointer(&self) -> &Arc<RuntimeClass> {
        &self.current_frame().class_pointer
    }

    pub fn current_loader(&self, jvm: &'static JVMState) -> LoaderArc {
        let cp = self.current_class_pointer();
        cp.loader(jvm)
    }

    pub fn current_class_view(&self) -> &Arc<ClassView> {
        self.current_class_pointer().view()
    }


    pub fn current_frame(&'l self) -> &'l StackEntry {
        self.int_state.as_ref().unwrap().call_stack.last().unwrap()
    }

    pub fn current_frame_mut(&mut self) -> &mut StackEntry {
        self.int_state.as_mut().unwrap().call_stack.last_mut().unwrap()
    }

    pub fn push_current_operand_stack(&mut self, jval: JavaValue) {
        self.current_frame_mut().push(jval)
    }

    pub fn pop_current_operand_stack(&mut self) -> JavaValue {
        self.int_state.as_mut().unwrap().call_stack.last_mut().unwrap().operand_stack.pop().unwrap()
    }

    pub fn previous_frame_mut(&mut self) -> &mut StackEntry {
        let call_stack = &mut self.int_state.as_mut().unwrap().call_stack;
        let len = call_stack.len();
        &mut call_stack[len - 2]
    }

    pub fn previous_frame(&self) -> &StackEntry {
        let call_stack = &self.int_state.as_ref().unwrap().call_stack;
        let len = call_stack.len();
        &call_stack[len - 2]
    }

    // pub fn throw_mut(&mut self) -> &mut Option<Arc<Object>> {
    //     &mut self.int_state.as_mut().unwrap().throw
    // }

    pub fn set_throw(&mut self, val: Option<Arc<Object>>) {
        match self.int_state.as_mut() {
            None => {
                self.thread.interpreter_state.write().unwrap().throw = val
            },
            Some(val_mut) => {
                val_mut.throw = val;
            },
        }
    }


    pub fn function_return_mut(&mut self) -> &mut bool {
        &mut self.int_state.as_mut().unwrap().function_return
    }

    pub fn terminate_mut(&mut self) -> &mut bool {
        &mut self.int_state.as_mut().unwrap().terminate
    }


    pub fn throw(&self) -> Option<Arc<Object>> {
        match self.int_state.as_ref() {
            None => {
                self.thread.interpreter_state.read().unwrap().throw.clone()
            },
            Some(int_state) => int_state.throw.clone(),
        }
    }

    pub fn function_return(&self) -> &bool {
        &self.int_state.as_ref().unwrap().function_return
    }

    pub fn terminate(&self) -> &bool {
        &self.int_state.as_ref().unwrap().terminate
    }

    pub fn push_frame(&mut self, frame: StackEntry) {
        self.int_state.as_mut().unwrap().call_stack.push(frame);
    }

    pub fn pop_frame(&mut self) {
        self.int_state.as_mut().unwrap().call_stack.pop();
    }

    pub fn call_stack_depth(&self) -> usize {
        self.int_state.as_ref().unwrap().call_stack.len()
    }

    pub fn current_pc_mut(&mut self) -> &mut usize {
        &mut self.current_frame_mut().pc
    }

    pub fn current_pc(&self) -> &usize {
        &self.current_frame().pc
    }

    pub fn current_pc_offset_mut(&mut self) -> &mut isize {
        &mut self.current_frame_mut().pc_offset
    }

    pub fn current_pc_offset(&'l self) -> &'l isize {
        &self.current_frame().pc_offset
    }

    pub fn current_method_i(&self) -> CPIndex {
        self.current_frame().method_i
    }

    pub fn print_stack_trace(&self) {
        for (i, stack_entry) in self.int_state.as_ref().unwrap().call_stack.iter().enumerate().rev() {
            let name = stack_entry.class_pointer.view().name();
            if stack_entry.method_i > 0 && stack_entry.method_i != u16::MAX {
                let method_view = stack_entry.class_pointer.view().method_view_i(stack_entry.method_i as usize);
                let meth_name = method_view.name();
                println!("{}.{} {} {} pc: {}", name.get_referred_name(), meth_name, method_view.desc_str(), i, stack_entry.pc)
            }
        }
    }
}

#[derive(Debug)]
pub struct SuspendedStatus {
    pub suspended: std::sync::Mutex<bool>,
    pub suspend_condvar: std::sync::Condvar,
}

impl Default for SuspendedStatus {
    fn default() -> Self {
        Self {
            suspended: std::sync::Mutex::new(false),
            suspend_condvar: Default::default(),
        }
    }
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

    pub classes: Classes,

    pub main_class_name: ClassName,

    pub classpath: Arc<Classpath>,
    invoke_interface: RwLock<Option<JNIInvokeInterface_>>,

    pub jvmti_state: Option<JVMTIState>,
    pub thread_state: ThreadState,
    pub tracing: TracingSettings,
    pub method_table: RwLock<MethodTable>,
    pub field_table: RwLock<FieldTable>,
    pub native_interface_allocations: NativeAllocator,
    live: AtomicBool,
    pub int_state_guard: &'static LocalKey<RefCell<Option<*mut InterpreterStateGuard<'static>>>>,//so technically isn't 'static, but we need to be able to store this in a localkey
}

pub struct Classes {
    //todo maybe switch to coarser locking due to probabilty of races here
    pub prepared_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub initializing_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub initialized_classes: RwLock<HashMap<PTypeView, Arc<RuntimeClass>>>,
    pub class_object_pool: RwLock<HashMap<Arc<RuntimeClass>, Arc<Object>>>,
    //anon classes
    pub anon_class_counter: AtomicUsize,
    pub anon_class_live_object_ldc_pool: Arc<RwLock<Vec<Arc<Object>>>>,
}

pub mod threading;

thread_local! {
    static JVMTI_TLS: RefCell<*mut c_void> = RefCell::new(std::ptr::null_mut());
}


thread_local! {
    static INT_STATE_GUARD : RefCell<Option<*mut InterpreterStateGuard<'static>>> = RefCell::new(None);
}


impl JVMState {
    /*pub fn get_current_frame(&self) -> Rc<StackEntry> {
        let current_thread = self.get_current_thread();
        let temp = current_thread.call_stack.borrow();
        temp.last().unwrap().clone()
    }

    pub fn get_previous_frame(&self) -> Rc<StackEntry> {
        let thread = self.get_current_thread();
        let call_stack = thread.call_stack.borrow();
        call_stack.get(call_stack.len() - 2).unwrap().clone()
    }
*/
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
        let thread_state = ThreadState::new();
        let jvm = Self {
            properties,
            loaders: RwLock::new(HashMap::new()),
            bootstrap_loader,
            system_domain_loader: true,
            libjava: LibJavaLoading::new_java_loading(libjava),
            string_pool: StringPool {
                entries: HashSet::new()
            },
            start_instant: Instant::now(),
            classes: Classes {
                prepared_classes: RwLock::new(HashMap::new()),
                initializing_classes: RwLock::new(HashMap::new()),
                initialized_classes: RwLock::new(HashMap::new()),
                class_object_pool: RwLock::new(HashMap::new()),
                anon_class_live_object_ldc_pool: Arc::new(RwLock::new(vec![])),
                anon_class_counter: AtomicUsize::new(0),
            },
            main_class_name,
            classpath: classpath_arc,
            invoke_interface: RwLock::new(None),
            jvmti_state,
            thread_state,
            tracing,
            method_table: RwLock::new(MethodTable::new()),
            field_table: RwLock::new(FieldTable::new()),
            native_interface_allocations: NativeAllocator { allocations: RwLock::new(HashMap::new()) },
            live: AtomicBool::new(false),
            int_state_guard: &INT_STATE_GUARD
        };
        (args, jvm)
    }

    pub fn get_or_create_bootstrap_object_loader<'l>(&'static self, int_state: &mut InterpreterStateGuard) -> JavaValue {//todo this should really take frame as a parameter
        if !self.vm_live() {
            return JavaValue::Object(None);
        }
        let mut loader_guard = self.loaders.write().unwrap();
        match loader_guard.get(&self.bootstrap_loader.name()) {
            None => {
                let java_lang_class_loader = ClassName::new("java/lang/ClassLoader");
                let current_loader = int_state.current_frame_mut().class_pointer.loader(self).clone();
                let class_loader_class = check_inited_class(self, int_state, &java_lang_class_loader.into(), current_loader.clone());
                let res = Arc::new(Object::Object(NormalObject {
                    monitor: self.thread_state.new_monitor("bootstrap loader object monitor".to_string()),
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

    pub unsafe fn get_int_state_guard<'l>(&'static self) -> &'l mut InterpreterStateGuard<'l> {
        let ptr = self.int_state_guard.with(|refcell| refcell.borrow().unwrap());
        &mut *transmute::<_, *mut InterpreterStateGuard<'l>>(ptr)
    }

    pub unsafe fn set_int_state(&'static self, int_state: &mut InterpreterStateGuard) {
        self.int_state_guard.with(|refcell| {
            let ptr = int_state as *mut InterpreterStateGuard;
            refcell.replace(transmute::<_, *mut InterpreterStateGuard<'static>>(ptr).into())
        });
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
        self.live.load(Ordering::SeqCst)
    }

    pub fn get_live_object_pool_getter(&self) -> Arc<dyn LivePoolGetter> {
        Arc::new(LivePoolGetterImpl { anon_class_live_object_ldc_pool: self.classes.anon_class_live_object_ldc_pool.clone() })
    }

    /*pub fn register_main_thread(&self, main_thread: Arc<JavaThread>) {
        //todo perhaps there should be a single rw lock for this
        self.thread_state.alive_threads.write().unwrap().insert(1, main_thread.clone());
        let mut main_thread_writer = self.thread_state.main_thread.write().unwrap();
        main_thread_writer.replace(main_thread.clone().into());
        self.set_current_thread(main_thread);
    }*/

    /*pub fn main_thread(&self) -> Arc<JavaThread> {
        let read_guard = self.thread_state.main_thread.read().unwrap();
        read_guard.as_ref().unwrap().clone()
    }*/
}

pub fn run_main<'l>(args: Vec<String>, jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) -> Result<(), Box<dyn Error>> {
    // let main_view = jvm.bootstrap_loader.load_class(jvm.bootstrap_loader.clone(), &jvm.main_class_name, jvm.bootstrap_loader.clone(), jvm.get_live_object_pool_getter())?;
    // let main_class = prepare_class(&jvm, main_view.backing_class(), jvm.bootstrap_loader.clone());
    let main = check_inited_class(jvm, int_state, &jvm.main_class_name.clone().into(), jvm.bootstrap_loader.clone());
    let main_view = main.view();
    let main_i = locate_main_method(&jvm.bootstrap_loader, &main_view.backing_class());
    let main_thread = jvm.thread_state.get_main_thread();
    assert!(Arc::ptr_eq(&jvm.thread_state.get_current_thread(), &main_thread));
    // assert!(main_thread.get_underlying())//todo check we are running on underlying thread for main
    let num_vars = main_view.method_view_i(main_i).code_attribute().unwrap().max_locals;
    // jvm.jvmti_state.built_in_jdwp.vm_start(&jvm);
    let stack_entry = StackEntry {
        class_pointer: main,
        method_i: main_i as u16,
        local_vars: vec![JavaValue::Top; num_vars as usize],//todo handle parameters, todo handle non-zero size locals
        operand_stack: vec![],
        pc: 0,
        pc_offset: 0,
    };
    int_state.pop_frame();
    int_state.push_frame(stack_entry);

    setup_program_args(&jvm, int_state, args);
    run_function(&jvm, int_state);
    if int_state.throw().is_some() || *int_state.terminate() {
        unimplemented!()
    }
    Result::Ok(())
}


fn setup_program_args<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard, args: Vec<String>) {
    let mut arg_strings: Vec<JavaValue> = vec![];
    for arg_str in args {
        arg_strings.push(JString::from(jvm, int_state, arg_str.clone()).java_value());
    }
    let arg_array = JavaValue::Object(Some(Arc::new(Array(ArrayObject::new_array(
        jvm,
        int_state,
        arg_strings,
        PTypeView::Ref(ReferenceTypeView::Class(ClassName::string())),
        jvm.thread_state.new_monitor("arg array monitor".to_string()),
    )))));
    let local_vars = &mut int_state.current_frame_mut().local_vars;
    local_vars[0] = arg_array;
}


pub struct MainThreadInitializeInfo {
    pub system_class: Arc<RuntimeClass>
}


/*
Runs System.initializeSystemClass, which initializes the entire vm. This function is run on the main rust thread, which is different from the main java thread.
This means that the needed state needs to be transferred over.
 */
pub fn jvm_run_system_init<'l>(jvm: &'static JVMState, sender: Sender<MainThreadInitializeInfo>) -> Result<(), Box<dyn Error>> {
    let bl = &jvm.bootstrap_loader;
    let main_thread = jvm.thread_state.get_main_thread();

    let system_class = check_inited_class(jvm, &mut InterpreterStateGuard { int_state: main_thread.interpreter_state.write().unwrap().into(), thread: &main_thread }, &ClassName::system().into(), bl.clone());

    let init_method_view = locate_init_system_class(&system_class);
    let mut locals = vec![];
    for _ in 0..init_method_view.code_attribute().unwrap().max_locals {
        locals.push(JavaValue::Top);
    }
    let initialize_system_frame = StackEntry {
        class_pointer: system_class.clone(),
        method_i: init_method_view.method_i() as u16,
        local_vars: locals,
        operand_stack: vec![],
        pc: 0,
        pc_offset: 0,
    };
    main_thread.interpreter_state.write().unwrap().call_stack = vec![initialize_system_frame];
    sender.send(MainThreadInitializeInfo { system_class }).unwrap();
    Result::Ok(())
}

fn set_properties<'l>(jvm: &'static JVMState, int_state: &mut InterpreterStateGuard) {
    let properties = &jvm.properties;
    let prop_obj = System::props(jvm, int_state);
    assert_eq!(properties.len() % 2, 0);
    for i in 0..properties.len() / 2 {
        let key_i = 2 * i;
        let value_i = 2 * i + 1;
        let key = JString::from(jvm, int_state, properties[key_i].clone());
        let value = JString::from(jvm, int_state, properties[value_i].clone());
        prop_obj.set_property(jvm, int_state, key, value);
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


pub mod instructions;
pub mod interpreter_util;
pub mod rust_jni;
pub mod loading;
pub mod jvmti;
pub mod invoke_interface;
pub mod stack_entry;
pub mod class_objects;
pub mod tracing;
pub mod interpreter;
pub mod method_table;
pub mod field_table;
pub mod native_allocation;